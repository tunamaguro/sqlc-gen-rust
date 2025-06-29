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
    users_full_name: String,
    users_created_at: std::time::SystemTime,
    users_updated_at: std::time::SystemTime,
}
struct GetUserByEmailRow {
    users_id: uuid::Uuid,
    users_username: String,
    users_email: String,
    users_hashed_password: String,
    users_full_name: String,
    users_created_at: std::time::SystemTime,
    users_updated_at: std::time::SystemTime,
}
struct ListUsersRow {
    users_id: uuid::Uuid,
    users_username: String,
    users_email: String,
    users_full_name: String,
    users_created_at: std::time::SystemTime,
}
struct CreateProductRow {
    products_id: uuid::Uuid,
    products_category_id: i32,
    products_name: String,
    products_description: String,
    products_price: i32,
    products_stock_quantity: i32,
    products_attributes: serde_json::Value,
    products_created_at: std::time::SystemTime,
    products_updated_at: std::time::SystemTime,
}
struct GetProductWithCategoryRow {
    products_id: uuid::Uuid,
    products_name: String,
    products_description: String,
    products_price: i32,
    products_stock_quantity: i32,
    products_attributes: serde_json::Value,
    products_created_at: std::time::SystemTime,
    categories_category_name: String,
    categories_category_slug: String,
}
struct SearchProductsRow {
    products_id: uuid::Uuid,
    products_category_id: i32,
    products_name: String,
    products_description: String,
    products_price: i32,
    products_stock_quantity: i32,
    products_attributes: serde_json::Value,
    products_created_at: std::time::SystemTime,
    products_updated_at: std::time::SystemTime,
    average_rating: f64,
}
struct GetProductsWithSpecificAttributeRow {
    products_id: uuid::Uuid,
    products_category_id: i32,
    products_name: String,
    products_description: String,
    products_price: i32,
    products_stock_quantity: i32,
    products_attributes: serde_json::Value,
    products_created_at: std::time::SystemTime,
    products_updated_at: std::time::SystemTime,
}
struct UpdateProductStockRow {}
struct CreateOrderRow {
    orders_id: i64,
    orders_user_id: uuid::Uuid,
    orders_status: OrderStatus,
    orders_total_amount: i32,
    orders_ordered_at: std::time::SystemTime,
}
struct CreateOrderItemRow {
    order_items_id: i64,
    order_items_order_id: i64,
    order_items_product_id: uuid::Uuid,
    order_items_quantity: i32,
    order_items_price_at_purchase: i32,
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
struct ListOrderItemsByOrderIdRow {
    order_items_quantity: i32,
    order_items_price_at_purchase: i32,
    products_product_id: uuid::Uuid,
    products_product_name: String,
}
struct CreateReviewRow {
    reviews_id: i64,
    reviews_user_id: uuid::Uuid,
    reviews_product_id: uuid::Uuid,
    reviews_rating: i32,
    reviews_comment: String,
    reviews_created_at: std::time::SystemTime,
}
struct GetProductAverageRatingRow {
    reviews_product_id: uuid::Uuid,
    average_rating: f64,
    review_count: i64,
}
struct GetCategorySalesRankingRow {
    categories_category_id: i32,
    categories_category_name: String,
    total_sales: i64,
    total_orders: i64,
}
struct DeleteUserAndRelatedDataRow {}
