#[derive(Debug, Clone, Copy, postgres_types::ToSql, postgres_types::FromSql)]
#[postgres(name = "order_status")]
enum OrderStatus {
    #[postgres(name = "pending")]
    Pending,
    #[postgres(name = "processing")]
    Processing,
    #[postgres(name = "shipped")]
    Shipped,
    #[postgres(name = "delivered")]
    Delivered,
    #[postgres(name = "cancelled")]
    Cancelled,
}
struct CreateUserRow {
    users_id: uuid::Uuid,
    users_username: String,
    users_email: String,
    users_hashed_password: String,
    users_full_name: Option<String>,
    users_created_at: std::time::SystemTime,
    users_updated_at: std::time::SystemTime,
}
impl CreateUserRow {
    fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        Ok(Self {
            users_id: row.try_get(0)?,
            users_username: row.try_get(1)?,
            users_email: row.try_get(2)?,
            users_hashed_password: row.try_get(3)?,
            users_full_name: row.try_get(4)?,
            users_created_at: row.try_get(5)?,
            users_updated_at: row.try_get(6)?,
        })
    }
}
struct CreateUser<'a> {
    users_username: &'a str,
    users_email: &'a str,
    users_hashed_password: &'a str,
    users_full_name: Option<&'a str>,
}
impl<'a> CreateUser<'a> {
    pub const QUERY: &'static str = r"INSERT INTO users (
    username, email, hashed_password, full_name
) VALUES (
    $1, $2, $3, $4
)
RETURNING id, username, email, hashed_password, full_name, created_at, updated_at";
    async fn query_one(
        &self,
        client: &impl tokio_postgres::GenericClient,
    ) -> Result<CreateUserRow, tokio_postgres::Error> {
        let row = client
            .query_one(
                Self::QUERY,
                &[
                    &self.users_username,
                    &self.users_email,
                    &self.users_hashed_password,
                    &self.users_full_name,
                ],
            )
            .await?;
        CreateUserRow::from_row(&row)
    }
    async fn query_opt(
        &self,
        client: &impl tokio_postgres::GenericClient,
    ) -> Result<Option<CreateUserRow>, tokio_postgres::Error> {
        let row = client
            .query_opt(
                Self::QUERY,
                &[
                    &self.users_username,
                    &self.users_email,
                    &self.users_hashed_password,
                    &self.users_full_name,
                ],
            )
            .await?;
        match row {
            Some(row) => Ok(Some(CreateUserRow::from_row(&row)?)),
            None => Ok(None),
        }
    }
}
impl<'a> CreateUser<'a> {
    const fn builder() -> CreateUserBuilder<'a, ((), (), (), ())> {
        CreateUserBuilder {
            fields: ((), (), (), ()),
            _phantom: std::marker::PhantomData,
        }
    }
}
struct CreateUserBuilder<'a, Fields = ((), (), (), ())> {
    fields: Fields,
    _phantom: std::marker::PhantomData<&'a ()>,
}
impl<'a> CreateUserBuilder<'a, (&'a str, &'a str, &'a str, Option<&'a str>)> {
    const fn build(self) -> CreateUser<'a> {
        let (users_username, users_email, users_hashed_password, users_full_name) = self.fields;
        CreateUser {
            users_username,
            users_email,
            users_hashed_password,
            users_full_name,
        }
    }
}
struct GetUserByEmailRow {
    users_id: uuid::Uuid,
    users_username: String,
    users_email: String,
    users_hashed_password: String,
    users_full_name: Option<String>,
    users_created_at: std::time::SystemTime,
    users_updated_at: std::time::SystemTime,
}
impl GetUserByEmailRow {
    fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        Ok(Self {
            users_id: row.try_get(0)?,
            users_username: row.try_get(1)?,
            users_email: row.try_get(2)?,
            users_hashed_password: row.try_get(3)?,
            users_full_name: row.try_get(4)?,
            users_created_at: row.try_get(5)?,
            users_updated_at: row.try_get(6)?,
        })
    }
}
struct GetUserByEmail<'a> {
    users_email: &'a str,
}
impl<'a> GetUserByEmail<'a> {
    pub const QUERY: &'static str = r"SELECT id, username, email, hashed_password, full_name, created_at, updated_at FROM users
WHERE email = $1 LIMIT 1";
    async fn query_one(
        &self,
        client: &impl tokio_postgres::GenericClient,
    ) -> Result<GetUserByEmailRow, tokio_postgres::Error> {
        let row = client.query_one(Self::QUERY, &[&self.users_email]).await?;
        GetUserByEmailRow::from_row(&row)
    }
    async fn query_opt(
        &self,
        client: &impl tokio_postgres::GenericClient,
    ) -> Result<Option<GetUserByEmailRow>, tokio_postgres::Error> {
        let row = client.query_opt(Self::QUERY, &[&self.users_email]).await?;
        match row {
            Some(row) => Ok(Some(GetUserByEmailRow::from_row(&row)?)),
            None => Ok(None),
        }
    }
}
impl<'a> GetUserByEmail<'a> {
    const fn builder() -> GetUserByEmailBuilder<'a, ((),)> {
        GetUserByEmailBuilder {
            fields: ((),),
            _phantom: std::marker::PhantomData,
        }
    }
}
struct GetUserByEmailBuilder<'a, Fields = ((),)> {
    fields: Fields,
    _phantom: std::marker::PhantomData<&'a ()>,
}
impl<'a> GetUserByEmailBuilder<'a, (&'a str,)> {
    const fn build(self) -> GetUserByEmail<'a> {
        let (users_email,) = self.fields;
        GetUserByEmail { users_email }
    }
}
struct ListUsersRow {
    users_id: uuid::Uuid,
    users_username: String,
    users_email: String,
    users_full_name: Option<String>,
    users_created_at: std::time::SystemTime,
}
impl ListUsersRow {
    fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        Ok(Self {
            users_id: row.try_get(0)?,
            users_username: row.try_get(1)?,
            users_email: row.try_get(2)?,
            users_full_name: row.try_get(3)?,
            users_created_at: row.try_get(4)?,
        })
    }
}
struct ListUsers {
    limit: i32,
    offset: i32,
}
impl ListUsers {
    pub const QUERY: &'static str = r"SELECT id, username, email, full_name, created_at FROM users
ORDER BY created_at DESC
LIMIT $1
OFFSET $2";
    async fn query_many(
        &self,
        client: &impl tokio_postgres::GenericClient,
    ) -> Result<Vec<ListUsersRow>, tokio_postgres::Error> {
        let rows = client
            .query(Self::QUERY, &[&self.limit, &self.offset])
            .await?;
        rows.into_iter()
            .map(|r| ListUsersRow::from_row(&r))
            .collect()
    }
}
impl ListUsers {
    const fn builder() -> ListUsersBuilder<'static, ((), ())> {
        ListUsersBuilder {
            fields: ((), ()),
            _phantom: std::marker::PhantomData,
        }
    }
}
struct ListUsersBuilder<'a, Fields = ((), ())> {
    fields: Fields,
    _phantom: std::marker::PhantomData<&'a ()>,
}
impl<'a> ListUsersBuilder<'a, (i32, i32)> {
    const fn build(self) -> ListUsers {
        let (limit, offset) = self.fields;
        ListUsers { limit, offset }
    }
}
struct CreateProductRow {
    products_id: uuid::Uuid,
    products_category_id: i32,
    products_name: String,
    products_description: Option<String>,
    products_price: i32,
    products_stock_quantity: i32,
    products_attributes: Option<serde_json::Value>,
    products_created_at: std::time::SystemTime,
    products_updated_at: std::time::SystemTime,
}
impl CreateProductRow {
    fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        Ok(Self {
            products_id: row.try_get(0)?,
            products_category_id: row.try_get(1)?,
            products_name: row.try_get(2)?,
            products_description: row.try_get(3)?,
            products_price: row.try_get(4)?,
            products_stock_quantity: row.try_get(5)?,
            products_attributes: row.try_get(6)?,
            products_created_at: row.try_get(7)?,
            products_updated_at: row.try_get(8)?,
        })
    }
}
struct CreateProduct<'a> {
    products_category_id: i32,
    products_name: &'a str,
    products_description: Option<&'a str>,
    products_price: i32,
    products_stock_quantity: i32,
    products_attributes: Option<&'a serde_json::Value>,
}
impl<'a> CreateProduct<'a> {
    pub const QUERY: &'static str = r"INSERT INTO products (
    category_id, name, description, price, stock_quantity, attributes
) VALUES (
    $1, $2, $3, $4, $5, $6
)
RETURNING id, category_id, name, description, price, stock_quantity, attributes, created_at, updated_at";
    async fn query_one(
        &self,
        client: &impl tokio_postgres::GenericClient,
    ) -> Result<CreateProductRow, tokio_postgres::Error> {
        let row = client
            .query_one(
                Self::QUERY,
                &[
                    &self.products_category_id,
                    &self.products_name,
                    &self.products_description,
                    &self.products_price,
                    &self.products_stock_quantity,
                    &self.products_attributes,
                ],
            )
            .await?;
        CreateProductRow::from_row(&row)
    }
    async fn query_opt(
        &self,
        client: &impl tokio_postgres::GenericClient,
    ) -> Result<Option<CreateProductRow>, tokio_postgres::Error> {
        let row = client
            .query_opt(
                Self::QUERY,
                &[
                    &self.products_category_id,
                    &self.products_name,
                    &self.products_description,
                    &self.products_price,
                    &self.products_stock_quantity,
                    &self.products_attributes,
                ],
            )
            .await?;
        match row {
            Some(row) => Ok(Some(CreateProductRow::from_row(&row)?)),
            None => Ok(None),
        }
    }
}
impl<'a> CreateProduct<'a> {
    const fn builder() -> CreateProductBuilder<'a, ((), (), (), (), (), ())> {
        CreateProductBuilder {
            fields: ((), (), (), (), (), ()),
            _phantom: std::marker::PhantomData,
        }
    }
}
struct CreateProductBuilder<'a, Fields = ((), (), (), (), (), ())> {
    fields: Fields,
    _phantom: std::marker::PhantomData<&'a ()>,
}
impl<'a>
    CreateProductBuilder<
        'a,
        (
            i32,
            &'a str,
            Option<&'a str>,
            i32,
            i32,
            Option<&'a serde_json::Value>,
        ),
    >
{
    const fn build(self) -> CreateProduct<'a> {
        let (
            products_category_id,
            products_name,
            products_description,
            products_price,
            products_stock_quantity,
            products_attributes,
        ) = self.fields;
        CreateProduct {
            products_category_id,
            products_name,
            products_description,
            products_price,
            products_stock_quantity,
            products_attributes,
        }
    }
}
struct GetProductWithCategoryRow {
    products_id: uuid::Uuid,
    products_name: String,
    products_description: Option<String>,
    products_price: i32,
    products_stock_quantity: i32,
    products_attributes: Option<serde_json::Value>,
    products_created_at: std::time::SystemTime,
    categories_category_name: String,
    categories_category_slug: String,
}
impl GetProductWithCategoryRow {
    fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        Ok(Self {
            products_id: row.try_get(0)?,
            products_name: row.try_get(1)?,
            products_description: row.try_get(2)?,
            products_price: row.try_get(3)?,
            products_stock_quantity: row.try_get(4)?,
            products_attributes: row.try_get(5)?,
            products_created_at: row.try_get(6)?,
            categories_category_name: row.try_get(7)?,
            categories_category_slug: row.try_get(8)?,
        })
    }
}
struct GetProductWithCategory {
    products_id: uuid::Uuid,
}
impl GetProductWithCategory {
    pub const QUERY: &'static str = r"SELECT
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
    p.id = $1";
    async fn query_one(
        &self,
        client: &impl tokio_postgres::GenericClient,
    ) -> Result<GetProductWithCategoryRow, tokio_postgres::Error> {
        let row = client.query_one(Self::QUERY, &[&self.products_id]).await?;
        GetProductWithCategoryRow::from_row(&row)
    }
    async fn query_opt(
        &self,
        client: &impl tokio_postgres::GenericClient,
    ) -> Result<Option<GetProductWithCategoryRow>, tokio_postgres::Error> {
        let row = client.query_opt(Self::QUERY, &[&self.products_id]).await?;
        match row {
            Some(row) => Ok(Some(GetProductWithCategoryRow::from_row(&row)?)),
            None => Ok(None),
        }
    }
}
impl GetProductWithCategory {
    const fn builder() -> GetProductWithCategoryBuilder<'static, ((),)> {
        GetProductWithCategoryBuilder {
            fields: ((),),
            _phantom: std::marker::PhantomData,
        }
    }
}
struct GetProductWithCategoryBuilder<'a, Fields = ((),)> {
    fields: Fields,
    _phantom: std::marker::PhantomData<&'a ()>,
}
impl<'a> GetProductWithCategoryBuilder<'a, (uuid::Uuid,)> {
    const fn build(self) -> GetProductWithCategory {
        let (products_id,) = self.fields;
        GetProductWithCategory { products_id }
    }
}
struct SearchProductsRow {
    products_id: uuid::Uuid,
    products_category_id: i32,
    products_name: String,
    products_description: Option<String>,
    products_price: i32,
    products_stock_quantity: i32,
    products_attributes: Option<serde_json::Value>,
    products_created_at: std::time::SystemTime,
    products_updated_at: std::time::SystemTime,
    average_rating: f64,
}
impl SearchProductsRow {
    fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        Ok(Self {
            products_id: row.try_get(0)?,
            products_category_id: row.try_get(1)?,
            products_name: row.try_get(2)?,
            products_description: row.try_get(3)?,
            products_price: row.try_get(4)?,
            products_stock_quantity: row.try_get(5)?,
            products_attributes: row.try_get(6)?,
            products_created_at: row.try_get(7)?,
            products_updated_at: row.try_get(8)?,
            average_rating: row.try_get(9)?,
        })
    }
}
struct SearchProducts<'a> {
    limit: i32,
    offset: i32,
    products_name: Option<&'a str>,
    category_ids: &'a [i32],
    products_min_price: Option<i32>,
    products_max_price: Option<i32>,
}
impl<'a> SearchProducts<'a> {
    pub const QUERY: &'static str = r"SELECT
    p.id, p.category_id, p.name, p.description, p.price, p.stock_quantity, p.attributes, p.created_at, p.updated_at,
    (SELECT AVG(r.rating) FROM reviews r WHERE r.product_id = p.id) as average_rating
FROM products p
WHERE
    (p.name ILIKE $3 OR p.description ILIKE $3)
AND
    p.category_id = ANY($4::int[])
AND
    p.price >= $5
AND
    p.price <= $6
AND
    p.stock_quantity > 0
ORDER BY
    p.created_at DESC
LIMIT $1
OFFSET $2";
    async fn query_many(
        &self,
        client: &impl tokio_postgres::GenericClient,
    ) -> Result<Vec<SearchProductsRow>, tokio_postgres::Error> {
        let rows = client
            .query(
                Self::QUERY,
                &[
                    &self.limit,
                    &self.offset,
                    &self.products_name,
                    &self.category_ids,
                    &self.products_min_price,
                    &self.products_max_price,
                ],
            )
            .await?;
        rows.into_iter()
            .map(|r| SearchProductsRow::from_row(&r))
            .collect()
    }
}
impl<'a> SearchProducts<'a> {
    const fn builder() -> SearchProductsBuilder<'a, ((), (), (), (), (), ())> {
        SearchProductsBuilder {
            fields: ((), (), (), (), (), ()),
            _phantom: std::marker::PhantomData,
        }
    }
}
struct SearchProductsBuilder<'a, Fields = ((), (), (), (), (), ())> {
    fields: Fields,
    _phantom: std::marker::PhantomData<&'a ()>,
}
impl<'a>
    SearchProductsBuilder<
        'a,
        (
            i32,
            i32,
            Option<&'a str>,
            &'a [i32],
            Option<i32>,
            Option<i32>,
        ),
    >
{
    const fn build(self) -> SearchProducts<'a> {
        let (limit, offset, products_name, category_ids, products_min_price, products_max_price) =
            self.fields;
        SearchProducts {
            limit,
            offset,
            products_name,
            category_ids,
            products_min_price,
            products_max_price,
        }
    }
}
struct GetProductsWithSpecificAttributeRow {
    products_id: uuid::Uuid,
    products_category_id: i32,
    products_name: String,
    products_description: Option<String>,
    products_price: i32,
    products_stock_quantity: i32,
    products_attributes: Option<serde_json::Value>,
    products_created_at: std::time::SystemTime,
    products_updated_at: std::time::SystemTime,
}
impl GetProductsWithSpecificAttributeRow {
    fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        Ok(Self {
            products_id: row.try_get(0)?,
            products_category_id: row.try_get(1)?,
            products_name: row.try_get(2)?,
            products_description: row.try_get(3)?,
            products_price: row.try_get(4)?,
            products_stock_quantity: row.try_get(5)?,
            products_attributes: row.try_get(6)?,
            products_created_at: row.try_get(7)?,
            products_updated_at: row.try_get(8)?,
        })
    }
}
struct GetProductsWithSpecificAttribute<'a> {
    param: &'a serde_json::Value,
}
impl<'a> GetProductsWithSpecificAttribute<'a> {
    pub const QUERY: &'static str = r"SELECT id, category_id, name, description, price, stock_quantity, attributes, created_at, updated_at FROM products
WHERE attributes @> $1::jsonb";
    async fn query_many(
        &self,
        client: &impl tokio_postgres::GenericClient,
    ) -> Result<Vec<GetProductsWithSpecificAttributeRow>, tokio_postgres::Error> {
        let rows = client.query(Self::QUERY, &[&self.param]).await?;
        rows.into_iter()
            .map(|r| GetProductsWithSpecificAttributeRow::from_row(&r))
            .collect()
    }
}
impl<'a> GetProductsWithSpecificAttribute<'a> {
    const fn builder() -> GetProductsWithSpecificAttributeBuilder<'a, ((),)> {
        GetProductsWithSpecificAttributeBuilder {
            fields: ((),),
            _phantom: std::marker::PhantomData,
        }
    }
}
struct GetProductsWithSpecificAttributeBuilder<'a, Fields = ((),)> {
    fields: Fields,
    _phantom: std::marker::PhantomData<&'a ()>,
}
impl<'a> GetProductsWithSpecificAttributeBuilder<'a, (&'a serde_json::Value,)> {
    const fn build(self) -> GetProductsWithSpecificAttribute<'a> {
        let (param,) = self.fields;
        GetProductsWithSpecificAttribute { param }
    }
}
struct UpdateProductStockRow {}
impl UpdateProductStockRow {
    fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        Ok(Self {})
    }
}
struct UpdateProductStock {
    products_id: uuid::Uuid,
    products_add_quantity: i32,
}
impl UpdateProductStock {
    pub const QUERY: &'static str = r"UPDATE products
SET stock_quantity = stock_quantity + $2
WHERE id = $1";
    async fn execute(
        &self,
        client: &impl tokio_postgres::GenericClient,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(
                Self::QUERY,
                &[&self.products_id, &self.products_add_quantity],
            )
            .await
    }
}
impl UpdateProductStock {
    const fn builder() -> UpdateProductStockBuilder<'static, ((), ())> {
        UpdateProductStockBuilder {
            fields: ((), ()),
            _phantom: std::marker::PhantomData,
        }
    }
}
struct UpdateProductStockBuilder<'a, Fields = ((), ())> {
    fields: Fields,
    _phantom: std::marker::PhantomData<&'a ()>,
}
impl<'a> UpdateProductStockBuilder<'a, (uuid::Uuid, i32)> {
    const fn build(self) -> UpdateProductStock {
        let (products_id, products_add_quantity) = self.fields;
        UpdateProductStock {
            products_id,
            products_add_quantity,
        }
    }
}
struct CreateOrderRow {
    orders_id: i64,
    orders_user_id: uuid::Uuid,
    orders_status: OrderStatus,
    orders_total_amount: i32,
    orders_ordered_at: std::time::SystemTime,
}
impl CreateOrderRow {
    fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        Ok(Self {
            orders_id: row.try_get(0)?,
            orders_user_id: row.try_get(1)?,
            orders_status: row.try_get(2)?,
            orders_total_amount: row.try_get(3)?,
            orders_ordered_at: row.try_get(4)?,
        })
    }
}
struct CreateOrder {
    orders_user_id: uuid::Uuid,
    orders_status: OrderStatus,
    orders_total_amount: i32,
}
impl CreateOrder {
    pub const QUERY: &'static str = r"INSERT INTO orders (user_id, status, total_amount)
VALUES ($1, $2, $3)
RETURNING id, user_id, status, total_amount, ordered_at";
    async fn query_one(
        &self,
        client: &impl tokio_postgres::GenericClient,
    ) -> Result<CreateOrderRow, tokio_postgres::Error> {
        let row = client
            .query_one(
                Self::QUERY,
                &[
                    &self.orders_user_id,
                    &self.orders_status,
                    &self.orders_total_amount,
                ],
            )
            .await?;
        CreateOrderRow::from_row(&row)
    }
    async fn query_opt(
        &self,
        client: &impl tokio_postgres::GenericClient,
    ) -> Result<Option<CreateOrderRow>, tokio_postgres::Error> {
        let row = client
            .query_opt(
                Self::QUERY,
                &[
                    &self.orders_user_id,
                    &self.orders_status,
                    &self.orders_total_amount,
                ],
            )
            .await?;
        match row {
            Some(row) => Ok(Some(CreateOrderRow::from_row(&row)?)),
            None => Ok(None),
        }
    }
}
impl CreateOrder {
    const fn builder() -> CreateOrderBuilder<'static, ((), (), ())> {
        CreateOrderBuilder {
            fields: ((), (), ()),
            _phantom: std::marker::PhantomData,
        }
    }
}
struct CreateOrderBuilder<'a, Fields = ((), (), ())> {
    fields: Fields,
    _phantom: std::marker::PhantomData<&'a ()>,
}
impl<'a> CreateOrderBuilder<'a, (uuid::Uuid, OrderStatus, i32)> {
    const fn build(self) -> CreateOrder {
        let (orders_user_id, orders_status, orders_total_amount) = self.fields;
        CreateOrder {
            orders_user_id,
            orders_status,
            orders_total_amount,
        }
    }
}
struct CreateOrderItemRow {
    order_items_id: i64,
    order_items_order_id: i64,
    order_items_product_id: uuid::Uuid,
    order_items_quantity: i32,
    order_items_price_at_purchase: i32,
}
impl CreateOrderItemRow {
    fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        Ok(Self {
            order_items_id: row.try_get(0)?,
            order_items_order_id: row.try_get(1)?,
            order_items_product_id: row.try_get(2)?,
            order_items_quantity: row.try_get(3)?,
            order_items_price_at_purchase: row.try_get(4)?,
        })
    }
}
struct CreateOrderItem {
    order_items_order_id: i64,
    order_items_product_id: uuid::Uuid,
    order_items_quantity: i32,
    order_items_price_at_purchase: i32,
}
impl CreateOrderItem {
    pub const QUERY: &'static str = r"INSERT INTO order_items (order_id, product_id, quantity, price_at_purchase)
VALUES ($1, $2, $3, $4)
RETURNING id, order_id, product_id, quantity, price_at_purchase";
    async fn query_one(
        &self,
        client: &impl tokio_postgres::GenericClient,
    ) -> Result<CreateOrderItemRow, tokio_postgres::Error> {
        let row = client
            .query_one(
                Self::QUERY,
                &[
                    &self.order_items_order_id,
                    &self.order_items_product_id,
                    &self.order_items_quantity,
                    &self.order_items_price_at_purchase,
                ],
            )
            .await?;
        CreateOrderItemRow::from_row(&row)
    }
    async fn query_opt(
        &self,
        client: &impl tokio_postgres::GenericClient,
    ) -> Result<Option<CreateOrderItemRow>, tokio_postgres::Error> {
        let row = client
            .query_opt(
                Self::QUERY,
                &[
                    &self.order_items_order_id,
                    &self.order_items_product_id,
                    &self.order_items_quantity,
                    &self.order_items_price_at_purchase,
                ],
            )
            .await?;
        match row {
            Some(row) => Ok(Some(CreateOrderItemRow::from_row(&row)?)),
            None => Ok(None),
        }
    }
}
impl CreateOrderItem {
    const fn builder() -> CreateOrderItemBuilder<'static, ((), (), (), ())> {
        CreateOrderItemBuilder {
            fields: ((), (), (), ()),
            _phantom: std::marker::PhantomData,
        }
    }
}
struct CreateOrderItemBuilder<'a, Fields = ((), (), (), ())> {
    fields: Fields,
    _phantom: std::marker::PhantomData<&'a ()>,
}
impl<'a> CreateOrderItemBuilder<'a, (i64, uuid::Uuid, i32, i32)> {
    const fn build(self) -> CreateOrderItem {
        let (
            order_items_order_id,
            order_items_product_id,
            order_items_quantity,
            order_items_price_at_purchase,
        ) = self.fields;
        CreateOrderItem {
            order_items_order_id,
            order_items_product_id,
            order_items_quantity,
            order_items_price_at_purchase,
        }
    }
}
struct GetOrderDetailsRow {
    orders_order_id: i64,
    orders_status: OrderStatus,
    orders_total_amount: i32,
    orders_ordered_at: std::time::SystemTime,
    users_user_id: uuid::Uuid,
    users_username: String,
    users_email: String,
}
impl GetOrderDetailsRow {
    fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        Ok(Self {
            orders_order_id: row.try_get(0)?,
            orders_status: row.try_get(1)?,
            orders_total_amount: row.try_get(2)?,
            orders_ordered_at: row.try_get(3)?,
            users_user_id: row.try_get(4)?,
            users_username: row.try_get(5)?,
            users_email: row.try_get(6)?,
        })
    }
}
struct GetOrderDetails {
    orders_id: i64,
}
impl GetOrderDetails {
    pub const QUERY: &'static str = r"SELECT
    o.id as order_id,
    o.status,
    o.total_amount,
    o.ordered_at,
    u.id as user_id,
    u.username,
    u.email
FROM orders o
JOIN users u ON o.user_id = u.id
WHERE o.id = $1";
    async fn query_one(
        &self,
        client: &impl tokio_postgres::GenericClient,
    ) -> Result<GetOrderDetailsRow, tokio_postgres::Error> {
        let row = client.query_one(Self::QUERY, &[&self.orders_id]).await?;
        GetOrderDetailsRow::from_row(&row)
    }
    async fn query_opt(
        &self,
        client: &impl tokio_postgres::GenericClient,
    ) -> Result<Option<GetOrderDetailsRow>, tokio_postgres::Error> {
        let row = client.query_opt(Self::QUERY, &[&self.orders_id]).await?;
        match row {
            Some(row) => Ok(Some(GetOrderDetailsRow::from_row(&row)?)),
            None => Ok(None),
        }
    }
}
impl GetOrderDetails {
    const fn builder() -> GetOrderDetailsBuilder<'static, ((),)> {
        GetOrderDetailsBuilder {
            fields: ((),),
            _phantom: std::marker::PhantomData,
        }
    }
}
struct GetOrderDetailsBuilder<'a, Fields = ((),)> {
    fields: Fields,
    _phantom: std::marker::PhantomData<&'a ()>,
}
impl<'a> GetOrderDetailsBuilder<'a, (i64,)> {
    const fn build(self) -> GetOrderDetails {
        let (orders_id,) = self.fields;
        GetOrderDetails { orders_id }
    }
}
struct ListOrderItemsByOrderIdRow {
    order_items_quantity: i32,
    order_items_price_at_purchase: i32,
    products_product_id: uuid::Uuid,
    products_product_name: String,
}
impl ListOrderItemsByOrderIdRow {
    fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        Ok(Self {
            order_items_quantity: row.try_get(0)?,
            order_items_price_at_purchase: row.try_get(1)?,
            products_product_id: row.try_get(2)?,
            products_product_name: row.try_get(3)?,
        })
    }
}
struct ListOrderItemsByOrderId {
    order_items_order_id: i64,
}
impl ListOrderItemsByOrderId {
    pub const QUERY: &'static str = r"SELECT
    oi.quantity,
    oi.price_at_purchase,
    p.id as product_id,
    p.name as product_name
FROM order_items oi
JOIN products p ON oi.product_id = p.id
WHERE oi.order_id = $1";
    async fn query_many(
        &self,
        client: &impl tokio_postgres::GenericClient,
    ) -> Result<Vec<ListOrderItemsByOrderIdRow>, tokio_postgres::Error> {
        let rows = client
            .query(Self::QUERY, &[&self.order_items_order_id])
            .await?;
        rows.into_iter()
            .map(|r| ListOrderItemsByOrderIdRow::from_row(&r))
            .collect()
    }
}
impl ListOrderItemsByOrderId {
    const fn builder() -> ListOrderItemsByOrderIdBuilder<'static, ((),)> {
        ListOrderItemsByOrderIdBuilder {
            fields: ((),),
            _phantom: std::marker::PhantomData,
        }
    }
}
struct ListOrderItemsByOrderIdBuilder<'a, Fields = ((),)> {
    fields: Fields,
    _phantom: std::marker::PhantomData<&'a ()>,
}
impl<'a> ListOrderItemsByOrderIdBuilder<'a, (i64,)> {
    const fn build(self) -> ListOrderItemsByOrderId {
        let (order_items_order_id,) = self.fields;
        ListOrderItemsByOrderId {
            order_items_order_id,
        }
    }
}
struct CreateReviewRow {
    reviews_id: i64,
    reviews_user_id: uuid::Uuid,
    reviews_product_id: uuid::Uuid,
    reviews_rating: i32,
    reviews_comment: Option<String>,
    reviews_created_at: std::time::SystemTime,
}
impl CreateReviewRow {
    fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        Ok(Self {
            reviews_id: row.try_get(0)?,
            reviews_user_id: row.try_get(1)?,
            reviews_product_id: row.try_get(2)?,
            reviews_rating: row.try_get(3)?,
            reviews_comment: row.try_get(4)?,
            reviews_created_at: row.try_get(5)?,
        })
    }
}
struct CreateReview<'a> {
    reviews_user_id: uuid::Uuid,
    reviews_product_id: uuid::Uuid,
    reviews_rating: i32,
    reviews_comment: Option<&'a str>,
}
impl<'a> CreateReview<'a> {
    pub const QUERY: &'static str = r"INSERT INTO reviews (user_id, product_id, rating, comment)
VALUES ($1, $2, $3, $4)
RETURNING id, user_id, product_id, rating, comment, created_at";
    async fn query_one(
        &self,
        client: &impl tokio_postgres::GenericClient,
    ) -> Result<CreateReviewRow, tokio_postgres::Error> {
        let row = client
            .query_one(
                Self::QUERY,
                &[
                    &self.reviews_user_id,
                    &self.reviews_product_id,
                    &self.reviews_rating,
                    &self.reviews_comment,
                ],
            )
            .await?;
        CreateReviewRow::from_row(&row)
    }
    async fn query_opt(
        &self,
        client: &impl tokio_postgres::GenericClient,
    ) -> Result<Option<CreateReviewRow>, tokio_postgres::Error> {
        let row = client
            .query_opt(
                Self::QUERY,
                &[
                    &self.reviews_user_id,
                    &self.reviews_product_id,
                    &self.reviews_rating,
                    &self.reviews_comment,
                ],
            )
            .await?;
        match row {
            Some(row) => Ok(Some(CreateReviewRow::from_row(&row)?)),
            None => Ok(None),
        }
    }
}
impl<'a> CreateReview<'a> {
    const fn builder() -> CreateReviewBuilder<'a, ((), (), (), ())> {
        CreateReviewBuilder {
            fields: ((), (), (), ()),
            _phantom: std::marker::PhantomData,
        }
    }
}
struct CreateReviewBuilder<'a, Fields = ((), (), (), ())> {
    fields: Fields,
    _phantom: std::marker::PhantomData<&'a ()>,
}
impl<'a> CreateReviewBuilder<'a, (uuid::Uuid, uuid::Uuid, i32, Option<&'a str>)> {
    const fn build(self) -> CreateReview<'a> {
        let (reviews_user_id, reviews_product_id, reviews_rating, reviews_comment) = self.fields;
        CreateReview {
            reviews_user_id,
            reviews_product_id,
            reviews_rating,
            reviews_comment,
        }
    }
}
struct GetProductAverageRatingRow {
    reviews_product_id: uuid::Uuid,
    average_rating: f64,
    review_count: i64,
}
impl GetProductAverageRatingRow {
    fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        Ok(Self {
            reviews_product_id: row.try_get(0)?,
            average_rating: row.try_get(1)?,
            review_count: row.try_get(2)?,
        })
    }
}
struct GetProductAverageRating {
    reviews_product_id: uuid::Uuid,
}
impl GetProductAverageRating {
    pub const QUERY: &'static str = r"SELECT
    product_id,
    AVG(rating)::float as average_rating,
    COUNT(id) as review_count
FROM reviews
WHERE product_id = $1
GROUP BY product_id";
    async fn query_one(
        &self,
        client: &impl tokio_postgres::GenericClient,
    ) -> Result<GetProductAverageRatingRow, tokio_postgres::Error> {
        let row = client
            .query_one(Self::QUERY, &[&self.reviews_product_id])
            .await?;
        GetProductAverageRatingRow::from_row(&row)
    }
    async fn query_opt(
        &self,
        client: &impl tokio_postgres::GenericClient,
    ) -> Result<Option<GetProductAverageRatingRow>, tokio_postgres::Error> {
        let row = client
            .query_opt(Self::QUERY, &[&self.reviews_product_id])
            .await?;
        match row {
            Some(row) => Ok(Some(GetProductAverageRatingRow::from_row(&row)?)),
            None => Ok(None),
        }
    }
}
impl GetProductAverageRating {
    const fn builder() -> GetProductAverageRatingBuilder<'static, ((),)> {
        GetProductAverageRatingBuilder {
            fields: ((),),
            _phantom: std::marker::PhantomData,
        }
    }
}
struct GetProductAverageRatingBuilder<'a, Fields = ((),)> {
    fields: Fields,
    _phantom: std::marker::PhantomData<&'a ()>,
}
impl<'a> GetProductAverageRatingBuilder<'a, (uuid::Uuid,)> {
    const fn build(self) -> GetProductAverageRating {
        let (reviews_product_id,) = self.fields;
        GetProductAverageRating { reviews_product_id }
    }
}
struct GetCategorySalesRankingRow {
    categories_category_id: i32,
    categories_category_name: String,
    total_sales: i64,
    total_orders: i64,
}
impl GetCategorySalesRankingRow {
    fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        Ok(Self {
            categories_category_id: row.try_get(0)?,
            categories_category_name: row.try_get(1)?,
            total_sales: row.try_get(2)?,
            total_orders: row.try_get(3)?,
        })
    }
}
struct GetCategorySalesRanking;
impl GetCategorySalesRanking {
    pub const QUERY: &'static str = r"SELECT
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
ORDER BY total_sales DESC";
    async fn query_many(
        &self,
        client: &impl tokio_postgres::GenericClient,
    ) -> Result<Vec<GetCategorySalesRankingRow>, tokio_postgres::Error> {
        let rows = client.query(Self::QUERY, &[]).await?;
        rows.into_iter()
            .map(|r| GetCategorySalesRankingRow::from_row(&r))
            .collect()
    }
}
struct DeleteUserAndRelatedDataRow {}
impl DeleteUserAndRelatedDataRow {
    fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        Ok(Self {})
    }
}
struct DeleteUserAndRelatedData {
    users_id: uuid::Uuid,
}
impl DeleteUserAndRelatedData {
    pub const QUERY: &'static str = r"DELETE FROM users WHERE id = $1";
    async fn execute(
        &self,
        client: &impl tokio_postgres::GenericClient,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(Self::QUERY, &[&self.users_id]).await
    }
}
impl DeleteUserAndRelatedData {
    const fn builder() -> DeleteUserAndRelatedDataBuilder<'static, ((),)> {
        DeleteUserAndRelatedDataBuilder {
            fields: ((),),
            _phantom: std::marker::PhantomData,
        }
    }
}
struct DeleteUserAndRelatedDataBuilder<'a, Fields = ((),)> {
    fields: Fields,
    _phantom: std::marker::PhantomData<&'a ()>,
}
impl<'a> DeleteUserAndRelatedDataBuilder<'a, (uuid::Uuid,)> {
    const fn build(self) -> DeleteUserAndRelatedData {
        let (users_id,) = self.fields;
        DeleteUserAndRelatedData { users_id }
    }
}
