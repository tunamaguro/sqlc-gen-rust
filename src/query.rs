use quote::ToTokens;
use std::vec;

use crate::{field_ident, plugin, value_ident};

#[derive(Clone)]
pub(crate) struct RsType {
    owned: syn::Type,
    slice: Option<syn::Type>,
    copy_cheap: bool,
}

impl RsType {
    pub(crate) fn new(owned: syn::Type, slice: Option<syn::Type>, copy_cheap: bool) -> Self {
        RsType {
            owned,
            slice,
            copy_cheap,
        }
    }

    /// 自己所有の型を返す
    pub(crate) fn owned(&self) -> proc_macro2::TokenStream {
        self.owned.to_token_stream()
    }

    /// スライスの型を返す。これに`&`をつけると参照になる
    #[allow(unused)]
    pub(crate) fn slice(&self) -> proc_macro2::TokenStream {
        if let Some(ref slice) = self.slice {
            slice.to_token_stream()
        } else {
            self.owned()
        }
    }
}

pub(crate) struct RsColType {
    rs_type: RsType,
    /// maybe dim
    dim: usize,
    /// col is optional
    optional: bool,
}

impl RsColType {
    pub(crate) fn new_with_type(db_type: &DbTypeMap, column: &plugin::Column) -> Self {
        fn make_column_type(db_type: &plugin::Identifier) -> String {
            if !db_type.schema.is_empty() {
                format!("{}.{}", db_type.schema, db_type.name)
            } else {
                db_type.name.to_string()
            }
        }

        let db_col_type = column
            .r#type
            .as_ref()
            .map(make_column_type)
            .expect("column type not found");

        let rs_type = db_type.get(&db_col_type);
        let dim = usize::try_from(column.array_dims).unwrap_or_default();
        let optional = !column.not_null;

        Self {
            rs_type,
            dim,
            optional,
        }
    }

