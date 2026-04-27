use crate::query::{self, DbEnum, DbTypeMap, Query, ReturningRows, TypeMapper};

mod postgres;
mod rusqlite;
mod sqlx;

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(untagged)]
pub enum SupportedDbCrate {
    Postgres(postgres::Postgres),
    Sqlx(sqlx::Sqlx),
    Rusqlite(rusqlite::Rusqlite),
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
            Self::Postgres(postgres) => postgres.type_map(),
            Self::Sqlx(sqlx) => sqlx.type_map(),
            Self::Rusqlite(rusqlite) => rusqlite.type_map(),
        }
    }

    fn init(&self) -> proc_macro2::TokenStream {
        match self {
            Self::Postgres(postgres) => postgres.init(),
            Self::Sqlx(sqlx) => sqlx.init(),
            Self::Rusqlite(rusqlite) => rusqlite.init(),
        }
    }

    fn defined_enum(&self, enum_type: &DbEnum) -> proc_macro2::TokenStream {
        match self {
            Self::Postgres(postgres) => postgres.defined_enum(enum_type),
            Self::Sqlx(sqlx) => sqlx.defined_enum(enum_type),
            Self::Rusqlite(rusqlite) => rusqlite.defined_enum(enum_type),
        }
    }

    fn generate_query(&self, row: &ReturningRows, query: &Query) -> proc_macro2::TokenStream {
        match self {
            Self::Postgres(postgres) => postgres.generate_query(row, query),
            Self::Sqlx(sqlx) => sqlx.generate_query(row, query),
            Self::Rusqlite(rusqlite) => rusqlite.generate_query(row, query),
        }
    }
}

impl Default for SupportedDbCrate {
    fn default() -> Self {
        Self::Postgres(postgres::Postgres::Tokio)
    }
}

fn make_return_row(row: &query::ReturningRows) -> proc_macro2::TokenStream {
    let ident = &row.struct_ident();
    let row_attribute = &row.attributes;
    let fields = row.fields.iter().map(|field| {
        let field_name = &field.name;
        let field_typ = field.typ.to_row_tokens();
        let attribute = &field.attribute;
        quote::quote! {
            #attribute
            pub #field_name:#field_typ
        }
    });
    quote::quote! {
        #row_attribute
        pub struct #ident {
            #(#fields,)*
        }
    }
}

enum DataBaseKind {
    Postgres,
    MySql,
    Sqlite,
}

struct QueryAst<'a> {
    pub ident: syn::Ident,
    pub lifetime: syn::Lifetime,
    query: &'a Query,
    kind: DataBaseKind,
}

impl<'a> QueryAst<'a> {
    fn new(query: &'a Query, kind: DataBaseKind) -> Self {
        let ident = crate::value_ident(&query.query_name);
        let lifetime = syn::Lifetime::new("'a", proc_macro2::Span::call_site());
        Self {
            ident,
            lifetime,
            query,
            kind,
        }
    }

    fn fields(&self) -> impl Iterator<Item = &query::ColumnField> {
        self.query.fields.iter()
    }

    fn need_lifetime(&self) -> bool {
        self.fields().any(|f| f.typ.need_lifetime())
    }

    fn need_expand_query(&self) -> bool {
        if matches!(self.kind, DataBaseKind::Postgres) {
            return false;
        }

        self.fields().any(|f| f.typ.is_array())
    }

    fn make_builder_setter(&self) -> proc_macro2::TokenStream {
        use quote::ToTokens;
        let num_params = self.query.fields.len();
        let fields_tuple = (0..num_params)
            .map(|_| quote::quote! {()})
            .collect::<Vec<_>>();

        let lifetime = &self.lifetime;
        let struct_ident = &self.ident;
        let builder_ident = crate::value_ident(&format!("{}Builder", struct_ident));

        let field_list = self
            .fields()
            .map(|f| &f.name)
            .map(|n| n.to_token_stream())
            .collect::<Vec<_>>();

        let typ_list = self
            .fields()
            .map(|f| &f.typ)
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
                .fields()
                .map(|f| &f.name)
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

        quote::quote! {
            #impl_struct_tt
            #builder_tt
            #builder_setter_tt
        }
    }

