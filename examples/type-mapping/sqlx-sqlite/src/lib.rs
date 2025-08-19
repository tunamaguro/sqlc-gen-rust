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

        let q = queries::InsertMapping::builder()
            .aff_integer_val(1)
            .aff_real_val(2.0)
            .aff_text_val("3")
            .aff_blob_val(&blob_val)
            .int_val(5)
            .integer_val(6)
            .tinyint_val(7)
            .smallint_val(8)
            .mediumint_val(9)
            .bigint_val(10)
            .unsigned_big_int_val(11)
            .int_2_val(12)
            .int_8_val(13)
            .character_20_val("14")
            .varchar_255_val("15")
            .varying_char_255_val("16")
            .nchar_55_val("17")
            .native_char_70_val("18")
            .nvarchar_100_val("19")
            .text_val("20")
            .clob_val("21")
            .real_val(22.0)
            .double_val(23.0)
            .double_precision_val(24.0)
            .float_val(25.0)
            .numeric_val(26.1)
            .decimal_10_5_val(27.1)
            .boolean_val(true)
            .date_val(&date_val)
            .time_val(&time_val)
            .datetime_val(&datetime_val)
            .build();

        q.execute(pool).await.unwrap();

        let _row = queries::GetMapping.query_one(pool).await.unwrap();
    }
}
