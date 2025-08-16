#[allow(warnings)]
mod queries;

#[derive(Debug, Clone, Copy, postgres_types::ToSql, postgres_types::FromSql)]
#[postgres(name = "complex")]
struct Complex {
    r: f64,
    i: f64,
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        net::{IpAddr, Ipv4Addr},
        time::SystemTime,
    };

    use super::*;
    use test_context::test_context;
    use test_utils::PgTokioTestContext;

    async fn migrate_db(client: &tokio_postgres::Client) {
        client
            .batch_execute(include_str!("../schema.sql"))
            .await
            .unwrap();
    }

    #[test_context(PgTokioTestContext)]
    #[tokio::test]
    async fn test_mapping(ctx: &mut PgTokioTestContext) {
        let client = &ctx.client;
        migrate_db(client).await;

        let bool_array_val = vec![true, false];
        let bytea_val = vec![1, 2, 3, 4, 5];
        let hstore_val: HashMap<String, Option<String>> =
            HashMap::from_iter([("type".to_string(), Some("hstore".to_string()))]);

        let timestamp_val = SystemTime::now();
        let timestamptz_val = SystemTime::now();
        let date_val = chrono::NaiveDate::from_ymd_opt(2025, 1, 23).unwrap();
        let time_val = chrono::NaiveTime::from_hms_opt(1, 23, 45).unwrap();
        let inet_val = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let json_val = serde_json::json! {{"type":  "json"}};
        let jsonb_val = serde_json::json! {{"type": "jsonb"}};

        let q = queries::InsertMapping::builder()
            .bool_val(true)
            .bool_array_val(&bool_array_val)
            .char_val(1)
            .smallint_val(2)
            .int_val(3)
            .int_nullable_val(Some(4))
            .oid_val(5)
            .bigint_val(6)
            .real_val(7.0)
            .double_val(8.0)
            .text_val("9")
            .text_nullable_val(Some("10"))
            .bytea_val(&bytea_val)
            .hstore_val(&hstore_val)
            .timestamp_val(&timestamp_val)
            .timestamptz_val(&timestamptz_val)
            .date_val(&date_val)
            .time_val(&time_val)
            .inet_val(&inet_val)
            .json_val(&json_val)
            .jsonb_val(&jsonb_val)
            .uuid_val("366dacaf-6812-4f94-8d20-25f5e7f4981c".parse().unwrap())
            .enum_val(queries::Mood::Sad)
            .composite_val(Complex { r: 12.3, i: 45.6 })
            .build();

        q.execute(client).await.unwrap();

        let _row = queries::GetMapping.query_one(client).await.unwrap();
    }
}
