#[allow(warnings)]
mod queries;

#[derive(Debug, Clone, Copy, sqlx::Type)]
#[sqlx(type_name = "complex")]
struct Complex {
    r: f64,
    i: f64,
}

#[cfg(test)]
mod tests {

    use std::{
        net::{IpAddr, Ipv4Addr},
        str::FromStr as _,
    };

    use super::*;
    use crate::queries;
    use chrono::TimeZone;
    use sqlx::postgres::types::*;
    use test_context::test_context;
    use test_utils::SqlxPgContext;

    async fn migrate_db(pool: &sqlx::PgPool) {
        sqlx::raw_sql(include_str!("../schema.sql"))
            .execute(pool)
            .await
            .unwrap();
    }

    #[test_context(SqlxPgContext)]
    #[tokio::test]
    async fn test_mapping(ctx: &mut SqlxPgContext) {
        let pool = &ctx.pool;
        migrate_db(pool).await;

        let bool_array_val = vec![true, false];
        let bytea_val = vec![1, 2, 3, 4, 5];
        let mut hstore_val = PgHstore::default();
        hstore_val.insert("type".to_string(), Some("hstore".to_string()));

        let timestamp_val = chrono::NaiveDateTime::from_str("2025-01-23T04:05:06").unwrap();
        let timestamptz_val = chrono::Utc.with_ymd_and_hms(2025, 1, 23, 4, 5, 6).unwrap();
        let date_val = chrono::NaiveDate::from_ymd_opt(2025, 1, 23).unwrap();
        let time_val = chrono::NaiveTime::from_hms_opt(1, 23, 45).unwrap();
        let inet_val = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let json_val = serde_json::json!({"type":  "json"});
        let jsonb_val = serde_json::json!({"type": "jsonb"});

        let money_val = PgMoney(12345);
        let ltree_val = PgLTree::new();
        let lquery_val = PgLQuery::from(vec![PgLQueryLevel::Star(None, None)]);
        let cube_val = PgCube::Point(12.34);
        let point_val = PgPoint { x: 12.34, y: 56.78 };
        let line_val = PgLine {
            a: 1.0,
            b: 2.0,
            c: 0.0,
        };
        let lseg_val = PgLSeg {
            start_x: 1.0,
            start_y: 2.0,
            end_x: 3.0,
            end_y: 4.0,
        };
        let box_val = PgBox {
            upper_right_x: 1.0,
            upper_right_y: 2.0,
            lower_left_x: 3.0,
            lower_left_y: 4.0,
        };
        let path_val = PgPath {
            closed: true,
            points: vec![PgPoint { x: 0.0, y: 0.0 }, PgPoint { x: 1.0, y: 1.0 }],
        };
        let polygon_val = PgPolygon {
            points: vec![
                PgPoint { x: 0.0, y: 0.0 },
                PgPoint { x: 1.0, y: 1.0 },
                PgPoint { x: 2.0, y: 0.0 },
            ],
        };
        let circle_val = PgCircle {
            x: 1.0,
            y: 2.0,
            radius: 1.0,
        };

        let q = queries::InsertMapping::builder()
            .bool_val(true)
            .bool_array_val(&bool_array_val)
            .char_val(1)
            .smallint_val(2)
            .int_val(3)
            .int_nullable_val(Some(4))
            .oid_val(sqlx::postgres::types::Oid(5))
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
            .money_val(&money_val)
            .ltree_val(&ltree_val)
            .lquery_val(&lquery_val)
            .cube_val(&cube_val)
            .point_val(&point_val)
            .line_val(&line_val)
            .lseg_val(&lseg_val)
            .box_val(&box_val)
            .path_val(&path_val)
            .polygon_val(&polygon_val)
            .circle_val(&circle_val)
            .build();

        q.execute(pool).await.unwrap();

        let _row = queries::GetMapping.query_one(pool).await.unwrap();
    }
}
