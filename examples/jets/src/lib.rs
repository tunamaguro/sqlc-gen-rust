#[allow(warnings)]
mod queries;

#[cfg(test)]
mod tests {
    use super::queries::*;
    use test_context::test_context;
    use test_utils::DeadPoolContext;

    async fn migrate_db(client: &impl deadpool_postgres::GenericClient) {
        client
            .batch_execute(include_str!("../schema.sql"))
            .await
            .unwrap();
    }

    #[test_context(DeadPoolContext)]
    #[tokio::test]
    async fn test_jets(ctx: &mut DeadPoolContext) {
        let pool = &mut ctx.pool;
        let conn = pool.get().await.unwrap();
        migrate_db(&conn).await;

        // Insert test data
        conn.execute(
            "INSERT INTO pilots (id, name) VALUES ($1, $2), ($3, $4), ($5, $6), ($7, $8), ($9, $10), ($11, $12)",
            &[
                &1i32, &"Tom Cruise",
                &2i32, &"Val Kilmer",
                &3i32, &"Anthony Edwards",
                &4i32, &"Tom Skerritt",
                &5i32, &"Michael Ironside",
                &6i32, &"John Stockwell",
            ],
        )
        .await
        .unwrap();

        // Count pilots
        let count = CountPilots.query_one(&conn).await.unwrap();
        assert_eq!(count.count, 6);

        // List pilots
        let pilots = ListPilots.query_many(&conn).await.unwrap();
        assert_eq!(pilots.len(), 5);
        assert_eq!(pilots[0].pilots_name, "Tom Cruise");

        // Delete pilot
        let deleted = DeletePilot::builder()
            .pilots_id(3)
            .build()
            .execute(&conn)
            .await
            .unwrap();
        assert_eq!(deleted, 1);

        // Verify deletion
        let count_after = CountPilots.query_one(&conn).await.unwrap();
        assert_eq!(count_after.count, 5);

        // Delete non-existent pilot
        let deleted_none = DeletePilot::builder()
            .pilots_id(999)
            .build()
            .execute(&conn)
            .await
            .unwrap();
        assert_eq!(deleted_none, 0);
    }
}
