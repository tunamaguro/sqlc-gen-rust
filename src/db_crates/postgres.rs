use quote::ToTokens;

use super::DbCrate;
use crate::{
    query::{Annotation, DbEnum, DbTypeMap, Query, ReturningRows, RsType},
    value_ident,
};

#[derive(Debug, Clone, Copy, Default)]
pub(crate) enum Postgres {
    Sync,
    #[default]
    Tokio,
    DeadPool,
}

impl<'de> serde::Deserialize<'de> for Postgres {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.trim() {
            "postgres" => Ok(Self::Sync),
            "tokio-postgres" => Ok(Self::Tokio),
            "deadpool-postgres" => Ok(Self::DeadPool),
            _ => Err(serde::de::Error::custom(format!(
                "`{s}` is unsupported crate."
            ))),
        }
    }
}

impl Postgres {
    fn generic_client_type(&self, lifetime: Option<&syn::Lifetime>) -> syn::Type {
        let l = lifetime.map(|s| s.to_token_stream()).unwrap_or_default();
        let s = match self {
            Postgres::Sync => quote::quote! {&#l mut impl postgres::GenericClient},
            Postgres::Tokio => quote::quote! {&#l impl tokio_postgres::GenericClient},
            Postgres::DeadPool => quote::quote! {&#l impl deadpool_postgres::GenericClient},
        };
        syn::parse2(s).unwrap()
    }

    fn row_type(&self) -> syn::Type {
        let s = match self {
            Postgres::Sync => "postgres::Row",
            Postgres::Tokio => "tokio_postgres::Row",
            Postgres::DeadPool => "deadpool_postgres::tokio_postgres::Row",
        };
        syn::parse_str(s).unwrap()
    }

    fn error_type(&self) -> syn::Type {
        let s = match self {
            Postgres::Sync => "postgres::Error",
            Postgres::Tokio => "tokio_postgres::Error",
            Postgres::DeadPool => "deadpool_postgres::tokio_postgres::Error",
        };
        syn::parse_str(s).unwrap()
    }

    fn async_part(&self) -> proc_macro2::TokenStream {
        match self {
            Postgres::Sync => quote::quote! {},
            Postgres::Tokio => quote::quote! {async},
            Postgres::DeadPool => quote::quote! {async},
        }
    }

    fn await_part(&self) -> proc_macro2::TokenStream {
        match self {
            Postgres::Sync => quote::quote! {},
            Postgres::Tokio => quote::quote! {.await},
            Postgres::DeadPool => quote::quote! {.await},
        }
    }

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

        // struct XXXRow {
        //  table_col: i32,...
        // }
        let row_tt = quote::quote! {
            pub struct #ident {
                #(#fields,)*
            }
        };

        let error_typ = self.error_type();
        let row_typ = self.row_type();
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
                pub fn from_row(#arg_ident: &#row_typ)->Result<Self,#error_typ>{
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
}

impl DbCrate for Postgres {
    /// Creates a new `DbTypeMap` with default types for PostgreSQL.
    ///
    /// See below
    /// -
    /// - https://github.com/sqlc-dev/sqlc/blob/v1.29.0/internal/codegen/golang/postgresql_type.go#L37-L605
    /// - https://docs.rs/postgres-types/0.2.9/postgres_types/trait.ToSql.html#types
    /// - https://www.postgresql.jp/document/17/html/datatype.html
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

    fn init(&self) -> proc_macro2::TokenStream {
        let use_tosql = match self {
            Postgres::Sync => quote::quote! {use postgres::types::ToSql;},
            Postgres::Tokio => quote::quote! {use tokio_postgres::types::ToSql;},
            Postgres::DeadPool => {
                quote::quote! {use deadpool_postgres::tokio_postgres::types::ToSql;}
            }
        };

        quote::quote! {
            #use_tosql
        }
    }
    fn defined_enum(&self, enum_type: &DbEnum) -> proc_macro2::TokenStream {
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
            pub enum #enum_name {
                #(#fields,)*
            }
        }
    }
    fn generate_query(&self, row: &ReturningRows, query: &Query) -> proc_macro2::TokenStream {
        let struct_ident = value_ident(&query.query_name);
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

        let need_lifetime = super::need_lifetime(query);
        let has_fields = !query.param_names.is_empty();
        let struct_tt = match (need_lifetime, has_fields) {
            (true, _) => {
                quote::quote! {
                    pub struct #struct_ident<#lifetime>{
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

        let client_ident = quote::format_ident!("client");
        let client_typ = self.generic_client_type(None);
        let error_typ = self.error_type();
        let async_part = self.async_part();
        let await_part = self.await_part();

        let params = query
            .param_names
            .iter()
            .fold(quote::quote! {}, |acc, x| quote::quote! {#acc &self.#x,});

        let query_fns = match query.annotation {
            Annotation::One => {
                let row_ident = row.struct_ident();

                quote::quote! {
                    pub #async_part fn query_one(&self,#client_ident: #client_typ)->Result<#row_ident,#error_typ>{
                        let row = client.query_one(Self::QUERY, &self.as_slice()) #await_part?;
                        #row_ident::from_row(&row)
                    }

                    pub #async_part fn query_opt(&self,#client_ident: #client_typ)->Result<Option<#row_ident>,#error_typ>{
                        let row = client.query_opt(Self::QUERY, &self.as_slice()) #await_part?;
                        match row {
                            Some(row) => Ok(Some(#row_ident::from_row(&row)?)),
                            None => Ok(None)
                        }
                    }
                }
            }
            Annotation::Many => {
                let row_ident = row.struct_ident();

                let query_it = {
                    match self {
                        Postgres::Sync => {
                            quote::quote! {
                                pub fn query_iter<'row_iter>(&self,#client_ident:&'row_iter mut  impl postgres::GenericClient)
                                ->Result<postgres::RowIter<'row_iter>,#error_typ>
                                {

                                    client.query_raw(Self::QUERY, self.as_slice().into_iter())
                                }
                            }
                        }
                        Postgres::Tokio => {
                            quote::quote! {
                                pub async fn query_stream(&self,#client_ident: #client_typ)
                                ->Result<tokio_postgres::RowStream,#error_typ>{
                                    let st = client.query_raw(Self::QUERY, self.as_slice().into_iter()).await?;
                                    Ok(st)
                                }
                            }
                        }
                        Postgres::DeadPool => {
                            quote::quote! {
                                 pub async fn query_stream(&self,#client_ident: #client_typ)
                                ->Result<deadpool_postgres::tokio_postgres::RowStream,#error_typ>{
                                    let st = client.query_raw(Self::QUERY, self.as_slice().into_iter()).await?;
                                    Ok(st)
                                }
                            }
                        }
                    }
                };

                quote::quote! {
                    pub #async_part fn query_many(&self,#client_ident: #client_typ)->Result<Vec<#row_ident>,#error_typ>{
                        let rows = client.query(Self::QUERY, &[#params]) #await_part?;
                        rows.into_iter().map(|r|#row_ident::from_row(&r)).collect()
                    }

                    #query_it
                }
            }
            Annotation::Exec | Annotation::ExecResult | Annotation::ExecRows => {
                quote::quote! {
                    pub #async_part fn execute(&self,#client_ident: #client_typ)->Result<u64,#error_typ>{
                        client.execute(Self::QUERY, &self.as_slice()) #await_part
                    }
                }
            }
            _ => quote::quote! {},
        };

        let fetch_tt = {
            let query_str = query.query_str();
            let imp_ident = if need_lifetime {
                quote::quote! {<#lifetime> #struct_ident<#lifetime>}
            } else {
                quote::quote! {#struct_ident}
            };

            let param_num = proc_macro2::Literal::usize_unsuffixed(query.param_names.len());

            quote::quote! {
                impl #imp_ident {
                    pub const QUERY:&'static str = #query_str;
                    #query_fns

                    pub fn as_slice(&self) -> [&(dyn ToSql + Sync); #param_num] {
                        [ #params ]
                    }
                }
            }
        };

        let returning_row = self.returning_row(row);
        let builder = super::create_builder(query);
        quote::quote! {
            #returning_row
            #struct_tt
            #fetch_tt
            #builder
        }
    }
}
