use super::DbCrate;
use crate::{
    query::{Annotation, DbEnum, Query, ReturningRows, RsType, SimpleTypeMap, TypeMapper},
    value_ident,
};

#[derive(Default)]
pub struct MySqlTypeMap {
    /// db_type to rust type
    type_map: std::collections::BTreeMap<String, RsType>,
}

impl TypeMapper for MySqlTypeMap {
    fn find_rs_type(&self, db_type_name: &str) -> Option<&RsType> {
        self.type_map.get(db_type_name)
    }

    fn find_column_type(&self, column: &crate::plugin::Column) -> Option<RsType> {
        let col_type = column
            .r#type
            .as_ref()
            .map(crate::query::make_column_type)?
            .to_lowercase();
        if let Some(rs_type) = self.find_rs_type(&col_type) {
            return Some(rs_type.clone());
        };

        match col_type.as_str() {
            "tinyint" => match (column.length, column.unsigned) {
                (1, _) => Some(RsType::new(syn::parse_str("bool").unwrap(), None, true)),
                (_, true) => Some(RsType::new(syn::parse_str("u8").unwrap(), None, true)),
                (_, false) => Some(RsType::new(syn::parse_str("i8").unwrap(), None, true)),
            },
            "smallint" => {
                if column.unsigned {
                    Some(RsType::new(syn::parse_str("u16").unwrap(), None, true))
                } else {
                    Some(RsType::new(syn::parse_str("i16").unwrap(), None, true))
                }
            }
            "int" | "integer" | "mediumint" => {
                if column.unsigned {
                    Some(RsType::new(syn::parse_str("u32").unwrap(), None, true))
                } else {
                    Some(RsType::new(syn::parse_str("i32").unwrap(), None, true))
                }
            }
            "bigint" => {
                if column.unsigned {
                    Some(RsType::new(syn::parse_str("u64").unwrap(), None, true))
                } else {
                    Some(RsType::new(syn::parse_str("i64").unwrap(), None, true))
                }
            }
            _ => None,
        }
    }

    fn insert_db_type(&mut self, db_type: &str, rs_type: RsType) {
        self.type_map.insert(db_type.to_string(), rs_type);
    }
}

#[derive(Default)]
pub struct SqliteTypeMap {
    /// db_type to rust type
    type_map: std::collections::BTreeMap<String, RsType>,
}

impl TypeMapper for SqliteTypeMap {
    fn find_rs_type(&self, db_type_name: &str) -> Option<&RsType> {
        self.type_map.get(db_type_name)
    }

    fn find_column_type(&self, column: &crate::plugin::Column) -> Option<RsType> {
        let col_type = column
            .r#type
            .as_ref()
            .map(crate::query::make_column_type)?
            .to_lowercase();
        if let Some(rs_type) = self.find_rs_type(&col_type) {
            return Some(rs_type.clone());
        };

        // Rust type determine by affinity
        // See https://www.sqlite.org/datatype3.html
        if col_type.contains("int") {
            return self.find_rs_type("int").cloned();
        }

        if col_type.contains("char") || col_type.contains("clob") || col_type.contains("text") {
            return self.find_rs_type("text").cloned();
        }

        if col_type.contains("blob") || col_type.is_empty() {
            return self.find_rs_type("blob").cloned();
        }

        if col_type.contains("real") || col_type.contains("floa") || col_type.contains("doub") {
            return self.find_rs_type("real").cloned();
        }

        self.find_rs_type("numeric").cloned()
    }

    fn insert_db_type(&mut self, db_type: &str, rs_type: RsType) {
        self.type_map.insert(db_type.to_string(), rs_type);
    }
}

struct CopyDataSink;

impl CopyDataSink {
    fn ident() -> syn::Ident {
        quote::format_ident!("CopyDataSink")
    }

    fn box_error() -> syn::Type {
        syn::parse_quote! {
            Box<dyn std::error::Error + Send + Sync>
        }
    }

    fn generic_constraint() -> syn::Type {
        syn::parse_quote! {
            std::ops::DerefMut<Target = sqlx::PgConnection>
        }
    }

