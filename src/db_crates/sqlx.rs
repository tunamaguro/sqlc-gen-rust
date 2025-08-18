use super::DbCrate;
use crate::{
    query::{
        Annotation, DbEnum, DbTypeMap, DbTypeMapper, Query, QueryError, ReturningRows, RsType,
        make_column_type,
    },
    value_ident,
};

#[derive(Default)]
pub struct MySqlTypeMap {
    /// db_type to rust type
    type_map: std::collections::BTreeMap<String, RsType>,
    /// column name to rust type
    column_map: std::collections::BTreeMap<String, RsType>,
}

impl DbTypeMapper for MySqlTypeMap {
    fn get_column_type(
        &self,
        column: &crate::plugin::Column,
    ) -> Result<RsType, crate::query::QueryError> {
        let db_col_name = crate::query::make_column_name(column);
        if let Some(rs_type) = self.column_map.get(&db_col_name) {
            return Ok(rs_type.clone());
        };

        let col_type = column
            .r#type
            .as_ref()
            .map(make_column_type)
            .ok_or_else(|| QueryError::missing_column_type(db_col_name.clone()))?;

        if let Some(rs_type) = self.type_map.get(&col_type) {
            return Ok(rs_type.clone());
        };

        match col_type.as_str() {
            "tinyint" => match (column.length, column.unsigned) {
                (1, _) => Ok(RsType::new(syn::parse_str("bool").unwrap(), None, true)),
                (_, true) => Ok(RsType::new(syn::parse_str("u8").unwrap(), None, true)),
                (_, false) => Ok(RsType::new(syn::parse_str("i8").unwrap(), None, true)),
            },
            "smallint" => {
                if column.unsigned {
                    Ok(RsType::new(syn::parse_str("u16").unwrap(), None, true))
                } else {
                    Ok(RsType::new(syn::parse_str("i16").unwrap(), None, true))
                }
            }
            "int" | "integer" | "mediumint" => {
                if column.unsigned {
                    Ok(RsType::new(syn::parse_str("u32").unwrap(), None, true))
                } else {
                    Ok(RsType::new(syn::parse_str("i32").unwrap(), None, true))
                }
            }
            "bigint" => {
                if column.unsigned {
                    Ok(RsType::new(syn::parse_str("u64").unwrap(), None, true))
                } else {
                    Ok(RsType::new(syn::parse_str("i64").unwrap(), None, true))
                }
            }
            _ => Err(QueryError::cannot_map_type(
                db_col_name,
                col_type.to_string(),
            )),
        }
    }

    fn insert_db_type(&mut self, db_type: &str, rs_type: RsType) -> Option<RsType> {
        self.type_map.insert(db_type.to_string(), rs_type)
    }

    fn insert_column_type(&mut self, column_name: &str, rs_type: RsType) -> Option<RsType> {
        self.column_map.insert(column_name.to_string(), rs_type)
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
        let fields = row
            .column_names
            .iter()
            .zip(row.column_types.iter())
            .map(|(col, rs_type)| {
                let col_t = rs_type.to_row_tokens();
                quote::quote! {pub #col:#col_t}
            })
            .collect::<Vec<_>>();

        let ident = row.struct_ident();
        let row_tt = quote::quote! {
            #[derive(sqlx::FromRow)]
            pub struct #ident {
                #(#fields,)*
            }
        };

        row_tt
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
                    (
                        "sqlx::mysql::types::MySqlTime",
                        &["date", "timestamp", "datetime", "time"],
                    ),
                ];
                COPY_CHEAP
            }
            Sqlx::Sqlite => {
                const COPY_CHEAP: &[(&str, &[&str])] = &[
                    ("bool", &["bool", "boolean"]),
                    (
                        "i64",
                        &[
                            "int",
                            "integer",
                            "tinyint",
                            "smallint",
                            "mediumint",
                            "bigint",
                            "unsignedbigint",
                            "int2",
                            "int4",
                            "int8",
                        ],
                    ),
                    ("f64", &["real", "double", "doubleprecision", "float"]),
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
                    ("String", Some("str"), &["text"]),
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
    fn db_type_map(&self) -> Box<dyn DbTypeMapper> {
        let copy_cheap = self.copy_cheap_types();

        let default_types = self.default_types();

        let mut map: Box<dyn DbTypeMapper> = match self {
            Sqlx::Postgres => Box::new(DbTypeMap::default()),
            Sqlx::MySql => Box::new(MySqlTypeMap::default()),
            Sqlx::Sqlite => Box::new(DbTypeMap::default()),
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
        let fields = enum_type
            .values
            .iter()
            .map(|field| {
                let ident = value_ident(field);
                quote::quote! {
                    #[sqlx(rename = #field)]
                    #ident
                }
            })
            .collect::<Vec<_>>();

        let original_name = &enum_type.name;
        let enum_name = enum_type.ident();
        quote::quote! {
            #[derive(Debug,Clone,Copy, sqlx::Type)]
            #[sqlx(type_name = #original_name)]
            pub enum #enum_name {
                #(#fields,)*
            }
        }
    }

    fn generate_query(&self, row: &ReturningRows, query: &Query) -> proc_macro2::TokenStream {
        let struct_ident = value_ident(&query.query_name);
        let lifetime_a = syn::Lifetime::new("'a", proc_macro2::Span::call_site());

        let fields = query
            .param_names
            .iter()
            .zip(query.param_types.iter())
            .map(|(r, typ)| {
                let typ = typ.to_param_tokens(&lifetime_a);
                quote::quote! {#r:#typ}
            })
            .collect::<Vec<_>>();

        let need_lifetime = super::need_lifetime(query);
        let has_fields = !query.param_names.is_empty();
        let struct_tt = match (need_lifetime, has_fields) {
            (true, _) => {
                quote::quote! {
                    pub struct #struct_ident<#lifetime_a>{
                        #(#fields,)*
                    }
                }
            }
            (false, true) => {
                quote::quote! {
                    pub struct #struct_ident{
                        #(#fields,)*
                    }
                }
            }
            (false, false) => {
                quote::quote! {
                    pub struct #struct_ident;
                }
            }
        };

        let params = query.param_names.iter().fold(
            quote::quote! {},
            |acc, x| quote::quote! {#acc .bind(self.#x)},
        );
        let query_fns = {
            let database_ident = self.database_ident();
            let row_ident = row.struct_ident();

            let query_as_def = if need_lifetime {
                quote::quote! {
                    query_as(&#lifetime_a self)
                }
            } else {
                quote::quote! {
                     query_as<#lifetime_a>(&#lifetime_a self)
                }
            };
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
                                ) #params
                }
            };

            let lifetime_b = syn::Lifetime::new("'b", proc_macro2::Span::call_site());

            let lifetime_generic = if need_lifetime {
                quote::quote! {#lifetime_b  }
            } else {
                quote::quote! {#lifetime_a, #lifetime_b }
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
                                )  #params .execute(&mut *conn).await
                            }
                        }
                    }
                }
                (Sqlx::Postgres, Annotation::CopyFrom) => {
                    let add_row = query.param_names.iter().map(|x| {
                        quote::quote! {sink.add(&self.#x).await?;}
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
        let builder_tt = super::create_builder(query);
        quote::quote! {
            #returning_row
            #struct_tt
            #fetch_tt
            #builder_tt
        }
    }
}
