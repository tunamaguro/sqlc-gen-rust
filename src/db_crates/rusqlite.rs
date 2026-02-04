use crate::{
    db_crates::DbCrate,
    query::{Annotation, RsType, TypeMapper},
};

struct SqliteTypeMap {
    type_map: std::collections::BTreeMap<String, RsType>,
}

impl SqliteTypeMap {
    fn new() -> Self {
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
        const DEFAULT_TYPE: &[(&str, Option<&str>, &[&str])] = &[
            ("String", Some("str"), &["text", "clob"]),
            ("Vec<u8>", Some("[u8]"), &["blob"]),
        ];

        let mut m = Self {
            type_map: Default::default(),
        };

        for (owned_type, pg_types) in COPY_CHEAP {
            let owned_type = syn::parse_str::<syn::Type>(owned_type).expect("Failed to parse type");

            for pg_type in pg_types.iter() {
                m.upsert_db_type(pg_type, RsType::new(owned_type.clone(), None, true));
            }
        }

        for (owned_type, slice_type, pg_types) in DEFAULT_TYPE {
            let owned_type = syn::parse_str::<syn::Type>(owned_type).expect("Failed to parse type");
            let slice_type = slice_type
                .map(|s| syn::parse_str::<syn::Type>(s).expect("Failed to parse slice type"));

            for pg_type in pg_types.iter() {
                m.upsert_db_type(
                    pg_type,
                    RsType::new(owned_type.clone(), slice_type.clone(), false),
                );
            }
        }

        m
    }
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

    fn upsert_db_type(&mut self, db_type: &str, mut rs_type: RsType) -> Option<RsType> {
        if let Some(exist) = self.type_map.get_mut(db_type) {
            core::mem::swap(exist, &mut rs_type);
            Some(rs_type)
        } else {
            self.type_map.insert(db_type.to_string(), rs_type);
            None
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct Rusqlite;

impl<'de> serde::Deserialize<'de> for Rusqlite {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.trim() {
            "rusqlite" => Ok(Self),
            _ => Err(serde::de::Error::custom(format!(
                "`{s}` is unsupported crate."
            ))),
        }
    }
}

impl Rusqlite {
    fn returning_row(row: &crate::ReturningRows) -> proc_macro2::TokenStream {
        let row_struct = super::make_return_row(row);

        let ident = row.struct_ident();
        let arg_ident = quote::format_ident!("row");

        let from_fields = row.fields.iter().enumerate().map(|(idx, field)| {
            let field_ident = &field.name;
            let literal = proc_macro2::Literal::usize_unsuffixed(idx);
            quote::quote! {#field_ident:#arg_ident.get(#literal)?}
        });

        let from_tt = quote::quote! {
            impl #ident {
                pub fn from_row(#arg_ident: &rusqlite::Row)->rusqlite::Result<Self>{
                    Ok(Self{
                        #(#from_fields,)*
                    })
                }
            }
        };

        quote::quote! {
            #row_struct
            #from_tt
        }
    }
}

impl DbCrate for Rusqlite {
    fn type_map(&self) -> Box<dyn crate::query::TypeMapper> {
        Box::new(SqliteTypeMap::new())
    }

    fn init(&self) -> proc_macro2::TokenStream {
        quote::quote! {
            pub trait RusqliteClient {
                fn prepare(&self, sql: &str) -> rusqlite::Result<rusqlite::Statement<'_>>;
            }

            impl RusqliteClient for rusqlite::Connection {
                fn prepare(&self, sql: &str) -> rusqlite::Result<rusqlite::Statement<'_>> {
                    self.prepare(sql)
                }
            }

            impl RusqliteClient for rusqlite::Transaction<'_> {
                fn prepare(&self, sql: &str) -> rusqlite::Result<rusqlite::Statement<'_>> {
                    rusqlite::Connection::prepare(&self, sql)
                }
            }
        }
    }

    fn defined_enum(&self, _enum_type: &crate::query::DbEnum) -> proc_macro2::TokenStream {
        quote::quote! {
            compile_error!("sqlite do not support enum")
        }
    }

    fn generate_query(
        &self,
        row: &crate::query::ReturningRows,
        query: &crate::query::Query,
    ) -> proc_macro2::TokenStream {
        let row_tt = Self::returning_row(row);
        let query_ast = super::QueryAst::new(query);
        let builder_tt = query_ast.make_builder();

        let query_fns = match query.annotation {
            Annotation::One => {
                let row_ident = row.struct_ident();
                quote::quote! {
                    pub fn query_one(&self, client: &impl RusqliteClient)->rusqlite::Result<#row_ident>{
                            Self::prepare(client)?
                                .query_row(self.as_params(), #row_ident::from_row)
                    }
                    pub fn query_opt(&self, client: &impl RusqliteClient)->rusqlite::Result<Option<#row_ident>>{
                            Self::prepare(client)?
                                .query_map(self.as_params(), #row_ident::from_row)?
                                .next()
                                .transpose()
                    }
                }
            }
            Annotation::Many => {
                let row_ident = row.struct_ident();
                quote::quote! {
                    pub fn query_many(&self, client: &impl RusqliteClient)->rusqlite::Result<Vec<#row_ident>>{
                        Self::prepare(client)?
                            .query_map(self.as_params(), #row_ident::from_row)?
                            .collect()
                    }
                }
            }
            Annotation::Exec | Annotation::ExecResult | Annotation::ExecRows => {
                quote::quote! {
                    pub fn execute(&self, client: &impl RusqliteClient)->rusqlite::Result<usize>{
                        Self::prepare(client)?
                            .execute(self.as_params())
                    }
                }
            }
            _ => {
                quote::quote! {}
            }
        };

        let fetch_tt = {
            let query_str = query.query_str();
            let struct_ident = &query_ast.ident;
            let imp_ident = if query_ast.need_lifetime() {
                let lifetime = &query_ast.lifetime;
                quote::quote! {<#lifetime> #struct_ident<#lifetime>}
            } else {
                quote::quote! {#struct_ident}
            };

            let param_types = query_ast
                .fields
                .iter()
                .map(|f| f.typ.to_param_tokens(&query_ast.lifetime));
            let params = query.fields.iter().map(|f| {
                let name = &f.name;
                quote::quote! {self.#name}
            });

            quote::quote! {
                impl #imp_ident {
                    pub const QUERY:&'static str = #query_str;
                    #query_fns

                    pub fn prepare<'conn>(client:&'conn impl RusqliteClient)->rusqlite::Result<rusqlite::Statement<'conn>>{
                        client.prepare(Self::QUERY)
                    }

                    pub fn as_params(&self) -> (#(#param_types,)*) {
                        ( #(#params,)* )
                    }
                }
            }
        };

        quote::quote! {
            #row_tt
            #query_ast
            #fetch_tt
            #builder_tt
        }
    }
}