    fn struct_tokens() -> proc_macro2::TokenStream {
        let ident = Self::ident();
        let constraint = Self::generic_constraint();
        quote::quote! {
           pub struct #ident<C: #constraint> {
                encode_buf: sqlx::postgres::PgArgumentBuffer,
                data_buf: Vec<u8>,
                copy_in: sqlx::postgres::PgCopyIn<C>,
            }
        }
    }

    fn impl_fn() -> proc_macro2::TokenStream {
        let ident = Self::ident();
        let constraint = Self::generic_constraint();
        let error_type = Self::box_error();
        quote::quote! {
                impl<C: #constraint> #ident<C> {
                    const BUFFER_SIZE: usize = 4096;

                    fn new(copy_in: sqlx::postgres::PgCopyIn<C>) -> Self {
                        let mut data_buf = Vec::with_capacity(Self::BUFFER_SIZE);
                        const COPY_SIGNATURE: &[u8] = &[
                            b'P', b'G', b'C', b'O', b'P', b'Y', b'\n',
                            0xFF,
                            b'\r', b'\n',
                            0x00,
                        ];

                        assert_eq!(COPY_SIGNATURE.len(), 11);
                        data_buf.extend_from_slice(COPY_SIGNATURE);
                        data_buf.extend(0_i32.to_be_bytes());
                        data_buf.extend(0_i32.to_be_bytes());

                        CopyDataSink {
                            encode_buf: Default::default(),
                            data_buf,
                            copy_in,
                        }
                    }

                    async fn send(&mut self) -> Result<(), #error_type> {
                        let _copy_in = self.copy_in.send(self.data_buf.as_slice()).await?;

                        self.data_buf.clear();
                        Ok(())
                    }

                    /// Complete copy process and return number of rows affected.
                    pub async fn finish(mut self) -> Result<u64, #error_type> {
                        const COPY_TRAILER: &[u8] = &(-1_i16).to_be_bytes();

                        self.data_buf.extend(COPY_TRAILER);
                        self.send().await?;
                        self.copy_in.finish().await.map_err(|e| e.into())
                    }

                    fn insert_row(&mut self) {
                        let num_col = self.copy_in.num_columns() as i16;
                        self.data_buf.extend(num_col.to_be_bytes());
                    }

                    async fn add<'q, T>(&mut self, value: &T) -> Result<(), #error_type>
                    where
                        T: sqlx::Encode<'q, sqlx::Postgres> + sqlx::Type<sqlx::Postgres>,
                    {
                        let is_null = value.encode_by_ref(&mut self.encode_buf)?;

                        match is_null {
                            sqlx::encode::IsNull::Yes => {
                                self.data_buf.extend((-1_i32).to_be_bytes());
                            }
                            sqlx::encode::IsNull::No => {
                                self.data_buf
                                    .extend((self.encode_buf.len() as i32).to_be_bytes());
                                self.data_buf.extend_from_slice(self.encode_buf.as_slice());
                            }
                        }

                        self.encode_buf.clear();

                        if self.data_buf.len() > Self::BUFFER_SIZE {
                            self.send().await?;
                        }

                        Ok(())
                    }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) enum Sqlx {
    #[default]
    Postgres,
    MySql,
    Sqlite,
}

impl<'de> serde::Deserialize<'de> for Sqlx {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.trim() {
            "sqlx-postgres" => Ok(Self::Postgres),
            "sqlx-mysql" => Ok(Self::MySql),
            "sqlx-sqlite" => Ok(Self::Sqlite),
            _ => Err(serde::de::Error::custom(format!(
                "`{s}` is unsupported crate."
            ))),
        }
    }
}