    /// Convert to tokens for row struct
    pub(crate) fn to_row_tokens(&self) -> proc_macro2::TokenStream {
        let base_type = self.rs_type.owned();

        // 配列の次元数に応じてVecでラップ
        let mut wrapped_type = base_type;
        for _ in 0..self.dim {
            wrapped_type = quote::quote! { Vec<#wrapped_type> };
        }

        // optionalの場合はOptionでラップ
        if self.optional {
            quote::quote! { Option<#wrapped_type> }
        } else {
            wrapped_type
        }
    }

    pub(crate) fn need_lifetime(&self) -> bool {
        let is_slice = self.dim != 0;
        let copy_expensive = !self.rs_type.copy_cheap;

        is_slice || copy_expensive
    }

    /// Convert to tokens for function parameter struct
    pub(crate) fn to_param_tokens(&self, life_time: &syn::Lifetime) -> proc_macro2::TokenStream {
        let wrapped_type = match self.dim {
            0 => {
                let slice_type = self.rs_type.slice();
                quote::quote! {#slice_type}
            }
            _ => {
                let mut base_type = self.rs_type.owned();
                for _ in 1..self.dim {
                    base_type = quote::quote! {Vec<#base_type>}
                }

                quote::quote! {[#base_type]}
            }
        };

        match (self.need_lifetime(), self.optional) {
            (true, true) => {
                quote::quote! {Option<&#life_time #wrapped_type>}
            }
            (true, false) => {
                quote::quote! {&#life_time #wrapped_type}
            }
            (false, true) => {
                quote::quote! {Option<#wrapped_type>}
            }
            (false, false) => {
                quote::quote! {#wrapped_type}
            }
        }
    }
}

#[derive(Default)]
pub(crate) struct DbTypeMap {
    /// db_type to rust type
    typ_map: std::collections::BTreeMap<String, RsType>,
}

impl DbTypeMap {
    /// Creates a new `DbTypeMap` with default types for PostgreSQL.
    ///
    /// See below
    /// -
    /// - https://github.com/sqlc-dev/sqlc/blob/v1.29.0/internal/codegen/golang/postgresql_type.go#L37-L605
    /// - https://docs.rs/postgres-types/0.2.9/postgres_types/trait.ToSql.html#types
    /// - https://www.postgresql.jp/document/17/html/datatype.html
    pub(crate) fn new_for_postgres() -> Self {
        let copy_cheap = [
            ("i32", vec!["serial", "serial4", "pg_catalog.serial4"]),
            ("i64", vec!["bigserial", "serial8", "pg_catalog.serial8"]),
            ("i16", vec!["smallserial", "serial2", "pg_catalog.serial2"]),
            ("i32", vec!["integer", "int", "int4", "pg_catalog.int4"]),
            ("i64", vec!["bigint", "int8", "pg_catalog.int8"]),
            ("i16", vec!["smallint", "int2", "pg_catalog.int2"]),
            (
                "f64",
                vec!["float", "double precision", "float8", "pg_catalog.float8"],
            ),
            ("f32", vec!["real", "float4", "pg_catalog.float4"]),
            ("bool", vec!["boolean", "bool", "pg_catalog.bool"]),
            ("u32", vec!["oid", "pg_catalog.oid"]),
            ("uuid::Uuid", vec!["uuid"]),
        ];

        let default_types = [
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
        ];

        let mut map = DbTypeMap::default();

        for (owned_type, pg_types) in copy_cheap {
            let owned_type = syn::parse_str::<syn::Type>(owned_type).expect("Failed to parse type");

            for pg_type in pg_types {
                map.typ_map.insert(
                    pg_type.to_string(),
                    RsType::new(owned_type.clone(), None, true),
                );
            }
        }

        for ((owned_type, slice_type), pg_types) in default_types {
            let owned_type = syn::parse_str::<syn::Type>(owned_type).expect("Failed to parse type");
            let slice_type = slice_type
                .map(|s| syn::parse_str::<syn::Type>(s).expect("Failed to parse slice type"));

            for pg_type in pg_types {
                map.typ_map.insert(
                    pg_type.to_string(),
                    RsType::new(owned_type.clone(), slice_type.clone(), false),
                );
            }
        }
        map
    }

    fn get(&self, db_type: &str) -> RsType {
        if let Some(rs_type) = self.typ_map.get(db_type) {
            rs_type.clone()
        } else {
            RsType {
                owned: syn::parse_str("std::convert::Infallible").unwrap(),
                slice: None,
                copy_cheap: false,
            }
        }
    }

    pub(crate) fn insert_type(&mut self, db_type: &str, rs_type: RsType) -> Option<RsType> {
        self.typ_map.insert(db_type.to_string(), rs_type)
    }
}

#[derive(Clone)]
pub(crate) struct DbEnum {
    /// name of enum
    ///
    /// ```sql
    /// CREATE TYPE book_type AS ENUM (
    ///             ^^^^^^^^^
    ///           'FICTION',
    ///           'NONFICTION'
    /// );
    /// ```
    pub(crate) name: String,

    /// values of enum
    ///
    /// CREATE TYPE book_type AS ENUM (
    ///           'FICTION',
    ///            ^^^^^^^
    ///           'NONFICTION'
    ///            ^^^^^^^^^^
    /// );
    /// ```
    pub(crate) values: Vec<String>,
}

impl DbEnum {
    pub(crate) fn ident(&self) -> syn::Ident {
        value_ident(&self.name)
    }
}

pub(crate) fn collect_enums(catalog: &plugin::Catalog) -> Vec<DbEnum> {
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

pub(crate) struct ReturningRows {
    pub(crate) column_names: Vec<syn::Ident>,
    pub(crate) column_types: Vec<RsColType>,
    pub(crate) query_name: String,
}

impl ReturningRows {
    pub(crate) fn from_query(db_type: &DbTypeMap, query: &plugin::Query) -> Self {
        let mut column_names = vec![];
        let mut column_types = vec![];
        for column in &query.columns {
            let col_name = if let Some(table) = &column.table {
                format!("{}_{}", table.name, column.name)
            } else {
                column.name.to_string()
            };

            let rs_type = RsColType::new_with_type(db_type, column);

            column_names.push(field_ident(&col_name));
            column_types.push(rs_type);
        }

        Self {
            column_names,
            column_types,
            query_name: query.name.to_string(),
        }
    }

    pub(crate) fn struct_ident(&self) -> syn::Ident {
        value_ident(&format!("{}Row", self.query_name))
    }
}

/// sqlc annotation
/// See https://docs.sqlc.dev/en/stable/reference/query-annotations.html
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Annotation {
    Exec,
    ExecResult,
    ExecRows,
    ExecLastId,
    Many,
    One,
    BatchExec,
    BatchMany,
    BatchOne,
    CopyFrom,
    Unknown(String),
}

impl std::fmt::Display for Annotation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let txt = match self {
            Annotation::Exec => ":exec",
            Annotation::ExecResult => ":execresult",
            Annotation::ExecRows => ":execrows",
            Annotation::ExecLastId => ":execlastid",
            Annotation::Many => ":many",
            Annotation::One => ":one",
            Annotation::BatchExec => ":batch",
            Annotation::BatchMany => ":batchmany",
            Annotation::BatchOne => ":batchone",
            Annotation::CopyFrom => ":copyfrom",
            Annotation::Unknown(s) => s,
        };
        f.write_str(txt)
    }
}

impl std::str::FromStr for Annotation {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let annotation = match s {
            ":exec" => Annotation::Exec,
            ":execresult" => Annotation::ExecResult,
            ":execrows" => Annotation::ExecRows,
            ":execlastid" => Annotation::ExecLastId,
            ":many" => Annotation::Many,
            ":one" => Annotation::One,
            ":batch" => Annotation::BatchExec,
            ":batchmany" => Annotation::BatchMany,
            ":batchone" => Annotation::BatchOne,
            ":copyfrom" => Annotation::CopyFrom,
            _ => Annotation::Unknown(s.to_string()),
        };
        Ok(annotation)
    }
}

pub(crate) struct Query {
    pub(crate) param_names: Vec<syn::Ident>,
    pub(crate) param_types: Vec<RsColType>,

    pub(crate) annotation: Annotation,
    /// ```sql
    /// -- name: GetAuthor :one
    ///          ^^^^^^^^^
    /// SELECT * FROM authors
    /// WHERE id = $1 LIMIT 1;
    /// ```
    pub(crate) query_name: String,
    /// ```sql
    /// -- name: GetAuthor :one
    /// SELECT * FROM authors
    /// ^^^^^^^^^^^^^^^^^^^^^
    /// WHERE id = $1 LIMIT 1;
    /// ^^^^^^^^^^^^^^^^^^^^^^
    /// ```
    pub(crate) query_str: proc_macro2::TokenStream,
}

impl Query {
    pub(crate) fn from_query(db_type: &DbTypeMap, query: &plugin::Query) -> Self {
        let mut param_data = vec![];
        for param in &query.params {
            let col = param.column.as_ref().unwrap();
            let col_name = if let Some(table) = &col.table {
                format!("{}_{}", table.name, col.name)
            } else if !col.name.is_empty() {
                col.name.to_string()
            } else {
                "param".to_string()
            };

            let param_name = quote::format_ident!("{}", col_name);
            let param_type = RsColType::new_with_type(db_type, col);
            let param_idx = param.number;

            param_data.push((param_name, param_type, param_idx));
        }

        param_data.sort_by_key(|(_, _, idx)| *idx);
        let (param_names, param_types): (Vec<_>, Vec<_>) = param_data
            .into_iter()
            .map(|(name, typ, _)| (name, typ))
            .unzip();

        let annotation = query.cmd.parse::<Annotation>().unwrap();
        let query_name = query.name.to_string();

        fn make_raw_string_literal(s: &str) -> proc_macro2::TokenStream {
            // 文字列内の"#の組み合わせを検出して、必要なハッシュ数を決定
            let mut hash_count = 0;
            let mut current_hashes = 0;
            let mut in_quote = false;

            for ch in s.chars() {
                match ch {
                    '"' => {
                        if in_quote {
                            hash_count = hash_count.max(current_hashes + 1);
                        }
                        in_quote = !in_quote;
                        current_hashes = 0;
                    }
                    '#' if in_quote => {
                        current_hashes += 1;
                    }
                    _ => {
                        current_hashes = 0;
                    }
                }
            }

            // raw string literalを構築
            let hashes = "#".repeat(hash_count);
            let raw_str = format!("r{0}\"{1}\"{0}", hashes, s);

            raw_str
                .parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| proc_macro2::Literal::string(s).to_token_stream())
        }

        let query_str = make_raw_string_literal(&query.text);

        Self {
            param_names,
            param_types,
            annotation,
            query_name,
            query_str,
        }
    }
}
