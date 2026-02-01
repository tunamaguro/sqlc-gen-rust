#[allow(warnings)]
mod queries;

#[cfg(test)]
mod tests {
    use super::*;
    use test_context::test_context;
    use test_utils::RusqliteContext;

    fn migrate_db(conn: &rusqlite::Connection) {
        conn.execute_batch(include_str!("../../sqlx-sqlite/schema.sql"))
            .unwrap();
    }

    #[test_context(RusqliteContext)]
    #[test]
    fn test_authors(ctx: &mut RusqliteContext) {
        let conn = &ctx.conn;
        migrate_db(&conn);

        let authors = queries::ListAuthors.query_many(conn).unwrap();
        assert_eq!(authors.len(), 0);

        let affected_rows = queries::CreateAuthor::builder()
            .name("Brian Kernighan")
            .bio(Some(
                "Co-author of The C Programming Language and The Go Programming Language",
            ))
            .build()
            .execute(conn)
            .unwrap();

        assert_eq!(affected_rows, 1);

        let _fetched_author = queries::GetAuthor::builder()
            .id(conn.last_insert_rowid())
            .build()
            .query_one(conn)
            .unwrap();
    }
}
