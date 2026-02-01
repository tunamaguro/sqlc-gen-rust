use crate::query::{self, DbEnum, DbTypeMap, Query, ReturningRows, RsColType, TypeMapper};

mod postgres;
mod sqlx;

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(untagged)]
pub enum SupportedDbCrate {
    Postgres(postgres::Postgres),
    Sqlx(sqlx::Sqlx),
}

pub(super) trait DbCrate {
    fn type_map(&self) -> Box<dyn TypeMapper>;

    // Generate DB type to Rust type mapping
    fn db_type_map(&self) -> DbTypeMap {
        DbTypeMap::from_dyn(self.type_map())
    }

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
    fn type_map(&self) -> Box<dyn TypeMapper> {
        match self {
            SupportedDbCrate::Postgres(postgres) => postgres.type_map(),
            SupportedDbCrate::Sqlx(sqlx) => sqlx.type_map(),
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

struct RowAst {
    pub ident: syn::Ident,
    pub fields: Vec<query::ColumnField>,
}

impl RowAst {
    fn new(row: &ReturningRows) -> Self {
        let fields = row.fields.clone();
        let ident = row.struct_ident();
        Self { ident, fields }
    }
}

impl quote::ToTokens for RowAst {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = &self.ident;
        let fields = self.fields.iter().map(|field| {
            let field_name = &field.column_name;
            let field_typ = field.typ.to_row_tokens();
            let attribute = &field.attribute;
            quote::quote! {
                #attribute
                pub #field_name:#field_typ
            }
        });

        let tt = quote::quote! {
            pub struct #ident {
                #(#fields,)*
            }
        };
        tokens.extend(tt);
    }
}

struct QueryAst {
    pub ident: syn::Ident,
    pub lifetime: syn::Lifetime,
    pub fields: Vec<(syn::Ident, RsColType)>,
}

impl QueryAst {
    fn new(query: &Query) -> Self {
        let ident = crate::value_ident(&query.query_name);
        let lifetime = syn::Lifetime::new("'a", proc_macro2::Span::call_site());
        let fields = query
            .param_names
            .iter()
            .cloned()
            .zip(query.param_types.iter().cloned())
            .collect();
        Self {
            ident,
            lifetime,
            fields,
        }
    }

    fn need_lifetime(&self) -> bool {
        self.fields.iter().any(|(_, typ)| typ.need_lifetime())
    }

    fn make_builder(&self) -> proc_macro2::TokenStream {
        use quote::ToTokens;
        let num_params = self.fields.len();
        let fields_tuple = (0..num_params)
            .map(|_| quote::quote! {()})
            .collect::<Vec<_>>();

        let lifetime = &self.lifetime;
        let struct_ident = &self.ident;
        let builder_ident = crate::value_ident(&format!("{}Builder", struct_ident));

        let field_list = self
            .fields
            .iter()
            .map(|(field, _)| field)
            .map(|n| n.to_token_stream())
            .collect::<Vec<_>>();
        let typ_list = self
            .fields
            .iter()
            .map(|(_, typ)| typ)
            .map(|typ| typ.to_param_tokens(lifetime))
            .collect::<Vec<_>>();

        // implement `GetXXX::builder`
        let impl_struct_tt = if self.need_lifetime() {
            quote::quote! {
                impl <#lifetime> #struct_ident<#lifetime>{
                    pub const fn builder()->#builder_ident<#lifetime, (#(#fields_tuple,)*)>{
                        #builder_ident{
                            fields: (#(#fields_tuple,)*),
                            _phantom: std::marker::PhantomData
                        }
                    }
                }
            }
        } else {
            quote::quote! {
                impl #struct_ident{
                    pub const fn builder()->#builder_ident<'static, (#(#fields_tuple,)*)>{
                        #builder_ident{
                            fields: (#(#fields_tuple,)*),
                            _phantom: std::marker::PhantomData
                        }
                    }
                }
            }
        };

        // implement `GetXXXBuilder`
        let builder_tt = quote::quote! {
            pub struct #builder_ident<#lifetime, Fields = (#(#fields_tuple,)*)>{
                fields: Fields,
                _phantom: std::marker::PhantomData<&#lifetime ()>
            }
        };

        // implement `GetXXXBuilder`
        let builder_setter_tt = {
            let typ_generics = self
                .fields
                .iter()
                .map(|(field, _)| field)
                .map(|n| crate::value_ident(&n.to_string()))
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

                result.extend(tt);
            }

            result
        };

        let builder_build_tt = {
            let build_struct = if self.need_lifetime() {
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
}

impl quote::ToTokens for QueryAst {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let fields = self.fields.iter().map(|(field, typ)| {
            let typ = typ.to_param_tokens(&self.lifetime);
            quote::quote! {#field:#typ}
        });
        let ident = &self.ident;
        let lifetime = &self.lifetime;

        let tt = match (self.need_lifetime(), !self.fields.is_empty()) {
            (true, _) => {
                quote::quote! {
                    pub struct #ident<#lifetime>{
                        #(#fields,)*
                    }
                }
            }
            (false, true) => {
                quote::quote! {
                    pub struct #ident{
                        #(#fields,)*
                    }
                }
            }
            (false, false) => {
                quote::quote! {
                    pub struct #ident;
                }
            }
        };

        tokens.extend(tt);
    }
}
