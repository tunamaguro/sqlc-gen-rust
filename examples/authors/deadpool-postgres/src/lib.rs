#[allow(warnings)]
mod queries;

#[cfg(test)]
mod tests {
    use super::*;
    use test_context::test_context;
    use test_utils::DeadPoolContext;

    async fn migrate_db(client: &deadpool_postgres::Client) {
        client
            .batch_execute(include_str!("../../tokio-postgres/schema.sql"))
            .await
            .unwrap();
    }

    /// port from https://github.com/sqlc-dev/sqlc/blob/v1.29.0/examples/authors/postgresql/db_test.go
    #[test_context(DeadPoolContext)]
    #[tokio::test]
    async fn test_authors(ctx: &mut DeadPoolContext) {
        let pool = &ctx.pool;
        let client = pool.get().await.unwrap();
        migrate_db(&client).await;

        let authors = queries::ListAuthors.query_many(&client).await.unwrap();
        assert_eq!(authors.len(), 0);

        let inserted_author = queries::CreateAuthor::builder()
            .name("Brian Kernighan")
            .bio(Some(
                "Co-author of The C Programming Language and The Go Programming Language",
            ))
            .build()
            .query_one(&client)
            .await
            .unwrap();

        let _fetched_author = queries::GetAuthor::builder()
            .id(inserted_author.id)
            .build()
            .query_one(&client)
            .await
            .unwrap();
    }
}
