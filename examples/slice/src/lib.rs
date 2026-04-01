#[allow(warnings)]
mod mysql_query;

#[cfg(test)]
mod tests {
    use super::*;
    use test_context::test_context;
    use test_utils::SqlxMysqlContext;

    async fn migrate_db(pool: &sqlx::MySqlPool) {
        sqlx::raw_sql(include_str!("../mysql-schema.sql"))
            .execute(pool)
            .await
            .unwrap();
    }

    async fn seed_authors(pool: &sqlx::MySqlPool) {
        mysql_query::CreateAuthor::builder()
            .name("Alice")
            .bio(Some("Alice's bio"))
            .build()
            .execute(pool)
            .await
            .unwrap();

        mysql_query::CreateAuthor::builder()
            .name("Bob")
            .bio(None)
            .build()
            .execute(pool)
            .await
            .unwrap();

        mysql_query::CreateAuthor::builder()
            .name("Charlie")
            .bio(Some("Charlie's bio"))
            .build()
            .execute(pool)
            .await
            .unwrap();
    }

    #[test_context(SqlxMysqlContext)]
    #[tokio::test]
    async fn test_get_authors_by_ids(ctx: &mut SqlxMysqlContext) {
        let pool = &ctx.pool;
        migrate_db(pool).await;
        seed_authors(pool).await;

        let authors = mysql_query::GetAuthorsByIds::builder()
            .ids(&[1, 3])
            .build()
            .query_many(pool)
            .await
            .unwrap();

        assert_eq!(authors.len(), 2);
        assert_eq!(authors[0].name, "Alice");
        assert_eq!(authors[1].name, "Charlie");
    }

    #[test_context(SqlxMysqlContext)]
    #[tokio::test]
    async fn test_get_authors_by_ids_single(ctx: &mut SqlxMysqlContext) {
        let pool = &ctx.pool;
        migrate_db(pool).await;
        seed_authors(pool).await;

        let authors = mysql_query::GetAuthorsByIds::builder()
            .ids(&[2])
            .build()
            .query_many(pool)
            .await
            .unwrap();

        assert_eq!(authors.len(), 1);
        assert_eq!(authors[0].name, "Bob");
        assert!(authors[0].bio.is_none());
    }

    #[test_context(SqlxMysqlContext)]
    #[tokio::test]
    async fn test_get_authors_by_ids_empty(ctx: &mut SqlxMysqlContext) {
        let pool = &ctx.pool;
        migrate_db(pool).await;
        seed_authors(pool).await;

        let authors = mysql_query::GetAuthorsByIds::builder()
            .ids(&[])
            .build()
            .query_many(pool)
            .await
            .unwrap();

        assert_eq!(authors.len(), 0);
    }

    #[test_context(SqlxMysqlContext)]
    #[tokio::test]
    async fn test_get_authors_by_ids_and_name(ctx: &mut SqlxMysqlContext) {
        let pool = &ctx.pool;
        migrate_db(pool).await;
        seed_authors(pool).await;

        let authors = mysql_query::GetAuthorsByIdsAndName::builder()
            .name("Alice")
            .ids(&[1, 2, 3])
            .build()
            .query_many(pool)
            .await
            .unwrap();

        assert_eq!(authors.len(), 1);
        assert_eq!(authors[0].name, "Alice");
        assert_eq!(authors[0].bio, Some("Alice's bio".to_string()));
    }

    #[test_context(SqlxMysqlContext)]
    #[tokio::test]
    async fn test_get_authors_by_ids_and_name_no_match(ctx: &mut SqlxMysqlContext) {
        let pool = &ctx.pool;
        migrate_db(pool).await;
        seed_authors(pool).await;

        let authors = mysql_query::GetAuthorsByIdsAndName::builder()
            .name("Alice")
            .ids(&[2, 3])
            .build()
            .query_many(pool)
            .await
            .unwrap();

        assert_eq!(authors.len(), 0);
    }
}
