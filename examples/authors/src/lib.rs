#[allow(warnings)]
mod queries;

#[cfg(test)]
mod tests {
    use super::*;
    use test_context::test_context;
    use test_utils::PgTokioTestContext;

    async fn migrate_db(clinet: &tokio_postgres::Client) {
        clinet
            .batch_execute(include_str!("../schema.sql"))
            .await
            .unwrap();
    }

    /// port from https://github.com/sqlc-dev/sqlc/blob/v1.29.0/examples/authors/postgresql/db_test.go
    #[test_context(PgTokioTestContext)]
    #[tokio::test]
    async fn test_authors(ctx: &mut PgTokioTestContext) {
        let client = &ctx.client;
        migrate_db(client).await;

        let authors = queries::ListAuthors.query_many(client).await.unwrap();
        assert_eq!(authors.len(), 0);

        let inserted_author = queries::CreateAuthor::builder()
            .authors_name("Brian Kernighan")
            .authors_bio(Some(
                "Co-author of The C Programming Language and The Go Programming Language",
            ))
            .build()
            .query_one(client)
            .await
            .unwrap();

        let _fetched_author = queries::GetAuthor::builder()
            .authors_id(inserted_author.authors_id)
            .build()
            .query_one(client)
            .await
            .unwrap();
    }
}
