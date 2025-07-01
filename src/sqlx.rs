use crate::{
    DbCrate,
    query::{DbEnum, Query, ReturningRows},
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

impl DbCrate for Sqlx {
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
        quote::quote! {}
    }
}
