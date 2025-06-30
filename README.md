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
                "url": "https://github.com/tunamaguro/sqlc-gen-rust/releases/download/v0.1.1/sqlc-gen-rust.wasm",
                "sha256": "6306cf54f019d732cc0fdbc1b1bb2958a5a020bbd2f5f6aeafd48045a60a631e"
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
                    "out": "examples/e-commerce/src"
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



## Options

> [!NOTE]
> This plugin supports json only.

### `db_crate`

The crate of generated code. Default is `tokio-postgres`. Available values are below.

- `postgres`
- `tokio-postgres`
- `deadpool-postgres`

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

Change the output destination of the generated code. Default is `queries.rs`. 

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](./LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](./LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.