# sqlc-gen-rust

sqlc-plugin for rust db crates.

## Usage

```json
{
    "version": "2",
    "plugins": [
        {
            "name": "sqlc-gen-rust",
            "wasm": {
                "url": "https://github.com/tunamaguro/sqlc-gen-rust/releases/download/v0.1.3/sqlc-gen-rust.wasm",
                "sha256": "305607928df31023864d2dda9c8a9145834bdb129c25d250791324ccb93e55f0"
            }
        }
    ],
    "sql": [
        {
            "schema": "schema.sql",
            "queries": "queries.sql",
            "engine": "postgresql",
            "codegen": [
                {
                    "plugin": "sqlc-gen-rust",
                    "out": "src/db"
                }
            ]
        }
    ]
}
```

## Supported crates

- [postgres](https://crates.io/crates/postgres)
- [tokio-postgres](https://crates.io/crates/tokio-postgres)
- [deadpool-postgres](https://crates.io/crates/deadpool-postgres)
- [sqlx-postgres](https://docs.rs/sqlx/latest/sqlx/postgres/index.html)

## Example

### Schema

```sql
-- schema.sql
CREATE TABLE authors (
    id   BIGSERIAL PRIMARY KEY,
    name text      NOT NULL,
    bio  text
);
```

### Query

```sql
-- queries.sql
-- name: GetAuthor :one
SELECT * FROM authors
WHERE id = $1 LIMIT 1;

-- name: CreateAuthor :one
INSERT INTO authors (name, bio)
VALUES ($1, $2)
RETURNING *;
```

### Generated code

```rust
pub struct GetAuthorRow {
    pub id: i64,
    pub name: String,
    pub bio: Option<String>,
}

pub struct GetAuthor {
    id: i64,
}

impl GetAuthor {
    pub const QUERY: &'static str = r"SELECT id, name, bio FROM authors
WHERE id = $1 LIMIT 1";
    
    pub async fn query_one(
        &self,
        client: &impl tokio_postgres::GenericClient,
    ) -> Result<GetAuthorRow, tokio_postgres::Error> {
        // ...
    }
}

// Builder API
let author = GetAuthor::builder()
    .id(1)
    .build()
    .query_one(&client)
    .await?;
```

See below to see other crates example.

- [`postgres` generated code](./examples/e-commerce/src/postgres_query.rs)
- [`tokio-postgres` generated code](./examples/e-commerce/src/tokio_query.rs)
- [`deadpool-postgres` generated code](./examples/e-commerce/src/deadpool_query.rs)
- [`sqlx-postgres` generated code](./examples/e-commerce/src/sqlx_query.rs)

## Options

> [!NOTE]
> This plugin supports json option only. yaml is not supported.

### `db_crate`

The crate of generated code. Default is `tokio-postgres`. Available values are below.

- `postgres` 
- `tokio-postgres`
- `deadpool-postgres`
- `sqlx-postgres`

### `overrides`

Add or override DB and Rust type mappings. See example blow.

```json
{
    "sql": [
        {
            "codegen": [
                {
                    "plugin": "sqlc-gen-rust",
                    "out": "examples/e-commerce/src",
                    "options": {
                        "overrides": [
                            {
                                "db_type":"pg_catalog.varchar", // Database type
                                "rs_type": "String", // Rust type
                                "rs_slice": "str", // Optional. Default is None. If set, the argument of the generated code uses `&str` instead of `&String`.
                                "copy_cheap": false // Optional. Default is false. If true, the argument of the generated code uses `i32` instead of `&i32`.
                            }
                            // other overrides..
                        ]
                    }
                }
            ]
        }
    ]
}
```

See [source code](https://github.com/sqlc-dev/sqlc/blob/v1.29.0/internal/codegen/golang/postgresql_type.go#L37-L605) for details on overwriting.

### `output`

Generated code destination. Default is `queries.rs`. 

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](./LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](./LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.