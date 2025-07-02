use super::DbCrate;
use crate::{
    query::{Annotation, DbEnum, DbTypeMap, Query, ReturningRows, RsType},
    value_ident,
};

#[derive(Debug, Clone, Copy, Default)]
pub(crate) enum Sqlx {
    #[default]
    Postgres,
}

impl<'de> serde::Deserialize<'de> for Sqlx {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.trim() {
            "sqlx-postgres" => Ok(Self::Postgres),
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
}

impl DbCrate for Sqlx {
    /// Creates a new `DbTypeMap` with default types for PostgreSQL.
    ///
    /// See below
    /// - https://github.com/sqlc-dev/sqlc/blob/v1.29.0/internal/codegen/golang/postgresql_type.go#L37-L605
    /// - https://docs.rs/sqlx/latest/sqlx/postgres/types/index.html
    fn db_type_map(&self) -> crate::query::DbTypeMap {
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
            (
                ("sqlx::postgres::types::PgInterval", None),
                vec!["interval", "pg_catalog.interval"],
            ),
            // TODO: Add PgRange<T>
            // https://github.com/sqlc-dev/sqlc/blob/v1.29.0/internal/codegen/golang/postgresql_type.go#L355-L461
            (("sqlx::postgres::types::PgMoney", None), vec!["money"]),
            (("sqlx::postgres::types::PgLTree", None), vec!["ltree"]),
            (("sqlx::postgres::types::PgLQuery", None), vec!["lquery"]),
            // `citext` is not added because `String` is usually sufficient.
            (("sqlx::postgres::types::PgCube", None), vec!["cube"]),
            (("sqlx::postgres::types::PgPoint", None), vec!["point"]),
            (("sqlx::postgres::types::PgLine", None), vec!["line"]),
            (("sqlx::postgres::types::PgLSeg", None), vec!["lseg"]),
            (("sqlx::postgres::types::PgBox", None), vec!["box"]),
            (("sqlx::postgres::types::PgPath", None), vec!["path"]),
            (("sqlx::postgres::types::PgPolygon", None), vec!["polygon"]),
            (("sqlx::postgres::types::PgCircle", None), vec!["circle"]),
            (("sqlx::postgres::types::PgHstore", None), vec!["hstore"]),
            (
                ("sqlx::postgres::types::PgTimeTz", None),
                vec!["pg_catalog.timetz"],
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
                map.insert_type(pg_type, RsType::new(owned_type.clone(), None, true));
            }
        }

        for ((owned_type, slice_type), pg_types) in default_types {
            let owned_type = syn::parse_str::<syn::Type>(owned_type).expect("Failed to parse type");
            let slice_type = slice_type
                .map(|s| syn::parse_str::<syn::Type>(s).expect("Failed to parse slice type"));

            for pg_type in pg_types {
                map.insert_type(
                    pg_type,
                    RsType::new(owned_type.clone(), slice_type.clone(), false),
                );
            }
        }
        map
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
            let query_str = &query.query_str;
            let lifetime_b = syn::Lifetime::new("'b", proc_macro2::Span::call_site());

            let lifetime_generic = if need_lifetime {
                quote::quote! {#lifetime_b  }
            } else {
                quote::quote! {#lifetime_a, #lifetime_b }
            };

            match query.annotation {
                Annotation::One => {
                    let row_ident = row.struct_ident();

                    // See https://docs.rs/sqlx/latest/sqlx/trait.Acquire.html
                    quote::quote! {
                        pub fn query_one<#lifetime_generic,A>(&#lifetime_a self,conn:A)
                        ->impl Future<Output=Result<#row_ident,sqlx::Error>> + Send + #lifetime_a
                        where A: sqlx::Acquire<#lifetime_b, Database = sqlx::Postgres> + Send + #lifetime_a,
                        {
                            async move {
                                let mut conn = conn.acquire().await?;
                                let val :#row_ident = sqlx::query_as(
                                    #query_str,
                                ) #params .fetch_one(&mut *conn).await?;

                                Ok(val)
                            }
                        }

                        pub fn query_opt<#lifetime_generic,A>(&#lifetime_a self,conn:A)
                        ->impl Future<Output=Result<Option<#row_ident>,sqlx::Error>> + Send + #lifetime_a
                        where A: sqlx::Acquire<#lifetime_b, Database = sqlx::Postgres> + Send + #lifetime_a,
                        {
                            async move {
                                let mut conn = conn.acquire().await?;
                                let val:Option<#row_ident> = sqlx::query_as(
                                    #query_str,
                                ) #params .fetch_optional(&mut *conn).await?;

                                Ok(val)
                            }
                        }
                    }
                }
                Annotation::Many => {
                    let row_ident = row.struct_ident();

                    quote::quote! {
                        pub fn query_many<#lifetime_generic,A>(&#lifetime_a self,conn:A)
                        ->impl Future<Output=Result<Vec<#row_ident>,sqlx::Error>> + Send + #lifetime_a
                        where A: sqlx::Acquire<#lifetime_b, Database = sqlx::Postgres> + Send + #lifetime_a,
                        {
                            async move {
                                let mut conn = conn.acquire().await?;
                                let vals:Vec<#row_ident> = sqlx::query_as(
                                    #query_str,
                                ) #params .fetch_all(&mut *conn).await?;

                                Ok(vals)
                            }
                        }
                    }
                }
                Annotation::Exec | Annotation::ExecResult | Annotation::ExecRows => {
                    quote::quote! {
                        pub fn execute<#lifetime_generic,A>(&#lifetime_a self,conn:A)
                        ->impl Future<Output=Result<<sqlx::Postgres as sqlx::Database>::QueryResult,sqlx::Error>> + Send + #lifetime_a
                        where A: sqlx::Acquire<#lifetime_b, Database = sqlx::Postgres> + Send + #lifetime_a,
                        {
                            async move {
                                let mut conn = conn.acquire().await?;
                                sqlx::query(
                                    #query_str,
                                )  #params .execute(&mut *conn).await
                            }
                        }
                    }
                }
                _ => {
                    // not supported
                    quote::quote! {}
                }
            }
        };
        let fetch_tt = {
            let imp_ident = if need_lifetime {
                quote::quote! {<#lifetime_a> #struct_ident<#lifetime_a>}
            } else {
                quote::quote! {#struct_ident}
            };
            quote::quote! {
                impl #imp_ident {
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
