-- name: CreateUser :one
INSERT INTO users (
    username, email, hashed_password, full_name
) VALUES (
    $1, $2, $3, $4
)
RETURNING *;

-- name: GetUserByEmail :one
SELECT * FROM users
WHERE email = $1 LIMIT 1;

-- name: ListUsers :many
SELECT id, username, email, full_name, created_at FROM users
ORDER BY created_at DESC
LIMIT $1
OFFSET $2;

-- name: CreateProduct :one
INSERT INTO products (
    category_id, name, description, price, stock_quantity, attributes
) VALUES (
    $1, $2, $3, $4, $5, $6
)
RETURNING *;

-- name: GetProductWithCategory :one
SELECT
    p.id,
    p.name,
    p.description,
    p.price,
    p.stock_quantity,
    p.attributes,
    p.created_at,
    c.name as category_name,
    c.slug as category_slug
FROM
    products p
JOIN
    categories c ON p.category_id = c.id
WHERE
    p.id = $1;

-- name: SearchProducts :many
SELECT
    p.*,
    (SELECT AVG(r.rating) FROM reviews r WHERE r.product_id = p.id) as average_rating
FROM products p
WHERE
    (p.name ILIKE sqlc.narg('name') OR p.description ILIKE sqlc.narg('name'))
AND
    p.category_id = ANY(sqlc.slice('category_ids')::int[])
AND
    p.price >= sqlc.narg('min_price')
AND
    p.price <= sqlc.narg('max_price')
AND
    p.stock_quantity > 0
ORDER BY
    p.created_at DESC
LIMIT $1
OFFSET $2;

-- name: GetProductsWithSpecificAttribute :many
-- JSONB内の特定のキーと値で商品を検索する例
-- e.g., '{"brand": "super-brand", "color": "red"}'
SELECT * FROM products
WHERE attributes @> $1::jsonb;


-- name: UpdateProductStock :exec
UPDATE products
SET stock_quantity = stock_quantity + sqlc.arg(add_quantity)
WHERE id = $1;

-- name: CreateOrder :one
-- 実際にはトランザクション内で order_items も作成する必要があります
INSERT INTO orders (user_id, status, total_amount)
VALUES ($1, $2, $3)
RETURNING *;

-- name: CreateOrderItem :one
INSERT INTO order_items (order_id, product_id, quantity, price_at_purchase)
VALUES ($1, $2, $3, $4)
RETURNING *;

-- name: GetOrderDetails :one
SELECT
    o.id as order_id,
    o.status,
    o.total_amount,
    o.ordered_at,
    u.id as user_id,
    u.username,
    u.email
FROM orders o
JOIN users u ON o.user_id = u.id
WHERE o.id = $1;

-- name: ListOrderItemsByOrderID :many
SELECT
    oi.quantity,
    oi.price_at_purchase,
    p.id as product_id,
    p.name as product_name
FROM order_items oi
JOIN products p ON oi.product_id = p.id
WHERE oi.order_id = $1;


-- name: CreateReview :one
INSERT INTO reviews (user_id, product_id, rating, comment)
VALUES ($1, $2, $3, $4)
RETURNING *;

-- name: GetProductAverageRating :one
SELECT
    product_id,
    AVG(rating)::float as average_rating,
    COUNT(id) as review_count
FROM reviews
WHERE product_id = $1
GROUP BY product_id;

-- name: GetCategorySalesRanking :many
-- 複雑なJOINと集計の例
SELECT
    c.id as category_id,
    c.name as category_name,
    SUM(oi.quantity * oi.price_at_purchase) as total_sales,
    COUNT(DISTINCT o.id) as total_orders
FROM categories c
JOIN products p ON c.id = p.category_id
JOIN order_items oi ON p.id = oi.product_id
JOIN orders o ON oi.order_id = o.id
WHERE o.status IN ('delivered', 'shipped')
GROUP BY c.id, c.name
ORDER BY total_sales DESC;

-- name: DeleteUserAndRelatedData :exec
-- ON DELETE CASCADEの動作確認用
DELETE FROM users WHERE id = $1;