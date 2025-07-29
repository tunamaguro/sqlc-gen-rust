#[allow(warnings)]
mod queries;

#[cfg(test)]
mod tests {
    use super::*;
    use test_context::test_context;
    use test_utils::SqlxPgContext;

    async fn migrate_db(pool: &sqlx::PgPool) {
        sqlx::raw_sql(include_str!("../../tokio-postgres/schema.sql"))
            .execute(pool)
            .await
            .unwrap();
    }

    /// port from https://github.com/sqlc-dev/sqlc/blob/v1.29.0/examples/authors/postgresql/db_test.go
    #[test_context(SqlxPgContext)]
    #[tokio::test]
    async fn test_authors(ctx: &mut SqlxPgContext) {
        let pool = &ctx.pool;
        migrate_db(pool).await;

        let authors = queries::ListAuthors.query_many(pool).await.unwrap();
        assert_eq!(authors.len(), 0);

        let inserted_author = queries::CreateAuthor::builder()
            .name("Brian Kernighan")
            .bio(Some(
                "Co-author of The C Programming Language and The Go Programming Language",
            ))
            .build()
            .query_one(pool)
            .await
            .unwrap();

        let _fetched_author = queries::GetAuthor::builder()
            .id(inserted_author.id)
            .build()
            .query_one(pool)
            .await
            .unwrap();
    }
}