impl Sqlx {
    fn returning_row(&self, row: &ReturningRows) -> proc_macro2::TokenStream {
        let mut row = row.clone();

        for field in row.fields.iter_mut() {
            let original = &field.name_original;
            let att = &field.attribute;
            let attribute = quote::quote! {
                #att
                #[sqlx(rename = #original)]
            };
            field.attribute = Some(attribute);
        }
        let struct_tt = super::make_return_row(&row);

        let derive_tt = quote::quote! {#[derive(sqlx::FromRow)]};
        quote::quote! {
            #derive_tt
            #struct_tt
        }
    }

    fn copy_cheap_types(&self) -> &[(&str, &[&str])] {
        match self {
            Sqlx::Postgres => {
                const COPY_CHEAP: &[(&str, &[&str])] = &[
                    ("i8", &["char"]),
                    ("i16", &["smallint", "int2", "pg_catalog.int2"]),
                    ("i32", &["serial", "serial4", "pg_catalog.serial4"]),
                    ("i64", &["bigserial", "serial8", "pg_catalog.serial8"]),
                    ("i16", &["smallserial", "serial2", "pg_catalog.serial2"]),
                    ("i32", &["integer", "int", "int4", "pg_catalog.int4"]),
                    ("i64", &["bigint", "int8", "pg_catalog.int8"]),
                    (
                        "f64",
                        &["float", "double precision", "float8", "pg_catalog.float8"],
                    ),
                    ("f32", &["real", "float4", "pg_catalog.float4"]),
                    ("bool", &["boolean", "bool", "pg_catalog.bool"]),
                    ("sqlx::postgres::types::Oid", &["oid", "pg_catalog.oid"]),
                    ("uuid::Uuid", &["uuid"]),
                ];
                COPY_CHEAP
            }
            Sqlx::MySql => {
                const COPY_CHEAP: &[(&str, &[&str])] = &[
                    ("bool", &["bool", "boolean"]),
                    // int type is handle in `get_column_type`
                    ("int16", &["year"]),
                    ("f32", &["float"]),
                    ("f64", &["double", "double precision", "real"]),
                    ("sqlx::mysql::types::MySqlTime", &["time"]),
                ];
                COPY_CHEAP
            }
            Sqlx::Sqlite => {
                const COPY_CHEAP: &[(&str, &[&str])] = &[
                    ("bool", &["bool", "boolean"]),
                    ("i8", &["tinyint"]),
                    ("i16", &["smallint", "int2"]),
                    ("i32", &["mediumint", "int4"]),
                    ("i64", &["int", "integer", "bigint", "int8"]),
                    ("f64", &["real", "double", "doubleprecision", "float"]),
                    // NUMERIC affinity
                    ("f64", &["numeric"]),
                ];
                COPY_CHEAP
            }
        }
    }

    fn default_types(&self) -> &[(&str, Option<&str>, &[&'static str])] {
        match self {
            Sqlx::Postgres => {
                /// See below
                /// - https://github.com/sqlc-dev/sqlc/blob/v1.29.0/internal/codegen/golang/postgresql_type.go#L37-L605
                /// - https://docs.rs/sqlx/latest/sqlx/postgres/types/index.html
                const DEFAULT_TYPE: &[(&str, Option<&str>, &[&str])] = &[
                    (
                        "String",
                        Some("str"),
                        &[
                            "text",
                            "pg_catalog.varchar",
                            "pg_catalog.bpchar",
                            "string",
                            "citext",
                            "name",
                        ],
                    ),
                    (
                        "Vec<u8>",
                        Some("[u8]"),
                        &["bytea", "blob", "pg_catalog.bytea"],
                    ),
                    (
                        "sqlx::postgres::types::PgInterval",
                        None,
                        &["interval", "pg_catalog.interval"],
                    ),
                    // TODO: Add PgRange<T>
                    // https://github.com/sqlc-dev/sqlc/blob/v1.29.0/internal/codegen/golang/postgresql_type.go#L355-L461
                    ("sqlx::postgres::types::PgMoney", None, &["money"]),
                    ("sqlx::postgres::types::PgLTree", None, &["ltree"]),
                    ("sqlx::postgres::types::PgLQuery", None, &["lquery"]),
                    // `citext` is not added because `String` is usually sufficient.
                    ("sqlx::postgres::types::PgCube", None, &["cube"]),
                    ("sqlx::postgres::types::PgPoint", None, &["point"]),
                    ("sqlx::postgres::types::PgLine", None, &["line"]),
                    ("sqlx::postgres::types::PgLSeg", None, &["lseg"]),
                    ("sqlx::postgres::types::PgBox", None, &["box"]),
                    ("sqlx::postgres::types::PgPath", None, &["path"]),
                    ("sqlx::postgres::types::PgPolygon", None, &["polygon"]),
                    ("sqlx::postgres::types::PgCircle", None, &["circle"]),
                    ("sqlx::postgres::types::PgHstore", None, &["hstore"]),
                    (
                        "sqlx::postgres::types::PgTimeTz",
                        None,
                        &["pg_catalog.timetz"],
                    ),
                    ("std::net::IpAddr", None, &["inet"]),
                    (
                        "serde_json::Value",
                        None,
                        &["json", "pg_catalog.json", "jsonb", "pg_catalog.jsonb"],
                    ),
                ];
                DEFAULT_TYPE
            }
            Sqlx::MySql => {
                /// https://github.com/sqlc-dev/sqlc/blob/v1.29.0/internal/codegen/golang/mysql_type.go
                /// https://docs.rs/sqlx/0.8.6/sqlx/mysql/types/index.html
                const DEFAULT_TYPE: &[(&str, Option<&str>, &[&str])] = &[
                    (
                        "String",
                        Some("str"),
                        &[
                            "varchar",
                            "text",
                            "char",
                            "tinytext",
                            "mediumtext",
                            "longtext",
                        ],
                    ),
                    (
                        "Vec<u8>",
                        Some("[u8]"),
                        &[
                            "blob",
                            "binary",
                            "varbinary",
                            "tinyblob",
                            "mediumblob",
                            "longblob",
                        ],
                    ),
                    ("serde_json::Value", None, &["json"]),
                    ("String", Some("str"), &["decimal", "dec", "fixed", "enum"]),
                ];
                DEFAULT_TYPE
            }
            Sqlx::Sqlite => {
                /// https://github.com/sqlc-dev/sqlc/blob/v1.29.0/internal/codegen/golang/sqlite_type.go
                /// https://docs.rs/sqlx/latest/sqlx/sqlite/types/index.html
                const DEFAULT_TYPE: &[(&str, Option<&str>, &[&str])] = &[
                    ("String", Some("str"), &["text", "clob"]),
                    ("Vec<u8>", Some("[u8]"), &["blob"]),
                ];
                DEFAULT_TYPE
            }
        }
    }

    fn database_ident(&self) -> syn::Type {
        match self {
            Sqlx::Postgres => syn::parse_quote! {sqlx::Postgres},
            Sqlx::MySql => syn::parse_quote! {sqlx::MySql},
            Sqlx::Sqlite => syn::parse_quote! {sqlx::Sqlite},
        }
    }
}

impl DbCrate for Sqlx {
    /// Creates a new `DbTypeMap` with default types for PostgreSQL.
    fn type_map(&self) -> Box<dyn crate::query::TypeMapper> {
        let copy_cheap = self.copy_cheap_types();

        let default_types = self.default_types();

        let mut map: Box<dyn TypeMapper> = match self {
            Sqlx::Postgres => Box::new(SimpleTypeMap::default()),
            Sqlx::MySql => Box::new(MySqlTypeMap::default()),
            Sqlx::Sqlite => Box::new(SqliteTypeMap::default()),
        };

        for (owned_type, pg_types) in copy_cheap {
            let owned_type = syn::parse_str::<syn::Type>(owned_type).expect("Failed to parse type");

            for pg_type in pg_types.iter() {
                map.insert_db_type(pg_type, RsType::new(owned_type.clone(), None, true));
            }
        }

        for (owned_type, slice_type, pg_types) in default_types {
            let owned_type = syn::parse_str::<syn::Type>(owned_type).expect("Failed to parse type");
            let slice_type = slice_type
                .map(|s| syn::parse_str::<syn::Type>(s).expect("Failed to parse slice type"));

            for pg_type in pg_types.iter() {
                map.insert_db_type(
                    pg_type,
                    RsType::new(owned_type.clone(), slice_type.clone(), false),
                );
            }
        }
        map
    }

    fn init(&self) -> proc_macro2::TokenStream {
        match self {
            Sqlx::Postgres => {
                let copy_data_sync = {
                    let struct_tt = CopyDataSink::struct_tokens();
                    let impl_fn = CopyDataSink::impl_fn();
                    quote::quote! {
                        #struct_tt
                        #impl_fn
                    }
                };
                quote::quote! {
                    #copy_data_sync
                }
            }
            _ => quote::quote! {},
        }
    }

    fn defined_enum(&self, enum_type: &DbEnum) -> proc_macro2::TokenStream {
        let derives = &enum_type.derives;
        let fields = enum_type.values.iter().map(|field| {
            let ident = value_ident(field);
            quote::quote! {
                #[sqlx(rename = #field)]
                #ident
            }
        });

        let original_name = &enum_type.name;
        let enum_name = enum_type.ident();
        let derive_tt = if derives.is_empty() {
            quote::quote! {#[derive(Debug,Clone,Copy, sqlx::Type)]}
        } else {
            quote::quote! {#[derive(Debug,Clone,Copy, sqlx::Type, #(#derives),*)]}
        };
        quote::quote! {
            #derive_tt
            #[sqlx(type_name = #original_name)]
            pub enum #enum_name {
                #(#fields,)*
            }
        }
    }

    fn generate_query(&self, row: &ReturningRows, query: &Query) -> proc_macro2::TokenStream {
        let query_ast = super::QueryAst::new(query);
        let struct_ident = &query_ast.ident;
        let lifetime_a = &query_ast.lifetime;
        let need_lifetime = query_ast.need_lifetime();

        let query_fns = {
            let database_ident = self.database_ident();
            let row_ident = row.struct_ident();

            let lifetime_b = syn::Lifetime::new("'b", proc_macro2::Span::call_site());

            let lifetime_generic = if need_lifetime {
                quote::quote! {#lifetime_b  }
            } else {
                quote::quote! {#lifetime_a, #lifetime_b }
            };

            if matches!(self, Sqlx::MySql | Sqlx::Sqlite) && query.has_slices {
                // MySQL/SQLite slice-aware codegen: dynamic query expansion at runtime
                let expand_slices = query_ast.fields.iter()
                    .filter(|f| f.is_sqlc_slice)
                    .map(|f| {
                        let name = &f.name;
                        let marker = syn::LitStr::new(
                            &format!("/*SLICE:{}*/?", f.name_original.value()),
                            proc_macro2::Span::call_site(),
                        );
                        quote::quote! {
                            if self.#name.is_empty() {
                                query = query.replacen(#marker, "NULL", 1);
                            } else {
                                query = query.replacen(#marker, &",?".repeat(self.#name.len())[1..], 1);
                            }
                        }
                    })
                    .collect::<Vec<_>>();

                let expand_query_fn = quote::quote! {
                    fn expand_query(&self) -> String {
                        let mut query = Self::QUERY.to_string();
                        #(#expand_slices)*
                        query
                    }
                };

                let bind_block = {
                    let tokens: Vec<_> = query_ast.fields.iter().map(|f| {
                        let name = &f.name;
                        if f.is_sqlc_slice {
                            quote::quote! {
                                for v in self.#name {
                                    q = q.bind(v);
                                }
                            }
                        } else {
                            quote::quote! {
                                q = q.bind(self.#name);
                            }
                        }
                    }).collect();
                    quote::quote! { #(#tokens)* }
                };

                let fn_tt = match query.annotation {
                    Annotation::One => {
                        quote::quote! {
                            pub fn query_one<#lifetime_generic,A>(&#lifetime_a self,conn:A)
                            ->impl Future<Output=Result<#row_ident,sqlx::Error>> + Send + #lifetime_a
                            where A: sqlx::Acquire<#lifetime_b, Database = #database_ident> + Send + #lifetime_a,
                            {
                                async move {
                                    let mut conn = conn.acquire().await?;
                                    let query_str = self.expand_query();
                                    let mut q = sqlx::query_as::<_,#row_ident>(&query_str);
                                    #bind_block
                                    let val = q.fetch_one(&mut *conn).await?;
                                    Ok(val)
                                }
                            }

                            pub fn query_opt<#lifetime_generic,A>(&#lifetime_a self,conn:A)
                            ->impl Future<Output=Result<Option<#row_ident>,sqlx::Error>> + Send + #lifetime_a
                            where A: sqlx::Acquire<#lifetime_b, Database = #database_ident> + Send + #lifetime_a,
                            {
                                async move {
                                    let mut conn = conn.acquire().await?;
                                    let query_str = self.expand_query();
                                    let mut q = sqlx::query_as::<_,#row_ident>(&query_str);
                                    #bind_block
                                    let val = q.fetch_optional(&mut *conn).await?;
                                    Ok(val)
                                }
                            }
                        }
                    }
                    Annotation::Many => {
                        quote::quote! {
                            pub fn query_many<#lifetime_generic,A>(&#lifetime_a self,conn:A)
                            ->impl Future<Output=Result<Vec<#row_ident>,sqlx::Error>> + Send + #lifetime_a
                            where A: sqlx::Acquire<#lifetime_b, Database = #database_ident> + Send + #lifetime_a,
                            {
                                async move {
                                    let mut conn = conn.acquire().await?;
                                    let query_str = self.expand_query();
                                    let mut q = sqlx::query_as::<_,#row_ident>(&query_str);
                                    #bind_block
                                    let vals = q.fetch_all(&mut *conn).await?;
                                    Ok(vals)
                                }
                            }
                        }
                    }
                    Annotation::Exec | Annotation::ExecResult | Annotation::ExecRows => {
                        quote::quote! {
                            pub fn execute<#lifetime_generic,A>(&#lifetime_a self,conn:A)
                            ->impl Future<Output=Result<<#database_ident as sqlx::Database>::QueryResult,sqlx::Error>> + Send + #lifetime_a
                            where A: sqlx::Acquire<#lifetime_b, Database = #database_ident> + Send + #lifetime_a,
                            {
                                async move {
                                    let mut conn = conn.acquire().await?;
                                    let query_str = self.expand_query();
                                    let mut q = sqlx::query(&query_str);
                                    #bind_block
                                    q.execute(&mut *conn).await
                                }
                            }
                        }
                    }
                    _ => quote::quote! {},
                };

                quote::quote! {
                    #expand_query_fn
                    #fn_tt
                }
            } else {
                // Standard codegen: static query string with chained binds
                let query_as_def = if need_lifetime {
                    quote::quote! {
                        query_as(&#lifetime_a self)
                    }
                } else {
                    quote::quote! {
                         query_as<#lifetime_a>(&#lifetime_a self)
                    }
                };

                let bind_params = query_ast.fields.iter().map(|f| &f.name).fold(
                    quote::quote! {},
                    |acc, x| quote::quote! {#acc .bind(self.#x)},
                );

                // `sqlx::query_as(QUERY).fetch` returns `Stream` trait directly, but we do not add other dependencies
                let query_as = quote::quote! {
                    pub fn #query_as_def->sqlx::query::QueryAs<
                    #lifetime_a,
                    #database_ident,
                    #row_ident,
                    <#database_ident as sqlx::Database>::Arguments<#lifetime_a>,
                    >{
                        sqlx::query_as::<_,#row_ident>(
                                        Self::QUERY,
                                    ) #bind_params
                    }
                };

                let fn_tt = match (self, query.annotation) {
                    (_, Annotation::One) => {
                        // See https://docs.rs/sqlx/latest/sqlx/trait.Acquire.html
                        quote::quote! {
                            pub fn query_one<#lifetime_generic,A>(&#lifetime_a self,conn:A)
                            ->impl Future<Output=Result<#row_ident,sqlx::Error>> + Send + #lifetime_a
                            where A: sqlx::Acquire<#lifetime_b, Database = #database_ident> + Send + #lifetime_a,
                            {
                                async move {
                                    let mut conn = conn.acquire().await?;
                                    let val = self.query_as().fetch_one(&mut *conn).await?;

                                    Ok(val)
                                }
                            }

                            pub fn query_opt<#lifetime_generic,A>(&#lifetime_a self,conn:A)
                            ->impl Future<Output=Result<Option<#row_ident>,sqlx::Error>> + Send + #lifetime_a
                            where A: sqlx::Acquire<#lifetime_b, Database = #database_ident> + Send + #lifetime_a,
                            {
                                async move {
                                    let mut conn = conn.acquire().await?;
                                    let val = self.query_as().fetch_optional(&mut *conn).await?;

                                    Ok(val)
                                }
                            }
                        }
                    }
                    (_, Annotation::Many) => {
                        let row_ident = row.struct_ident();

                        quote::quote! {
                            pub fn query_many<#lifetime_generic,A>(&#lifetime_a self,conn:A)
                            ->impl Future<Output=Result<Vec<#row_ident>,sqlx::Error>> + Send + #lifetime_a
                            where A: sqlx::Acquire<#lifetime_b, Database = #database_ident> + Send + #lifetime_a,
                            {
                                async move {
                                    let mut conn = conn.acquire().await?;
                                    let vals = self.query_as().fetch_all(&mut *conn).await?;

                                    Ok(vals)
                                }
                            }

                        }
                    }
                    (_, Annotation::Exec | Annotation::ExecResult | Annotation::ExecRows) => {
                        quote::quote! {
                            pub fn execute<#lifetime_generic,A>(&#lifetime_a self,conn:A)
                            ->impl Future<Output=Result<<#database_ident as sqlx::Database>::QueryResult,sqlx::Error>> + Send + #lifetime_a
                            where A: sqlx::Acquire<#lifetime_b, Database = #database_ident> + Send + #lifetime_a,
                            {
                                async move {
                                    let mut conn = conn.acquire().await?;
                                    sqlx::query(
                                        Self::QUERY,
                                    )  #bind_params .execute(&mut *conn).await
                                }
                            }
                        }
                    }
                    (Sqlx::Postgres, Annotation::CopyFrom) => {
                        let add_row = query.fields.iter().map(|x| {
                            let name = &x.name;
                            quote::quote! {sink.add(&self.#name).await?;}
                        });
                        let sink_ident = CopyDataSink::ident();
                        let sink_error = CopyDataSink::box_error();
                        let constraint = CopyDataSink::generic_constraint();

                        quote::quote! {
                            pub async fn copy_in<PgCopy>(
                                conn: &PgCopy,
                            ) -> Result<CopyDataSink<sqlx::pool::PoolConnection<#database_ident>>, sqlx::Error>
                            where
                                PgCopy: sqlx::postgres::PgPoolCopyExt,
                            {
                                let copy_in = conn.copy_in_raw(Self::QUERY).await?;
                                Ok(CopyDataSink::new(copy_in))
                            }
                            pub async fn copy_in_tx(
                                conn: &mut sqlx::postgres::PgConnection,
                            ) -> Result<CopyDataSink<&mut sqlx::postgres::PgConnection>, sqlx::Error> {
                                let copy_in = conn.copy_in_raw(Self::QUERY).await?;
                                Ok(CopyDataSink::new(copy_in))
                            }

                            pub async fn write<C: #constraint>(&self, sink: &mut #sink_ident<C>) -> Result<(), #sink_error> {
                                sink.insert_row();
                                #(#add_row)*
                                Ok(())
                            }
                        }
                    }
                    _ => {
                        // not supported
                        quote::quote! {}
                    }
                };

                quote::quote! {
                    #query_as
                    #fn_tt
                }
            }
        };
        let fetch_tt = {
            let query_str = query.query_str();
            let imp_ident = if need_lifetime {
                quote::quote! {<#lifetime_a> #struct_ident<#lifetime_a>}
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

        let returning_row = self.returning_row(row);
        let builder_tt = query_ast.make_builder();
        quote::quote! {
            #returning_row
            #query_ast
            #fetch_tt
            #builder_tt
        }
    }
}
