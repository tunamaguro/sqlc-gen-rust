use test_context::{AsyncTestContext, TestContext};

pub struct PgTokioTestContext {
    db_name: String,
    pub client: tokio_postgres::Client,
}

pub struct PgSyncTestContext {
    db_name: String,
    pub client: postgres::Client,
}

pub struct DeadPoolContext {
    db_name: String,
    pub pool: deadpool_postgres::Pool,
}

pub struct SqlxPgContext {
    db_name: String,
    pub pool: sqlx::PgPool,
}

fn postgres_config() -> postgres::Config {
    let database_url = std::env::var("DATABASE_URL").unwrap();
    let postgres_url = url::Url::parse(&database_url).unwrap();
    let db_name = {
        let path = postgres_url.path().trim_start_matches('/');
        if path.is_empty() { "postgres" } else { path }
    };
    let host = postgres_url
        .host()
        .map(|h| h.to_string())
        .unwrap_or("localhost".into());
    let port = postgres_url.port().unwrap_or(5432);

    let user = postgres_url.username();
    let password = postgres_url.password().unwrap_or("");

    let mut config = postgres::Config::default();
    config.dbname(db_name);
    config.host(&host);
    config.port(port);
    config.user(user);
    config.password(password);

    config
}

fn tokio_postgres_config() -> tokio_postgres::Config {
    let database_url = std::env::var("DATABASE_URL").unwrap();
    let postgres_url = url::Url::parse(&database_url).unwrap();
    let db_name = {
        let path = postgres_url.path().trim_start_matches('/');
        if path.is_empty() { "postgres" } else { path }
    };
    let host = postgres_url
        .host()
        .map(|h| h.to_string())
        .unwrap_or("localhost".into());
    let port = postgres_url.port().unwrap_or(5432);

    let user = postgres_url.username();
    let password = postgres_url.password().unwrap_or("");

    let mut config = tokio_postgres::Config::default();
    config.dbname(db_name);
    config.host(&host);
    config.port(port);
    config.user(user);
    config.password(password);

    config
}

fn generate_tmp_db() -> String {
    let db_rand = std::iter::repeat_with(fastrand::alphanumeric)
        .take(10)
        .collect::<String>();
    format!("test_db_{db_rand}").to_lowercase()
}

impl TestContext for PgSyncTestContext {
    fn setup() -> Self {
        let mut admin_client = postgres_config().connect(postgres::NoTls).unwrap();

        let test_db_name = generate_tmp_db();

        admin_client
            .batch_execute(&format!("CREATE DATABASE {test_db_name}"))
            .unwrap();

        let mut config = postgres_config();
        let config = config.dbname(&test_db_name);

        let client = config.connect(postgres::NoTls).unwrap();
        Self {
            client,
            db_name: test_db_name,
        }
    }
    fn teardown(self) {
        drop(self.client);

        let mut admin_client = postgres_config().connect(postgres::NoTls).unwrap();

        admin_client
            .batch_execute(&format!("DROP DATABASE {}", self.db_name))
            .unwrap();
    }
}

impl AsyncTestContext for PgTokioTestContext {
    async fn setup() -> Self {
        let (admin_client, admin_conn) = tokio_postgres_config()
            .connect(tokio_postgres::NoTls)
            .await
            .unwrap();
        tokio::spawn(async move {
            if let Err(e) = admin_conn.await {
                panic!("connection error: {e}");
            }
        });

        let test_db_name = generate_tmp_db();

        let stmt = format!("CREATE DATABASE {test_db_name};");
        admin_client.batch_execute(&stmt).await.unwrap();

        let mut config = tokio_postgres_config();
        let config = config.dbname(test_db_name.clone());

        let (client, conn) = config.connect(tokio_postgres::NoTls).await.unwrap();
        tokio::spawn(async move {
            if let Err(e) = conn.await {
                panic!("connection error: {e}");
            }
        });

        Self {
            client,
            db_name: test_db_name,
        }
    }
    async fn teardown(self) {
        drop(self.client);

        let (admin_client, admin_conn) = tokio_postgres_config()
            .connect(tokio_postgres::NoTls)
            .await
            .unwrap();

        tokio::spawn(async move {
            if let Err(e) = admin_conn.await {
                panic!("connection error: {e}");
            }
        });
        let stmt = format!("DROP DATABASE {};", self.db_name);
        admin_client.batch_execute(&stmt).await.unwrap();
    }
}

