use crate::query::{DbEnum, DbTypeMap, Query, ReturningRows};

pub(crate) mod postgres;
pub(crate) mod sqlx;

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(untagged)]
pub enum SupportedDbCrate {
    Postgres(postgres::Postgres),
    Sqlx(sqlx::Sqlx),
}

pub(crate) trait DbCrate {
    // Generate DB type to Rust type mapping
    fn db_type_map(&self) -> DbTypeMap;
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
