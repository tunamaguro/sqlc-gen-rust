use crate::query::{DbEnum, DbTypeMap, Query, ReturningRows};

mod postgres;
mod sqlx;

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(untagged)]
pub enum SupportedDbCrate {
    Postgres(postgres::Postgres),
    Sqlx(sqlx::Sqlx),
}

pub(super) trait DbCrate {
    // Generate DB type to Rust type mapping
    fn db_type_map(&self) -> DbTypeMap;

    /// Generate top `use` or `fn`
    fn init(&self) -> proc_macro2::TokenStream {
        proc_macro2::TokenStream::new()
    }

    /// Generate enum
    fn defined_enum(&self, enum_type: &DbEnum) -> proc_macro2::TokenStream;
    /// Generate returning row and query fn
    fn generate_query(&self, row: &ReturningRows, query: &Query) -> proc_macro2::TokenStream;
}

impl DbCrate for SupportedDbCrate {
    fn db_type_map(&self) -> DbTypeMap {
        match self {
            SupportedDbCrate::Postgres(postgres) => postgres.db_type_map(),
            SupportedDbCrate::Sqlx(sqlx) => sqlx.db_type_map(),
        }
    }

    fn init(&self) -> proc_macro2::TokenStream {
        match self {
            SupportedDbCrate::Postgres(postgres) => postgres.init(),
            SupportedDbCrate::Sqlx(sqlx) => sqlx.init(),
        }
    }

    fn defined_enum(&self, enum_type: &DbEnum) -> proc_macro2::TokenStream {
        match self {
            SupportedDbCrate::Postgres(postgres) => postgres.defined_enum(enum_type),
            SupportedDbCrate::Sqlx(sqlx) => sqlx.defined_enum(enum_type),
        }
    }

    fn generate_query(&self, row: &ReturningRows, query: &Query) -> proc_macro2::TokenStream {
        match self {
            SupportedDbCrate::Postgres(postgres) => postgres.generate_query(row, query),
            SupportedDbCrate::Sqlx(sqlx) => sqlx.generate_query(row, query),
        }
    }
}

impl Default for SupportedDbCrate {
    fn default() -> Self {
        Self::Postgres(postgres::Postgres::Tokio)
    }
}

/// Check query need lifetime
fn need_lifetime(query: &Query) -> bool {
    query.param_types.iter().any(|x| x.need_lifetime())
}

/// Generate type-state builder
fn create_builder(query: &Query) -> proc_macro2::TokenStream {
    use super::value_ident;
    use quote::ToTokens as _;

    let num_params = query.param_names.len();

    let fields_tuple = (0..num_params).fold(quote::quote! {}, |acc, _| quote::quote! {#acc (),});
    let need_lifetime = need_lifetime(query);
    let lifetime = syn::Lifetime::new("'a", proc_macro2::Span::call_site());
    let struct_ident = value_ident(&query.query_name);
    let builder_ident = value_ident(&format!("{}Builder", query.query_name));

    let field_list = query
        .param_names
        .iter()
        .map(|n| n.to_token_stream())
        .collect::<Vec<_>>();
    let typ_list = query
        .param_types
        .iter()
        .map(|typ| typ.to_param_tokens(&lifetime))
        .collect::<Vec<_>>();

    // implement `GetXXX::builder`
    let impl_struct_tt = if need_lifetime {
        quote::quote! {
            impl <#lifetime> #struct_ident<#lifetime>{
                pub const fn builder()->#builder_ident<#lifetime, (#fields_tuple)>{
                    #builder_ident{
                        fields: (#fields_tuple),
                        _phantom: std::marker::PhantomData
                    }
                }
            }
        }
    } else {
        quote::quote! {
            impl #struct_ident{
                pub const fn builder()->#builder_ident<'static, (#fields_tuple)>{
                    #builder_ident{
                        fields: (#fields_tuple),
                        _phantom: std::marker::PhantomData
                    }
                }
            }
        }
    };

    // implement `GetXXXBuilder`
    let builder_tt = quote::quote! {
        pub struct #builder_ident<#lifetime, Fields = (#fields_tuple)>{
            fields: Fields,
            _phantom: std::marker::PhantomData<&#lifetime ()>
        }
    };

    // implement `GetXXXBuilder`
    let builder_setter_tt = {
        let typ_generics = query
            .param_names
            .iter()
            .map(|n| value_ident(&n.to_string()))
            .collect::<Vec<_>>();

        let mut result = quote::quote! {};
        for (idx, (typ, name)) in typ_list.iter().zip(field_list.iter()).enumerate() {
            let (generics_head, rest) = typ_generics.split_at(idx);
            let generics_tail = if rest.is_empty() { &[] } else { &rest[1..] };

            let (field_head, rest) = field_list.split_at(idx);
            let field_tail = if rest.is_empty() { &[] } else { &rest[1..] };

            let tt = quote::quote! {
                impl <#lifetime,#(#generics_head,)* #(#generics_tail,)*> #builder_ident<#lifetime,(#(#generics_head,)* (), #(#generics_tail,)*)>{
                    pub fn #name(self, #name:#typ)->#builder_ident<#lifetime,(#(#generics_head,)* #typ, #(#generics_tail,)*)>{
                        let (#(#field_head,)* (), #(#field_tail,)*) = self.fields;
                        let _phantom = self._phantom;

                        #builder_ident{
                            fields: (#(#field_head,)* #name, #(#field_tail,)*),
                            _phantom
                        }
                    }
                }
            };

            result = quote::quote! {
                #result
                #tt
            }
        }

        result
    };

    let builder_build_tt = {
        let build_struct = if need_lifetime {
            quote::quote! {#struct_ident<#lifetime>}
        } else {
            quote::quote! {#struct_ident}
        };
        quote::quote! {
              impl <#lifetime> #builder_ident<#lifetime,(#(#typ_list,)*)>{
                pub const fn build(self)->#build_struct{
                    let (#(#field_list,)*) = self.fields;
                    #struct_ident{
                        #(#field_list,)*
                    }
                }
            }
        }
    };

    quote::quote! {
        #impl_struct_tt
        #builder_tt
        #builder_setter_tt
        #builder_build_tt
    }
}
