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
    use test_utils::{DeadPoolContext, PgSyncTestContext, PgTokioTestContext, SqlxPgContext};

    #[test_context(PgSyncTestContext)]
    #[test]
    fn test_postgres(ctx: &mut PgSyncTestContext) {
        use postgres::{binary_copy::BinaryCopyInWriter, types::Type};
        use postgres_query::{CreateAuthors, GetAuthor};
        let client = &mut ctx.client;

        client.batch_execute(include_str!("../schema.sql")).unwrap();

        let sink = client.copy_in(CreateAuthors::QUERY).unwrap();
        let mut writer = BinaryCopyInWriter::new(sink, &[Type::INT8, Type::TEXT, Type::TEXT]);

        let author1 = CreateAuthors::builder().id(0).name("Foo").bio(None).build();
        writer.write(&author1.as_slice()).unwrap();

        let author2 = CreateAuthors::builder()
            .id(1)
            .name("Bar")
            .bio(Some("Bar's bio"))
            .build();

        writer.write(&author2.as_slice()).unwrap();
        writer.finish().unwrap();

        let row = GetAuthor::builder()
            .id(0)
            .build()
            .query_one(client)
            .unwrap();

        assert_eq!(row.name, "Foo");
        assert!(row.bio.is_none());

        let row = GetAuthor::builder()
            .id(1)
            .build()
            .query_one(client)
            .unwrap();

        assert_eq!(row.name, "Bar");
        assert_eq!(row.bio, Some("Bar's bio".to_string()));
    }

    #[test_context(PgTokioTestContext)]
    #[tokio::test]
    async fn test_tokio_postgres(ctx: &mut PgTokioTestContext) {
        use tokio_postgres::{binary_copy::BinaryCopyInWriter, types::Type};
        use tokio_query::{CreateAuthors, GetAuthor};
        let client = &ctx.client;

        client
            .batch_execute(include_str!("../schema.sql"))
            .await
            .unwrap();

        let sink = client.copy_in(CreateAuthors::QUERY).await.unwrap();
        let writer = BinaryCopyInWriter::new(sink, &[Type::INT8, Type::TEXT, Type::TEXT]);
        tokio::pin!(writer);

        let author1 = CreateAuthors::builder().id(0).name("Foo").bio(None).build();
        writer.as_mut().write(&author1.as_slice()).await.unwrap();

        let author2 = CreateAuthors::builder()
            .id(1)
            .name("Bar")
            .bio(Some("Bar's bio"))
            .build();

        writer.as_mut().write(&author2.as_slice()).await.unwrap();
        writer.as_mut().finish().await.unwrap();

        let row = GetAuthor::builder()
            .id(0)
            .build()
            .query_one(client)
            .await
            .unwrap();

        assert_eq!(row.name, "Foo");
        assert!(row.bio.is_none());

        let row = GetAuthor::builder()
            .id(1)
            .build()
            .query_one(client)
            .await
            .unwrap();

        assert_eq!(row.name, "Bar");
        assert_eq!(row.bio, Some("Bar's bio".to_string()));
    }

    #[test_context(DeadPoolContext)]
    #[tokio::test]
    async fn test_deadpool_postgres(ctx: &mut DeadPoolContext) {
        use deadpool_postgres::tokio_postgres::{binary_copy::BinaryCopyInWriter, types::Type};
        use deadpool_query::{CreateAuthors, GetAuthor};
        let client = ctx.pool.get().await.unwrap();

        client
            .batch_execute(include_str!("../schema.sql"))
            .await
            .unwrap();

        let sink = client.copy_in(CreateAuthors::QUERY).await.unwrap();
        let writer = BinaryCopyInWriter::new(sink, &[Type::INT8, Type::TEXT, Type::TEXT]);
        tokio::pin!(writer);

        let author1 = CreateAuthors::builder().id(0).name("Foo").bio(None).build();
        writer.as_mut().write(&author1.as_slice()).await.unwrap();

        let author2 = CreateAuthors::builder()
            .id(1)
            .name("Bar")
            .bio(Some("Bar's bio"))
            .build();

        writer.as_mut().write(&author2.as_slice()).await.unwrap();
        writer.as_mut().finish().await.unwrap();

        let row = GetAuthor::builder()
            .id(0)
            .build()
            .query_one(&client)
            .await
            .unwrap();

        assert_eq!(row.name, "Foo");
        assert!(row.bio.is_none());

        let row = GetAuthor::builder()
            .id(1)
            .build()
            .query_one(&client)
            .await
            .unwrap();

        assert_eq!(row.name, "Bar");
        assert_eq!(row.bio, Some("Bar's bio".to_string()));
    }

    #[test_context(SqlxPgContext)]
    #[tokio::test]
    async fn test_sqlx_postgres(ctx: &mut SqlxPgContext) {
        use sqlx_query::{CreateAuthors, GetAuthor};
        let pool = &ctx.pool;

        sqlx::raw_sql(include_str!("../schema.sql"))
            .execute(pool)
            .await
            .unwrap();

        let mut sink = CreateAuthors::copy_in(pool).await.unwrap();
        let author1 = CreateAuthors::builder().id(0).name("Foo").bio(None).build();
        author1.write(&mut sink).await.unwrap();

        let author2 = CreateAuthors::builder()
            .id(1)
            .name("Bar")
            .bio(Some("Bar's bio"))
            .build();
        author2.write(&mut sink).await.unwrap();
        sink.finish().await.unwrap();

        let row = GetAuthor::builder()
            .id(0)
            .build()
            .query_one(pool)
            .await
            .unwrap();
        assert_eq!(row.name, "Foo");
        assert!(row.bio.is_none());

        let row = GetAuthor::builder()
            .id(1)
            .build()
            .query_one(pool)
            .await
            .unwrap();
        assert_eq!(row.name, "Bar");
        assert_eq!(row.bio, Some("Bar's bio".to_string()));
    }
}
