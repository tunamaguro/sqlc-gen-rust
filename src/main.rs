use convert_case::{Case, Casing as _};
use prost::Message as _;
use quote::ToTokens;
use std::{
    io::{Read as _, Write},
    vec,
};

pub mod plugin {
    include!(concat!(env!("OUT_DIR"), "/plugin.rs"));
}

fn deserialize_codegen_request(data: &[u8]) -> Result<plugin::GenerateRequest, prost::DecodeError> {
    plugin::GenerateRequest::decode(data)
}

fn serialize_codegen_response(response: &plugin::GenerateResponse) -> Vec<u8> {
    response.encode_to_vec()
}

fn normalize_str(value: &str) -> String {
    use regex_lite::Regex;
    use std::sync::LazyLock;
    static IDENT_PATTERN: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r#"[^a-zA-Z0-9_]"#).unwrap());

    let value = value.replace("-", "_");
    let value = value.replace(":", "_");
    let value = value.replace("/", "_");
    let value = IDENT_PATTERN.replace_all(&value, "");
    value.to_string()
}

fn value_ident(ident: &str) -> syn::Ident {
    let ident = normalize_str(ident).to_case(Case::Pascal);
    quote::format_ident!("{}", ident)
}

fn field_ident(ident: &str) -> syn::Ident {
    let ident = normalize_str(ident).to_case(Case::Snake);
    quote::format_ident!("{}", ident)
}

#[derive(Clone)]
struct RsType {
    owned: syn::Type,
    slice: Option<syn::Type>,
}

impl RsType {
    fn new(owned: syn::Type, slice: Option<syn::Type>) -> Self {
        RsType { owned, slice }
    }

    /// 自己所有の型を返す
    fn owned(&self) -> proc_macro2::TokenStream {
        self.owned.to_token_stream()
    }

    /// スライスの型を返す。これに`&`をつけると参照になる
    #[allow(unused)]
    fn slice(&self) -> proc_macro2::TokenStream {
        if let Some(ref slice) = self.slice {
            slice.to_token_stream()
        } else {
            self.owned()
        }
    }
}

#[derive(Default)]
struct DbTypeMap {
    inner: std::collections::BTreeMap<String, RsType>,
}

impl DbTypeMap {
    /// Creates a new `DbTypeMap` with default types for PostgreSQL.
    ///
    /// See below
    /// -
    /// - https://github.com/sqlc-dev/sqlc/blob/v1.29.0/internal/codegen/golang/postgresql_type.go#L37-L605
    /// - https://docs.rs/postgres-types/0.2.9/postgres_types/trait.ToSql.html#types
    /// - https://www.postgresql.jp/document/17/html/datatype.html
    fn new_for_postgres() -> Self {
        let default_types = [
            (
                ("i32", None),
                vec!["serial", "serial4", "pg_catalog.serial4"],
            ),
            (
                ("i64", None),
                vec!["bigserial", "serial8", "pg_catalog.serial8"],
            ),
            (
                ("i16", None),
                vec!["smallserial", "serial2", "pg_catalog.serial2"],
            ),
            (
                ("i32", None),
                vec!["integer", "int", "int4", "pg_catalog.int4"],
            ),
            (("i64", None), vec!["bigint", "int8", "pg_catalog.int8"]),
            (("i16", None), vec!["smallint", "int2", "pg_catalog.int2"]),
            (
                ("f64", None),
                vec!["float", "double precision", "float8", "pg_catalog.float8"],
            ),
            (("f32", None), vec!["real", "float4", "pg_catalog.float4"]),
            (("bool", None), vec!["boolean", "bool", "pg_catalog.bool"]),
            (("u32", None), vec!["oid", "pg_catalog.oid"]),
            (
                ("String", Some("str")),
                vec![
                    "text",
                    "pg_catalog.varchar",
                    "pg_catalog.bpchar",
                    "string",
                    "citext",
                    "name",
                ],
            ),
            (
                ("Vec<u8>", Some("[u8]")),
                vec!["bytea", "blob", "pg_catalog.bytea"],
            ),
            (("HashMap<String, Option<String>>", None), vec!["hstore"]),
            (
                ("std::time::SystemTime", None),
                vec![
                    "pg_catalog.timestamp",
                    "timestamp",
                    "pg_catalog.timestamptz",
                    "timestamptz",
                ],
            ),
            (("std::net::IpAddr", None), vec!["inet"]),
            (
                ("serde_json::Value", None),
                vec!["json", "pg_catalog.json", "jsonb", "pg_catalog.jsonb"],
            ),
            (("uuid::Uuid", None), vec!["uuid"]),
        ];

        let mut map = DbTypeMap::default();
        for ((owned_type, slice_type), pg_types) in default_types {
            let owned_type = syn::parse_str::<syn::Type>(owned_type).expect("Failed to parse type");
            let slice_type = slice_type
                .map(|s| syn::parse_str::<syn::Type>(s).expect("Failed to parse slice type"));

            for pg_type in pg_types {
                map.inner.insert(
                    pg_type.to_string(),
                    RsType::new(owned_type.clone(), slice_type.clone()),
                );
            }
        }
        map
    }