    fn make_builder_build(&self) -> proc_macro2::TokenStream {
        use quote::ToTokens;

        let lifetime = &self.lifetime;
        let struct_ident = &self.ident;
        let builder_ident = crate::value_ident(&format!("{}Builder", struct_ident));

        let field_list = self
            .fields()
            .map(|f| &f.name)
            .map(|n| n.to_token_stream())
            .collect::<Vec<_>>();

        let typ_list = self
            .fields()
            .map(|f| &f.typ)
            .map(|typ| typ.to_param_tokens(lifetime))
            .collect::<Vec<_>>();

        let builder_build_tt = {
            let build_struct = if self.need_lifetime() {
                quote::quote! {#struct_ident<#lifetime>}
            } else {
                quote::quote! {#struct_ident}
            };

            if self.need_expand_query() {
                let query_ident = quote::format_ident!("__query");

                let query_builder =
                    self.query
                        .fields
                        .iter()
                        .filter(|f| f.typ.is_array())
                        .map(|f| {
                            let name = &f.name;
                            let marker = format!("/*SLICE:{}*/?", name);
                            quote::quote! {
                                let #query_ident = match #name.len(){
                                    0 => {
                                        #query_ident.replace(#marker, "NULL")
                                    }
                                    1 => {
                                        #query_ident.replace(#marker, "?")
                                    }
                                    n => {
                                        let to = core::iter::once("?").chain(core::iter::repeat(",?").take(n - 1)).collect::<String>();
                                        #query_ident.replace(#marker, &to)
                                    }
                                };
                            }
                        });

                quote::quote! {
                      impl <#lifetime> #builder_ident<#lifetime,(#(#typ_list,)*)>{
                        pub fn build(self)->#build_struct{
                            let (#(#field_list,)*) = self.fields;

                            let #query_ident = #struct_ident::QUERY;
                            #(#query_builder)*

                            #struct_ident{
                                #(#field_list,)*
                                __query: #query_ident.into()
                            }
                        }
                    }
                }
            } else {
                quote::quote! {
                      impl <#lifetime> #builder_ident<#lifetime,(#(#typ_list,)*)>{
                        pub fn build(self)->#build_struct{
                            let (#(#field_list,)*) = self.fields;
                            #struct_ident{
                                #(#field_list,)*
                            }
                        }
                    }
                }
            }
        };

        builder_build_tt
    }

    fn make_builder(&self) -> proc_macro2::TokenStream {
        let setter_tt = self.make_builder_setter();
        let build_tt = self.make_builder_build();

        quote::quote! {
            #setter_tt
            #build_tt
        }
    }
}

impl<'a> quote::ToTokens for QueryAst<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let fields = self.fields().map(|f| {
            let name = &f.name;
            let typ = f.typ.to_param_tokens(&self.lifetime);
            quote::quote! {#name:#typ}
        });
        let ident = &self.ident;
        let lifetime = &self.lifetime;

        let query_str = self.query.query_str();

        let tt = match (self.need_lifetime(), !self.query.fields.is_empty()) {
            (true, _) => {
                if self.need_expand_query() {
                    quote::quote! {
                        pub struct #ident<#lifetime>{
                            #(#fields,)*
                            __query: String
                        }

                        impl <#lifetime> #ident<#lifetime>{
                            pub const QUERY : &'static str = #query_str;
                            pub fn query_str(&self)->&str{
                                &self.__query
                            }
                        }
                    }
                } else {
                    quote::quote! {
                        pub struct #ident<#lifetime>{
                            #(#fields,)*
                        }

                        impl <#lifetime> #ident<#lifetime>{
                            pub const QUERY : &'static str = #query_str;
                            pub fn query_str(&self)->&str{
                                Self::QUERY
                            }
                        }
                    }
                }
            }
            (false, true) => {
                quote::quote! {
                    pub struct #ident{
                        #(#fields,)*
                    }

                    impl #ident {
                        pub const QUERY : &'static str = #query_str;
                        pub fn query_str(&self)->&str{
                            Self::QUERY
                        }
                    }
                }
            }
            (false, false) => {
                quote::quote! {
                    pub struct #ident;

                    impl #ident {
                        pub const QUERY : &'static str = #query_str;
                        pub fn query_str(&self)->&str{
                            Self::QUERY
                        }
                    }
                }
            }
        };

        tokens.extend(tt);
    }
}
