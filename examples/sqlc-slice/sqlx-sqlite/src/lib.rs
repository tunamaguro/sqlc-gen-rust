#[allow(warnings)]
mod queries;

#[cfg(test)]
mod tests {
    use super::*;
    use test_context::test_context;
    use test_utils::SqlxSqliteContext;

    async fn migrate_db(pool: &sqlx::SqlitePool) {
        sqlx::raw_sql(include_str!("../schema.sql"))
            .execute(pool)
            .await
            .unwrap();
    }

    async fn seed_authors(pool: &sqlx::SqlitePool) {
        sqlx::query("INSERT INTO authors (id, name) VALUES (?, ?), (?, ?), (?, ?)")
            .bind(1i64)
            .bind("Alice")
            .bind(2i64)
            .bind("Bob")
            .bind(3i64)
            .bind("Charlie")
            .execute(pool)
            .await
            .unwrap();
    }

    #[test_context(SqlxSqliteContext)]
    #[tokio::test]
    async fn test_list_authors_by_ids(ctx: &mut SqlxSqliteContext) {
        let pool = &ctx.pool;
        migrate_db(pool).await;
        seed_authors(pool).await;

        // 0 ids → NULL branch
        let authors = queries::ListAuthorsByIDs::builder()
            .ids(&[])
            .build()
            .query_many(pool)
            .await
            .unwrap();
        assert_eq!(authors.len(), 0);

        // 1 id → single placeholder branch
        let ids = [2i64];
        let authors = queries::ListAuthorsByIDs::builder()
            .ids(&ids)
            .build()
            .query_many(pool)
            .await
            .unwrap();
        assert_eq!(authors.len(), 1);
        assert_eq!(authors[0].id, 2);

        // 2+ ids → repeated placeholder branch
        let ids = [1i64, 3i64];
        let authors = queries::ListAuthorsByIDs::builder()
            .ids(&ids)
            .build()
            .query_many(pool)
            .await
            .unwrap();
        assert_eq!(authors.len(), 2);
        assert_eq!(authors[0].id, 1);
        assert_eq!(authors[1].id, 3);
    }

    #[test_context(SqlxSqliteContext)]
    #[tokio::test]
    async fn test_list_authors_by_two_id_lists(ctx: &mut SqlxSqliteContext) {
        let pool = &ctx.pool;
        migrate_db(pool).await;
        seed_authors(pool).await;

        // both empty → NULL branch for both slices
        let authors = queries::ListAuthorsByTwoIdLists::builder()
            .ids(&[])
            .backup_ids(&[])
            .build()
            .query_many(pool)
            .await
            .unwrap();
        assert_eq!(authors.len(), 0);

        // 1 id in ids, backup empty
        let ids = [1i64];
        let authors = queries::ListAuthorsByTwoIdLists::builder()
            .ids(&ids)
            .backup_ids(&[])
            .build()
            .query_many(pool)
            .await
            .unwrap();
        assert_eq!(authors.len(), 1);
        assert_eq!(authors[0].id, 1);

        // 2+ ids across both slices
        let ids = [1i64, 2i64];
        let backup_ids = [3i64];
        let authors = queries::ListAuthorsByTwoIdLists::builder()
            .ids(&ids)
            .backup_ids(&backup_ids)
            .build()
            .query_many(pool)
            .await
            .unwrap();
        assert_eq!(authors.len(), 3);
    }

    #[test_context(SqlxSqliteContext)]
    #[tokio::test]
    async fn test_list_authors_by_ids_mixed(ctx: &mut SqlxSqliteContext) {
        let pool = &ctx.pool;
        migrate_db(pool).await;
        seed_authors(pool).await;

        // 0 ids → NULL branch
        let authors = queries::ListAuthorsByIDsMixed::builder()
            .ids(&[])
            .id(1)
            .skip_ids(&[])
            .name("X")
            .build()
            .query_many(pool)
            .await
            .unwrap();
        assert_eq!(authors.len(), 0);

        // 1 id, 1 skip_id
        let ids = [1i64];
        let skip_ids = [2i64];
        let authors = queries::ListAuthorsByIDsMixed::builder()
            .ids(&ids)
            .id(1)
            .skip_ids(&skip_ids)
            .name("X")
            .build()
            .query_many(pool)
            .await
            .unwrap();
        assert_eq!(authors.len(), 1);
        assert_eq!(authors[0].id, 1);

        // 2+ ids with skip and name exclusion
        let ids = [1i64, 2i64, 3i64];
        let skip_ids = [2i64];
        let authors = queries::ListAuthorsByIDsMixed::builder()
            .ids(&ids)
            .id(1)
            .skip_ids(&skip_ids)
            .name("Alice")
            .build()
            .query_many(pool)
            .await
            .unwrap();
        assert_eq!(authors.len(), 1);
        assert_eq!(authors[0].id, 3);
        assert_eq!(authors[0].name, "Charlie");
    }
}
