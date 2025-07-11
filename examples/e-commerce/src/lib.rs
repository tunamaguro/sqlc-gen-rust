#[allow(warnings)]
mod deadpool_query;
#[allow(warnings)]
mod postgres_query;
#[allow(warnings)]
mod sqlx_query;
#[allow(warnings)]
mod tokio_query;

#[cfg(test)]
mod tests {
    use super::*;
    use test_context::test_context;
    use test_utils::*;

    fn migrate_db_postgres(client: &mut impl postgres::GenericClient) {
        client.batch_execute(include_str!("../schema.sql")).unwrap();
    }

    #[test_context(PgSyncTestContext)]
    #[test]
    fn test_postgres(ctx: &mut PgSyncTestContext) {
        let client = &mut ctx.client;

        migrate_db_postgres(client);

        let username = "test_user";
        let email = "test@example.com";

        let user = postgres_query::CreateUser::builder()
            .username(username)
            .email(email)
            .hashed_password("password123")
            .full_name(Some("Test User"))
            .build()
            .query_one(client)
            .unwrap();

        assert_eq!(user.username, username);
        assert_eq!(user.email, email);
    }

    async fn migrate_db_tokio(client: &impl tokio_postgres::GenericClient) {
        client
            .batch_execute(include_str!("../schema.sql"))
            .await
            .unwrap();
    }

    #[test_context(PgTokioTestContext)]
    #[tokio::test]
    async fn test_tokio_postgres(ctx: &mut PgTokioTestContext) {
        let client = &ctx.client;

        migrate_db_tokio(client).await;

        let username = "test_user";
        let email = "test@example.com";

        let user = tokio_query::CreateUser::builder()
            .username(username)
            .email(email)
            .hashed_password("password123")
            .full_name(Some("Test User"))
            .build()
            .query_one(client)
            .await
            .unwrap();

        assert_eq!(user.username, username);
        assert_eq!(user.email, email);
    }

    async fn migrate_db_deadpool(client: &impl deadpool_postgres::GenericClient) {
        client
            .batch_execute(include_str!("../schema.sql"))
            .await
            .unwrap();
    }

    #[test_context(DeadPoolContext)]
    #[tokio::test]
    async fn test_deadpool_postgres(ctx: &mut DeadPoolContext) {
        let client = ctx.pool.get().await.unwrap();

        migrate_db_deadpool(&client).await;

        let username = "test_user";
        let email = "test@example.com";

        let user = deadpool_query::CreateUser::builder()
            .username(username)
            .email(email)
            .hashed_password("password123")
            .full_name(Some("Test User"))
            .build()
            .query_one(&client)
            .await
            .unwrap();

        assert_eq!(user.username, username);
        assert_eq!(user.email, email);
    }

    #[test_context(SqlxPgContext)]
    #[tokio::test]
    async fn test_sqlx_postgres(ctx: &mut SqlxPgContext) {
        sqlx::raw_sql(include_str!("../schema.sql"))
            .execute(&ctx.pool)
            .await
            .unwrap();

        let username = "test_user";
        let email = "test@example.com";

        let user = sqlx_query::CreateUser::builder()
            .username(username)
            .email(email)
            .hashed_password("password123")
            .full_name(Some("Test User"))
            .build()
            .query_one(&ctx.pool)
            .await
            .unwrap();

        assert_eq!(user.username, username);
        assert_eq!(user.email, email);

        use futures::TryStreamExt;

        let user_query = sqlx_query::ListUsers::builder()
            .limit(100)
            .offset(0)
            .build();

        let mut user_stream = user_query.query_as().fetch(&ctx.pool);

        while let Some(_user) = user_stream.try_next().await.unwrap() {}
    }
}
