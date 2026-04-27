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

    fn seed_authors(conn: &rusqlite::Connection) {
        conn.execute(
            "INSERT INTO authors (id, name) VALUES (?, ?), (?, ?), (?, ?)",
            rusqlite::params![1i64, "Alice", 2i64, "Bob", 3i64, "Charlie"],
        )
        .unwrap();
    }

    #[test_context(RusqliteContext)]
    #[test]
    fn test_list_authors_by_ids(ctx: &mut RusqliteContext) {
        let conn = &ctx.conn;
        migrate_db(conn);
        seed_authors(conn);

        let authors = queries::ListAuthorsByIDs::builder()
            .ids(&[])
            .build()
            .query_many(conn)
            .unwrap();
        assert_eq!(authors.len(), 0);

        let ids = [2i64];
        let authors = queries::ListAuthorsByIDs::builder()
            .ids(&ids)
            .build()
            .query_many(conn)
            .unwrap();
        assert_eq!(authors.len(), 1);
        assert_eq!(authors[0].id, 2);

        let ids = [1i64, 3i64];
        let authors = queries::ListAuthorsByIDs::builder()
            .ids(&ids)
            .build()
            .query_many(conn)
            .unwrap();
        assert_eq!(authors.len(), 2);
        assert_eq!(authors[0].id, 1);
        assert_eq!(authors[1].id, 3);
    }

    #[test_context(RusqliteContext)]
    #[test]
    fn test_list_authors_by_two_id_lists(ctx: &mut RusqliteContext) {
        let conn = &ctx.conn;
        migrate_db(conn);
        seed_authors(conn);

        let authors = queries::ListAuthorsByTwoIdLists::builder()
            .ids(&[])
            .backup_ids(&[])
            .build()
            .query_many(conn)
            .unwrap();
        assert_eq!(authors.len(), 0);

        let ids = [1i64];
        let authors = queries::ListAuthorsByTwoIdLists::builder()
            .ids(&ids)
            .backup_ids(&[])
            .build()
            .query_many(conn)
            .unwrap();
        assert_eq!(authors.len(), 1);
        assert_eq!(authors[0].id, 1);

        let ids = [1i64, 2i64];
        let backup_ids = [3i64];
        let authors = queries::ListAuthorsByTwoIdLists::builder()
            .ids(&ids)
            .backup_ids(&backup_ids)
            .build()
            .query_many(conn)
            .unwrap();
        assert_eq!(authors.len(), 3);
    }

    #[test_context(RusqliteContext)]
    #[test]
    fn test_list_authors_by_ids_mixed(ctx: &mut RusqliteContext) {
        let conn = &ctx.conn;
        migrate_db(conn);
        seed_authors(conn);

        let authors = queries::ListAuthorsByIDsMixed::builder()
            .ids(&[])
            .id(1)
            .skip_ids(&[])
            .name("X")
            .build()
            .query_many(conn)
            .unwrap();
        assert_eq!(authors.len(), 0);

        let ids = [1i64];
        let skip_ids = [2i64];
        let authors = queries::ListAuthorsByIDsMixed::builder()
            .ids(&ids)
            .id(1)
            .skip_ids(&skip_ids)
            .name("X")
            .build()
            .query_many(conn)
            .unwrap();
        assert_eq!(authors.len(), 1);
        assert_eq!(authors[0].id, 1);

        let ids = [1i64, 2i64, 3i64];
        let skip_ids = [2i64];
        let authors = queries::ListAuthorsByIDsMixed::builder()
            .ids(&ids)
            .id(1)
            .skip_ids(&skip_ids)
            .name("Alice")
            .build()
            .query_many(conn)
            .unwrap();
        assert_eq!(authors.len(), 1);
        assert_eq!(authors[0].id, 3);
        assert_eq!(authors[0].name, "Charlie");
    }
}
