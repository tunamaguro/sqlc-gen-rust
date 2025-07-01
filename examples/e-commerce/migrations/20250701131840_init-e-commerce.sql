-- pgcrypto拡張を有効化 (UUID生成のため)
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- 注文ステータス用のENUM型を定義
CREATE TYPE order_status AS ENUM (
    'pending',
    'processing',
    'shipped',
    'delivered',
    'cancelled'
);

-- ユーザーテーブル
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(255) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    hashed_password TEXT NOT NULL,
    full_name VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 商品カテゴリテーブル (自己参照で階層構造を表現)
CREATE TABLE categories (
    id SERIAL PRIMARY KEY,
    parent_id INTEGER REFERENCES categories(id) ON DELETE SET NULL,
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(255) UNIQUE NOT NULL
);

-- 商品テーブル
CREATE TABLE products (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    category_id INTEGER NOT NULL REFERENCES categories(id) ON DELETE RESTRICT,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    price INT NOT NULL CHECK (price >= 0),
    stock_quantity INT NOT NULL DEFAULT 0 CHECK (stock_quantity >= 0),
    -- 商品の仕様などを格納するJSONBカラム
    attributes JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 注文テーブル
CREATE TABLE orders (
    id BIGSERIAL PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    status order_status NOT NULL DEFAULT 'pending',
    total_amount INT NOT NULL CHECK (total_amount >= 0),
    ordered_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 注文商品テーブル (ordersとproductsの多対多を表現)
CREATE TABLE order_items (
    id BIGSERIAL PRIMARY KEY,
    order_id BIGINT NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
    product_id UUID NOT NULL REFERENCES products(id) ON DELETE RESTRICT,
    quantity INT NOT NULL CHECK (quantity > 0),
    -- 購入時点での商品価格を記録
    price_at_purchase INT NOT NULL CHECK (price_at_purchase >= 0),
    UNIQUE (order_id, product_id)
);

-- 商品レビューテーブル
CREATE TABLE reviews (
    id BIGSERIAL PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    product_id UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    rating INT NOT NULL CHECK (rating >= 1 AND rating <= 5),
    comment TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, product_id)
);

-- インデックスの作成
CREATE INDEX ON users (email);
CREATE INDEX ON categories (parent_id);
CREATE INDEX ON products (category_id);
CREATE INDEX ON products (name);
CREATE INDEX ON orders (user_id);
CREATE INDEX ON orders (status);
CREATE INDEX ON order_items (order_id);
CREATE INDEX ON order_items (product_id);
CREATE INDEX ON reviews (product_id);
-- JSONBカラム内の特定のキーに対するGINインデックス
CREATE INDEX ON products USING GIN (attributes);