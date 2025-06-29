#[derive(Debug, postgres_types::ToSql, postgres_types::FromSql)]
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
    async fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
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
    async fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
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
struct ListUsersRow {
    users_id: uuid::Uuid,
    users_username: String,
    users_email: String,
    users_full_name: Option<String>,
    users_created_at: std::time::SystemTime,
}
impl ListUsersRow {
    async fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        Ok(Self {
            users_id: row.try_get(0)?,
            users_username: row.try_get(1)?,
            users_email: row.try_get(2)?,
            users_full_name: row.try_get(3)?,
            users_created_at: row.try_get(4)?,
        })
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
    async fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
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
    async fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
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
    async fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
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
    async fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
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
struct UpdateProductStockRow {}
impl UpdateProductStockRow {
    async fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        Ok(Self {})
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
    async fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        Ok(Self {
            orders_id: row.try_get(0)?,
            orders_user_id: row.try_get(1)?,
            orders_status: row.try_get(2)?,
            orders_total_amount: row.try_get(3)?,
            orders_ordered_at: row.try_get(4)?,
        })
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
    async fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        Ok(Self {
            order_items_id: row.try_get(0)?,
            order_items_order_id: row.try_get(1)?,
            order_items_product_id: row.try_get(2)?,
            order_items_quantity: row.try_get(3)?,
            order_items_price_at_purchase: row.try_get(4)?,
        })
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
    async fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
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
struct ListOrderItemsByOrderIdRow {
    order_items_quantity: i32,
    order_items_price_at_purchase: i32,
    products_product_id: uuid::Uuid,
    products_product_name: String,
}
impl ListOrderItemsByOrderIdRow {
    async fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        Ok(Self {
            order_items_quantity: row.try_get(0)?,
            order_items_price_at_purchase: row.try_get(1)?,
            products_product_id: row.try_get(2)?,
            products_product_name: row.try_get(3)?,
        })
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
    async fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
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
struct GetProductAverageRatingRow {
    reviews_product_id: uuid::Uuid,
    average_rating: f64,
    review_count: i64,
}
impl GetProductAverageRatingRow {
    async fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        Ok(Self {
            reviews_product_id: row.try_get(0)?,
            average_rating: row.try_get(1)?,
            review_count: row.try_get(2)?,
        })
    }
}
struct GetCategorySalesRankingRow {
    categories_category_id: i32,
    categories_category_name: String,
    total_sales: i64,
    total_orders: i64,
}
impl GetCategorySalesRankingRow {
    async fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        Ok(Self {
            categories_category_id: row.try_get(0)?,
            categories_category_name: row.try_get(1)?,
            total_sales: row.try_get(2)?,
            total_orders: row.try_get(3)?,
        })
    }
}
struct DeleteUserAndRelatedDataRow {}
impl DeleteUserAndRelatedDataRow {
    async fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        Ok(Self {})
    }
}