    fn get(&self, db_type: &str) -> RsType {
        if let Some(rs_type) = self.inner.get(db_type) {
            rs_type.clone()
        } else {
            RsType {
                owned: syn::parse_str("std::convert::Infallible").unwrap(),
                slice: None,
            }
        }
    }

    fn insert(&mut self, db_type: &str, rs_type: RsType) -> Option<RsType> {
        self.inner.insert(db_type.to_string(), rs_type)
    }
}

#[derive(Clone)]
struct DbEnum {
    /// name of enum
    ///
    /// ```sql
    /// CREATE TYPE book_type AS ENUM (
    ///             ^^^^^^^^^
    ///           'FICTION',
    ///           'NONFICTION'
    /// );
    /// ```
    name: String,

    /// values of enum
    ///
    /// CREATE TYPE book_type AS ENUM (
    ///           'FICTION',
    ///            ^^^^^^^
    ///           'NONFICTION'
    ///            ^^^^^^^^^^
    /// );
    /// ```
    values: Vec<String>,
}

impl DbEnum {
    fn ident(&self) -> syn::Ident {
        value_ident(&self.name)
    }

    fn to_pg_token(&self) -> proc_macro2::TokenStream {
        let fields = self
            .values
            .iter()
            .map(|field| {
                let ident = value_ident(field);
                quote::quote! {
                    #[postgres(name = #field)]
                    #ident
                }
            })
            .collect::<Vec<_>>();

        let original_name = &self.name;
        let enum_name = self.ident();
        quote::quote! {
            #[derive(Debug, postgres_types::ToSql, postgres_types::FromSql)]
            #[postgres(name = #original_name)]
            enum #enum_name {
                #(#fields,)*
            }
        }
    }
}

fn collect_enums(catalog: &plugin::Catalog) -> Vec<DbEnum> {
    let mut res = vec![];

    for schema in &catalog.schemas {
        for s_enum in &schema.enums {
            let db_enum = DbEnum {
                name: s_enum.name.clone(),
                values: s_enum.vals.clone(),
            };
            res.push(db_enum);
        }
    }

    res
}

fn make_column_type(db_type: &plugin::Identifier) -> String {
    if !db_type.schema.is_empty() {
        format!("{}.{}", db_type.schema, db_type.name)
    } else {
        db_type.name.to_string()
    }
}

fn create_returning_row(db_type: &DbTypeMap, query: &plugin::Query) -> proc_macro2::TokenStream {
    let mut columns_name = vec![];
    let mut columns_type = vec![];
    for column in &query.columns {
        let col_name = if let Some(table) = &column.table {
            format!("{}_{}", table.name, column.name)
        } else {
            column.name.to_string()
        };

        let db_col_type = column
            .r#type
            .as_ref()
            .map(make_column_type)
            .expect("column type not found");

        let rs_type = db_type.get(&db_col_type);

        columns_name.push(field_ident(&col_name));
        columns_type.push(rs_type);
    }

    let fields = columns_name
        .iter()
        .zip(columns_type)
        .map(|(col, rs_type)| {
            let col_t_own = rs_type.owned();
            quote::quote! {#col:#col_t_own}
        })
        .collect::<Vec<_>>();

    let struct_name = value_ident(&format!("{}Row", query.name));

    // struct XXXRow {
    //  table_col: i32,...
    // }
    quote::quote! {
        struct #struct_name {
            #(#fields,)*
        }
    }
}

fn main() {
    let mut stdin = std::io::stdin().lock();
    let mut buffer = Vec::new();
    stdin.read_to_end(&mut buffer).unwrap();

    let request = deserialize_codegen_request(&buffer).expect("Failed to decode GenerateRequest");

    let debug_file = plugin::File {
        name: "debug.txt".to_string(),
        contents: format!("{:#?}", request).into_bytes(),
    };

    let bin_file = plugin::File {
        name: "input.bin".to_string(),
        contents: request.encode_to_vec(),
    };

    let mut response = plugin::GenerateResponse::default();
    response.files.push(debug_file);
    response.files.push(bin_file);

    let mut db_type = DbTypeMap::new_for_postgres();

    let defined_enums = request
        .catalog
        .as_ref()
        .map(collect_enums)
        .unwrap_or_default();

    for e in &defined_enums {
        db_type.insert(
            &e.name,
            RsType {
                owned: syn::parse_str(&e.ident().to_string()).unwrap(),
                slice: None,
            },
        );
    }

    let returning_rows = request
        .queries
        .iter()
        .map(|q| create_returning_row(&db_type, q))
        .collect::<Vec<_>>();

    let enums_ts = defined_enums
        .iter()
        .map(|e| e.to_pg_token())
        .collect::<Vec<_>>();
    let enums_tt = quote::quote! {#(#enums_ts)*};
    let rows_tt = quote::quote! {#(#returning_rows)*};

    let tt = quote::quote! {#enums_tt #rows_tt};
    let ast = syn::parse2(tt).unwrap();
    let query_file = plugin::File {
        name: "queries.rs".to_string(),
        contents: prettyplease::unparse(&ast).into(),
    };
    response.files.push(query_file);

    let serialized_response = serialize_codegen_response(&response);

    std::io::stdout()
        .write_all(&serialized_response)
        .expect("Failed to write response");
}
