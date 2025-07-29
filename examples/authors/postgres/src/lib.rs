#[allow(warnings)]
mod queries;

#[cfg(test)]
mod tests {
    use super::*;
    use test_context::test_context;
    use test_utils::PgSyncTestContext;

    // this function used but rust compiler generate dead_code warning
    #[allow(dead_code)]
    fn migrate_db(client: &mut impl postgres::GenericClient) {
        client
            .batch_execute(include_str!("../../tokio-postgres/schema.sql"))
            .unwrap();
    }

    /// port from https://github.com/sqlc-dev/sqlc/blob/v1.29.0/examples/authors/postgresql/db_test.go
    #[test_context(PgSyncTestContext)]
    fn test_authors(ctx: &mut PgSyncTestContext) {
        let client = &mut ctx.client;
        migrate_db(client);

        let authors = queries::ListAuthors.query_many(client).unwrap();
        assert_eq!(authors.len(), 0);

        let inserted_author = queries::CreateAuthor::builder()
            .name("Brian Kernighan")
            .bio(Some(
                "Co-author of The C Programming Language and The Go Programming Language",
            ))
            .build()
            .query_one(client)
            .unwrap();

        let _fetched_author = queries::GetAuthor::builder()
            .id(inserted_author.id)
            .build()
            .query_one(client)
            .unwrap();
    }
}
