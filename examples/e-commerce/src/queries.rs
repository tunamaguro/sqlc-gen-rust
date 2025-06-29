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
struct CreateUser<'a> {
    users_username: std::borrow::Cow<'a, str>,
    users_email: std::borrow::Cow<'a, str>,
    users_hashed_password: std::borrow::Cow<'a, str>,
    users_full_name: Option<std::borrow::Cow<'a, str>>,
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
struct GetUserByEmail<'a> {
    users_email: std::borrow::Cow<'a, str>,
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
struct ListUsers<'a> {
    limit: std::borrow::Cow<'a, i32>,
    offset: std::borrow::Cow<'a, i32>,
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
struct CreateProduct<'a> {
    products_category_id: std::borrow::Cow<'a, i32>,
    products_name: std::borrow::Cow<'a, str>,
    products_description: Option<std::borrow::Cow<'a, str>>,
    products_price: std::borrow::Cow<'a, i32>,
    products_stock_quantity: std::borrow::Cow<'a, i32>,
    products_attributes: Option<std::borrow::Cow<'a, serde_json::Value>>,
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
struct GetProductWithCategory<'a> {
    products_id: std::borrow::Cow<'a, uuid::Uuid>,
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
struct SearchProducts<'a> {
    limit: std::borrow::Cow<'a, i32>,
    offset: std::borrow::Cow<'a, i32>,
    products_name: Option<std::borrow::Cow<'a, str>>,
    category_ids: std::borrow::Cow<'a, [i32]>,
    products_min_price: Option<std::borrow::Cow<'a, i32>>,
    products_max_price: Option<std::borrow::Cow<'a, i32>>,
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
struct GetProductsWithSpecificAttribute<'a> {
    param: std::borrow::Cow<'a, serde_json::Value>,
}
struct UpdateProductStockRow {}
impl UpdateProductStockRow {
    async fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        Ok(Self {})
    }
}
struct UpdateProductStock<'a> {
    products_id: std::borrow::Cow<'a, uuid::Uuid>,
    products_add_quantity: std::borrow::Cow<'a, i32>,
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
struct CreateOrder<'a> {
    orders_user_id: std::borrow::Cow<'a, uuid::Uuid>,
    orders_status: std::borrow::Cow<'a, OrderStatus>,
    orders_total_amount: std::borrow::Cow<'a, i32>,
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
struct CreateOrderItem<'a> {
    order_items_order_id: std::borrow::Cow<'a, i64>,
    order_items_product_id: std::borrow::Cow<'a, uuid::Uuid>,
    order_items_quantity: std::borrow::Cow<'a, i32>,
    order_items_price_at_purchase: std::borrow::Cow<'a, i32>,
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
struct GetOrderDetails<'a> {
    orders_id: std::borrow::Cow<'a, i64>,
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
struct ListOrderItemsByOrderID<'a> {
    order_items_order_id: std::borrow::Cow<'a, i64>,
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
struct CreateReview<'a> {
    reviews_user_id: std::borrow::Cow<'a, uuid::Uuid>,
    reviews_product_id: std::borrow::Cow<'a, uuid::Uuid>,
    reviews_rating: std::borrow::Cow<'a, i32>,
    reviews_comment: Option<std::borrow::Cow<'a, str>>,
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
struct GetProductAverageRating<'a> {
    reviews_product_id: std::borrow::Cow<'a, uuid::Uuid>,
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
struct GetCategorySalesRanking;
struct DeleteUserAndRelatedDataRow {}
impl DeleteUserAndRelatedDataRow {
    async fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        Ok(Self {})
    }
}
struct DeleteUserAndRelatedData<'a> {
    users_id: std::borrow::Cow<'a, uuid::Uuid>,
}
