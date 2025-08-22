# sqlc-gen-rust

sqlc plugin for Rust database crates.

## Usage

Add the following to your `sqlc.json` configuration file to use this plugin.

```json
{
    "version": "2",
    "plugins": [
        {
            "name": "sqlc-gen-rust",
            "wasm": {
                "url": "https://github.com/tunamaguro/sqlc-gen-rust/releases/download/v0.1.9/sqlc-gen-rust.wasm",
                "sha256": "ba3bfb9b5ce6c3121ada3afc3481d6adb8791afa72907d0026698db965b6dbef"
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
- [sqlx-mysql](https://docs.rs/sqlx/latest/sqlx/mysql/index.html)
- [sqlx-sqlite](https://docs.rs/sqlx/latest/sqlx/sqlite/index.html)

> [!NOTE]
> When using `sqlx-sqlite`: SQLite uses dynamic typing. Columns with **NUMERIC affinity** may store values as **INTEGER** when they can be represented exactly as integers. 
> For example, `13.0` may be stored as `13`. The generated code always reads NUMERIC as `f64` (`REAL`), so decoding can fail with a type mismatch when SQLite returns an integer. See the [SQLite type affinity docs](https://www.sqlite.org/datatype3.html) and the [`sqlx` type mapping docs](https://docs.rs/sqlx/latest/sqlx/sqlite/types/index.html) for details.


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
-- name: GetAuthor :one
SELECT * FROM authors
WHERE id = $1 LIMIT 1;

-- name: ListAuthors :many
SELECT * FROM authors
ORDER BY name;

-- name: CreateAuthor :one
INSERT INTO authors (
          name, bio
) VALUES (
  $1, $2
)
RETURNING *;

-- name: DeleteAuthor :exec
DELETE FROM authors
WHERE id = $1;
```

### Using generated code

```rust
mod queries;

use queries::{CreateAuthor, DeleteAuthor, ListAuthors};

#[tokio::main]
async fn main() {
    let (client, conn) = tokio_postgres::connect(
        &std::env::var("DATABASE_URL").unwrap(),
        tokio_postgres::NoTls,
    )
    .await
    .unwrap();
    tokio::spawn(async move {
        if let Err(e) = conn.await {
            panic!("connection error: {e}");
        }
    });

    // list authors
    let authors = ListAuthors.query_many(&client).await.unwrap();
    assert_eq!(authors.len(), 0);
    // let author_stream = ListAuthors.query_stream(&client).await.unwrap(); // stream of rows

    // crate and get an author (INSERT ... RETURNING ...)
    let author = {
        let binding = CreateAuthor::builder()
            .name("John")
            .bio(Some("Foo"))
            .build();

        // let binding = CreateAuthor::builder().name("John").build(); // missing field won't compile

        binding.query_one(&client).await.unwrap()
        //  binding.query_opt(&client).await.unwrap() // this returns Option<T>
    };
    assert_eq!(author.id, 0);

    // delete author
    let affected_row = DeleteAuthor::builder()
        .id(0)
        .build()
        .execute(&client)
        .await
        .unwrap();
    assert_eq!(affected_row, 1);
}
```

See below for examples with other supported crates.

- [`postgres` generated code](./examples/authors/postgres/src/lib.rs)
- [`tokio-postgres` generated code](./examples/authors/tokio-postgres/src/lib.rs)
- [`deadpool-postgres` generated code](./examples/authors/deadpool-postgres/src/lib.rs)
- [`sqlx-postgres` generated code](./examples/authors/sqlx-postgres/src/lib.rs)
- [`sqlx-mysql` generated code](./examples/authors/sqlx-mysql/src/lib.rs)
- [`sqlx-sqlite` generated code](./examples/authors/sqlx-sqlite/src/lib.rs)

## Options

> [!NOTE]
> This plugin supports JSON only. YAML is not supported.

### `db_crate`

The crate used in the generated code. Default is `tokio-postgres`. Available values are below.

- `postgres` 
- `tokio-postgres`
- `deadpool-postgres`
- `sqlx-postgres`
- `sqlx-mysql`
- `sqlx-sqlite`

### `overrides`

Customize Rust type mapping per column or database type. Each entry **must include exactly one** of the following: `column` or `db_type`.

- `column`: Override a specific table column (e.g. `users.metadata`)
- `db_type`: Override all columns of a given database type (e.g. `pg_catalog.varchar`)

When both are specified, it will result in an error. Furthermore, entries with a `column` key are always prioritized over `db_type` overrides.

The following is an example configuration:

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
                                "db_type":"pg_catalog.varchar", 
                                "rs_type": "String", // Required. The target Rust type.
                                "rs_slice": "str", // Optional. If set, the argument of the generated code uses `&str` instead of `&String`.
                                "copy_cheap": false // Optional. If true, the argument of the generated code uses `i32` instead of `&i32`.
                            },
                            {
                                "column": "users.metadata",
                                "rs_type": "serde_json::Value"
                            }
                            // other overrides...
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