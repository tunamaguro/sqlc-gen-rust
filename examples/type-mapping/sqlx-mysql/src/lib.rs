#[allow(warnings)]
mod queries;

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr as _;
    use test_context::test_context;
    use test_utils::SqlxMysqlContext;

    async fn migrate_db(pool: &sqlx::MySqlPool) {
        sqlx::raw_sql(include_str!("../schema.sql"))
            .execute(pool)
            .await
            .unwrap();
    }

    #[test_context(SqlxMysqlContext)]
    #[tokio::test]
    async fn test_mapping(ctx: &mut SqlxMysqlContext) {
        let pool = &ctx.pool;
        migrate_db(pool).await;

        let blob_val = vec![1, 2, 3, 4, 5];
        let datetime_val = chrono::NaiveDateTime::from_str("2025-01-23T04:05:06").unwrap();
        let date_val = chrono::NaiveDate::from_ymd_opt(2025, 1, 23).unwrap();
        let time_val = chrono::NaiveTime::from_hms_opt(1, 23, 45).unwrap();
        let json_val = serde_json::json!({ "type": "json" });

        let q = queries::InsertMapping::builder()
            .bool_val(true)
            .tinyint_val(1)
            .smallint_val(2)
            .int_val(3)
            .int_nullable_val(Some(4))
            .bigint_val(5)
            .float_val(6.0)
            .double_val(7.0)
            .text_val("8")
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