impl AsyncTestContext for DeadPoolContext {
    async fn setup() -> Self {
        let (admin_client, admin_conn) = tokio_postgres_config()
            .connect(tokio_postgres::NoTls)
            .await
            .unwrap();
        tokio::spawn(async move {
            if let Err(e) = admin_conn.await {
                panic!("connection error: {e}");
            }
        });

        let test_db_name = generate_tmp_db();

        let stmt = format!("CREATE DATABASE {test_db_name};");
        admin_client.batch_execute(&stmt).await.unwrap();

        let mut pg_config = tokio_postgres_config();
        let pg_config = pg_config.dbname(test_db_name.clone());

        let mgr_config = {
            deadpool_postgres::ManagerConfig {
                recycling_method: deadpool_postgres::RecyclingMethod::Verified,
            }
        };
        let mgr = deadpool_postgres::Manager::from_config(
            pg_config.to_owned(),
            tokio_postgres::NoTls,
            mgr_config,
        );
        let pool = deadpool_postgres::Pool::builder(mgr)
            .max_size(4)
            .build()
            .unwrap();

        Self {
            pool,
            db_name: test_db_name,
        }
    }
    async fn teardown(self) {
        drop(self.pool);

        let (admin_client, admin_conn) = tokio_postgres_config()
            .connect(tokio_postgres::NoTls)
            .await
            .unwrap();
        tokio::spawn(async move {
            if let Err(e) = admin_conn.await {
                panic!("connection error: {e}");
            }
        });
        let stmt = format!("DROP DATABASE {};", self.db_name);
        admin_client.batch_execute(&stmt).await.unwrap();
    }
}

impl AsyncTestContext for SqlxPgContext {
    async fn setup() -> Self {
        let (admin_client, admin_conn) = tokio_postgres_config()
            .connect(tokio_postgres::NoTls)
            .await
            .unwrap();
        tokio::spawn(async move {
            if let Err(e) = admin_conn.await {
                panic!("connection error: {e}");
            }
        });

        let test_db_name = generate_tmp_db();

        let stmt = format!("CREATE DATABASE {test_db_name};");
        admin_client.batch_execute(&stmt).await.unwrap();

        let database_url = std::env::var("DATABASE_URL").unwrap();
        let mut postgres_url = url::Url::parse(&database_url).unwrap();
        postgres_url.set_path(&format!("/{test_db_name}"));

        let pool = sqlx::PgPool::connect(postgres_url.as_str()).await.unwrap();
        Self {
            pool,
            db_name: test_db_name,
        }
    }

    async fn teardown(self) {
        drop(self.pool);

        let (admin_client, admin_conn) = tokio_postgres_config()
            .connect(tokio_postgres::NoTls)
            .await
            .unwrap();
        tokio::spawn(async move {
            if let Err(e) = admin_conn.await {
                panic!("connection error: {e}");
            }
        });
        let stmt = format!("DROP DATABASE {};", self.db_name);
        admin_client.batch_execute(&stmt).await.unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_context::test_context;

    #[test_context(PgSyncTestContext)]
    fn test_pg_sync(ctx: &mut PgSyncTestContext) {
        let client = &mut ctx.client;

        let row = client
            .query_one("SELECT $1::INT as int_val", &[&123])
            .unwrap();

        let int_val: i32 = row.try_get("int_val").unwrap();
        assert_eq!(int_val, 123);
    }

    #[test_context(PgTokioTestContext)]
    #[tokio::test]
    async fn test_pg_tokio(ctx: &mut PgTokioTestContext) {
        let client = &ctx.client;

        let row = client
            .query_one("SELECT $1::INT as int_val", &[&123])
            .await
            .unwrap();

        let int_val: i32 = row.try_get("int_val").unwrap();
        assert_eq!(int_val, 123);
    }

    #[test_context(DeadPoolContext)]
    #[tokio::test]
    async fn test_deadpool(ctx: &mut DeadPoolContext) {
        let client = ctx.pool.get().await.unwrap();

        let row = client
            .query_one("SELECT $1::INT as int_val", &[&123])
            .await
            .unwrap();

        let int_val: i32 = row.try_get("int_val").unwrap();
        assert_eq!(int_val, 123);
    }

    #[test_context(SqlxPgContext)]
    #[tokio::test]
    async fn test_sqlx_postgres(ctx: &mut SqlxPgContext) {
        use sqlx::Row as _;
        let pool = &ctx.pool;

        let row = sqlx::query("SELECT $1::INT as int_val")
            .bind(123)
            .fetch_one(pool)
            .await
            .unwrap();

        let int_val: i32 = row.try_get("int_val").unwrap();
        assert_eq!(int_val, 123);
    }
}
