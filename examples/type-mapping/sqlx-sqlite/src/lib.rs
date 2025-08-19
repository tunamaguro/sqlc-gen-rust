#[allow(warnings)]
mod queries;

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr as _;
    use test_context::test_context;
    use test_utils::SqlxSqliteContext;

    async fn migrate_db(pool: &sqlx::SqlitePool) {
        sqlx::raw_sql(include_str!("../schema.sql"))
            .execute(pool)
            .await
            .unwrap();
    }

    #[test_context(SqlxSqliteContext)]
    #[tokio::test]
    async fn test_mapping(ctx: &mut SqlxSqliteContext) {
        let pool = &ctx.pool;
        migrate_db(pool).await;

        let blob_val = vec![1, 2, 3, 4, 5];
        let datetime_val = chrono::NaiveDateTime::from_str("2025-01-23T04:05:06").unwrap();
        let date_val = chrono::NaiveDate::from_ymd_opt(2025, 1, 23).unwrap();
        let time_val = chrono::NaiveTime::from_hms_opt(1, 23, 45).unwrap();
        let json_val = serde_json::json!({ "type": "json" });

        let q = queries::InsertMapping::builder()
            .bool_val(true)
            .int_val(1)
            .int_nullable_val(Some(2))
            .real_val(3.4)
            .text_val("text")
            .blob_val(&blob_val)
            .datetime_val(&datetime_val)
            .date_val(&date_val)
            .time_val(&time_val)
            .json_val(&json_val)
            .build();

        q.execute(pool).await.unwrap();

        let _row = queries::GetMapping.query_one(pool).await.unwrap();
    }
}
