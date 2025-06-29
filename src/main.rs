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
    copy_cheap: bool,
}

impl RsType {
    fn new(owned: syn::Type, slice: Option<syn::Type>, copy_cheap: bool) -> Self {
        RsType {
            owned,
            slice,
            copy_cheap,
        }
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

struct RsColType {
    rs_type: RsType,
    /// maybe dim
    dim: usize,
    /// col is optional
    optional: bool,
}

impl RsColType {
    fn new_with_type(db_type: &DbTypeMap, column: &plugin::Column) -> Self {
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
    fn to_row_tokens(&self) -> proc_macro2::TokenStream {
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

    fn need_lifetime(&self) -> bool {
        let is_slice = self.dim != 0;
        let copy_expensive = !self.rs_type.copy_cheap;

        is_slice || copy_expensive
    }

    /// Convert to tokens for function parameter struct
    fn to_param_tokens(&self, life_time: &syn::Lifetime) -> proc_macro2::TokenStream {
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
struct DbTypeMap {
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
    fn new_for_postgres() -> Self {
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

    fn insert_type(&mut self, db_type: &str, rs_type: RsType) -> Option<RsType> {
        self.typ_map.insert(db_type.to_string(), rs_type)
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

struct ReturningRows {
    column_names: Vec<syn::Ident>,
    column_types: Vec<RsColType>,
    query_name: String,
}

impl ReturningRows {
    fn from_query(db_type: &DbTypeMap, query: &plugin::Query) -> Self {
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

    fn struct_ident(&self) -> syn::Ident {
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

struct Query {
    param_names: Vec<syn::Ident>,
    param_types: Vec<RsColType>,

    annotation: Annotation,
    /// ```sql
    /// -- name: GetAuthor :one
    ///          ^^^^^^^^^
    /// SELECT * FROM authors
    /// WHERE id = $1 LIMIT 1;
    /// ```
    query_name: String,
    /// ```sql
    /// -- name: GetAuthor :one
    /// SELECT * FROM authors
    /// ^^^^^^^^^^^^^^^^^^^^^
    /// WHERE id = $1 LIMIT 1;
    /// ^^^^^^^^^^^^^^^^^^^^^^
    /// ```
    query_str: proc_macro2::TokenStream,
}

impl Query {
    fn from_query(db_type: &DbTypeMap, query: &plugin::Query) -> Self {
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

trait DbCrate {
    /// Generate returning row
    fn returning_row(row: &ReturningRows) -> proc_macro2::TokenStream;
    /// Generate enum
    fn defined_enum(enum_type: &DbEnum) -> proc_macro2::TokenStream;
    /// Generate query fn
    fn call_query(row: &ReturningRows, query: &Query) -> proc_macro2::TokenStream;
}

struct TokioPostgres;

impl TokioPostgres {
    fn need_lifetime(query: &Query) -> bool {
        query
            .param_types
            .iter()
            .fold(false, |acc, x| acc | x.need_lifetime())
    }
}

impl DbCrate for TokioPostgres {
    fn returning_row(row: &ReturningRows) -> proc_macro2::TokenStream {
        let fields = row
            .column_names
            .iter()
            .zip(row.column_types.iter())
            .map(|(col, rs_type)| {
                let col_t = rs_type.to_row_tokens();
                quote::quote! {#col:#col_t}
            })
            .collect::<Vec<_>>();

        let ident = row.struct_ident();

        // struct XXXRow {
        //  table_col: i32,...
        // }
        let row_tt = quote::quote! {
            struct #ident {
                #(#fields,)*
            }
        };

        let arg_ident = quote::format_ident!("row");
        let from_fields = row
            .column_names
            .iter()
            .enumerate()
            .map(|(idx, r)| {
                let literal = proc_macro2::Literal::usize_unsuffixed(idx);
                quote::quote! {#r:#arg_ident.try_get(#literal)?}
            })
            .collect::<Vec<_>>();
        let from_tt = quote::quote! {
            impl #ident {
                fn from_row(#arg_ident: &tokio_postgres::Row)->Result<Self,tokio_postgres::Error>{
                    Ok(Self{
                        #(#from_fields,)*
                    })
                }
            }
        };

        quote::quote! {
            #row_tt
            #from_tt
        }
    }
    fn defined_enum(enum_type: &DbEnum) -> proc_macro2::TokenStream {
        let fields = enum_type
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

        let original_name = &enum_type.name;
        let enum_name = enum_type.ident();
        quote::quote! {
            #[derive(Debug,Clone,Copy, postgres_types::ToSql, postgres_types::FromSql)]
            #[postgres(name = #original_name)]
            enum #enum_name {
                #(#fields,)*
            }
        }
    }
    fn call_query(row: &ReturningRows, query: &Query) -> proc_macro2::TokenStream {
        let struct_ident = quote::format_ident!("{}", &query.query_name);
        let lifetime = syn::Lifetime::new("'a", proc_macro2::Span::call_site());

        let fields = query
            .param_names
            .iter()
            .zip(query.param_types.iter())
            .map(|(r, typ)| {
                let typ = typ.to_param_tokens(&lifetime);
                quote::quote! {#r:#typ}
            })
            .collect::<Vec<_>>();

        let need_lifetime = TokioPostgres::need_lifetime(query);
        let has_fields = !query.param_names.is_empty();
        let struct_tt = match (need_lifetime, has_fields) {
            (true, _) => {
                quote::quote! {
                    struct #struct_ident<#lifetime>{
                        #(#fields,)*
                    }
                }
            }
            (false, true) => {
                quote::quote! {
                    struct #struct_ident{
                        #(#fields,)*
                    }
                }
            }
            (false, false) => {
                quote::quote! {
                    struct #struct_ident;
                }
            }
        };

        let client_ident = quote::format_ident!("client");
        let client_typ = syn::parse_str::<syn::Type>("impl tokio_postgres::GenericClient").unwrap();
        let params = query
            .param_names
            .iter()
            .fold(quote::quote! {}, |acc, x| quote::quote! {#acc &self.#x,});
        let params = quote::quote! {[#params]};

        let query_fns = match query.annotation {
            Annotation::One => {
                let row_ident = row.struct_ident();

                quote::quote! {
                    async fn query_one(&self,#client_ident: &#client_typ)->Result<#row_ident,tokio_postgres::Error>{
                        let row = client.query_one(Self::QUERY, &#params).await?;
                        #row_ident::from_row(&row)
                    }

                    async fn query_opt(&self,#client_ident: &#client_typ)->Result<Option<#row_ident>,tokio_postgres::Error>{
                        let row = client.query_opt(Self::QUERY, &#params).await?;
                        match row {
                            Some(row) => Ok(Some(#row_ident::from_row(&row)?)),
                            None => Ok(None)
                        }
                    }
                }
            }
            Annotation::Many => {
                let row_ident = row.struct_ident();

                quote::quote! {
                    async fn query_many(&self,#client_ident: &#client_typ)->Result<Vec<#row_ident>,tokio_postgres::Error>{
                        let rows = client.query(Self::QUERY, &#params).await?;
                        rows.into_iter().map(|r|#row_ident::from_row(&r)).collect()
                    }
                }
            }
            Annotation::Exec => {
                quote::quote! {
                    async fn execute(&self,#client_ident: &#client_typ)->Result<u64,tokio_postgres::Error>{
                        client.execute(Self::QUERY, &#params).await
                    }
                }
            }
            _ => {
                // not supported
                quote::quote! {}
            }
        };

        let fetch_tt = {
            let query_str = &query.query_str;
            let imp_ident = if need_lifetime {
                quote::quote! {<#lifetime> #struct_ident<#lifetime>}
            } else {
                quote::quote! {#struct_ident}
            };
            quote::quote! {
                impl #imp_ident {
                    pub const QUERY:&'static str = #query_str;
                    #query_fns
                }
            }
        };

        quote::quote! {
            #struct_tt
            #fetch_tt
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
        db_type.insert_type(
            &e.name,
            RsType {
                owned: syn::TypePath {
                    qself: None,
                    path: e.ident().clone().into(),
                }
                .into(),
                slice: None,
                copy_cheap: true,
            },
        );
    }

    let returning_rows = request
        .queries
        .iter()
        .map(|q| ReturningRows::from_query(&db_type, q))
        .collect::<Vec<_>>();
    let queries = request
        .queries
        .iter()
        .map(|q| Query::from_query(&db_type, q))
        .collect::<Vec<_>>();

    let enums_ts = defined_enums
        .iter()
        .map(TokioPostgres::defined_enum)
        .collect::<Vec<_>>();
    let enums_tt = quote::quote! {#(#enums_ts)*};

    let queries_ts = returning_rows
        .iter()
        .zip(queries.iter())
        .map(|(r, q)| {
            let row_tt = TokioPostgres::returning_row(r);
            let query_tt = TokioPostgres::call_query(r, q);

            quote::quote! {
                #row_tt
                #query_tt
            }
        })
        .collect::<Vec<_>>();
    let queries_tt = quote::quote! {#(#queries_ts)*};

    let tt = quote::quote! {
        #enums_tt
        #queries_tt
    };
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
