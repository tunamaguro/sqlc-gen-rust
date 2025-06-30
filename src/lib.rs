use convert_case::{Case, Casing as _};
use prost::Message as _;
use std::io::{Read as _, Write};

pub(crate) mod plugin {
    include!(concat!(env!("OUT_DIR"), "/plugin.rs"));
}

pub(crate) mod postgres;
pub(crate) mod query;
use postgres::Postgres;
use query::{DbEnum, DbTypeMap, Query, ReturningRows, RsType, collect_enums};

pub trait StackError: std::error::Error {
    /// format each error stack
    fn format_stack(&self, layer: usize, buf: &mut Vec<String>);
    /// next error
    fn next(&self) -> Option<&dyn StackError>;

    /// last error
    fn last(&self) -> &dyn StackError
    where
        Self: Sized,
    {
        let Some(mut result) = self.next() else {
            return self;
        };
        while let Some(err) = result.next() {
            result = err;
        }
        result
    }
}

pub(crate) trait StackErrorResult<T, E> {
    fn stacked(self) -> Result<T, E>;
}

pub trait StackErrorExt: StackError {
    fn stack_error(&self) -> Vec<String>
    where
        Self: Sized,
    {
        let mut buf = Vec::new();
        let mut layer = 0;
        let mut current: &dyn StackError = self;

        loop {
            current.format_stack(layer, &mut buf);
            match current.next() {
                Some(next) => {
                    current = next;
                    layer += 1;
                }
                None => break,
            }
        }

        buf
    }
}

impl<E: StackError> StackErrorExt for E {}

#[derive(Debug)]
pub enum Error {
    Io {
        source: std::io::Error,
        location: &'static std::panic::Location<'static>,
    },
    ProstDecode {
        source: prost::DecodeError,
        location: &'static std::panic::Location<'static>,
    },
    Json {
        source: serde_json::Error,
        location: &'static std::panic::Location<'static>,
    },
    QueryError {
        source: query::QueryError,
        location: &'static std::panic::Location<'static>,
    },
    Any {
        source: Box<dyn std::error::Error + 'static>,
        location: &'static std::panic::Location<'static>,
    },
}

impl Error {
    fn location(&self) -> &'static std::panic::Location<'static> {
        match self {
            Error::Io { location, .. } => location,
            Error::ProstDecode { location, .. } => location,
            Error::Json { location, .. } => location,
            Error::Any { location, .. } => location,
            Error::QueryError { location, .. } => location,
        }
    }

    #[track_caller]
    fn any(source: Box<dyn std::error::Error + 'static>) -> Self {
        Error::Any {
            source,
            location: std::panic::Location::caller(),
        }
    }
}

impl From<std::io::Error> for Error {
    #[track_caller]
    fn from(value: std::io::Error) -> Self {
        Self::Io {
            source: value,
            location: std::panic::Location::caller(),
        }
    }
}

impl From<prost::DecodeError> for Error {
    #[track_caller]
    fn from(value: prost::DecodeError) -> Self {
        Self::ProstDecode {
            source: value,
            location: std::panic::Location::caller(),
        }
    }
}

impl From<serde_json::Error> for Error {
    #[track_caller]
    fn from(value: serde_json::Error) -> Self {
        Self::Json {
            source: value,
            location: std::panic::Location::caller(),
        }
    }
}

impl From<query::QueryError> for Error {
    #[track_caller]
    fn from(value: query::QueryError) -> Self {
        Self::QueryError {
            source: value,
            location: std::panic::Location::caller(),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io { source, .. } => source.fmt(f),
            Error::ProstDecode { source, .. } => source.fmt(f),
            Error::Json { source, .. } => source.fmt(f),
            Error::Any { source, .. } => source.fmt(f),
            Error::QueryError { source, .. } => source.fmt(f),
        }
    }
}

impl StackError for Error {
    fn format_stack(&self, layer: usize, buf: &mut Vec<String>) {
        let location = self.location();
        let message = format!(
            "{}:{}, at {}:{}",
            layer,
            self,
            location.file(),
            location.line()
        );
        buf.push(message);
    }

    fn next(&self) -> Option<&dyn StackError> {
        None
    }

    fn last(&self) -> &dyn StackError
    where
        Self: Sized,
    {
        let Some(mut result) = self.next() else {
            return self;
        };
        while let Some(err) = result.next() {
            result = err;
        }
        result
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io { source, .. } => Some(source),
            Error::ProstDecode { source, .. } => Some(source),
            Error::Json { source, .. } => Some(source),
            Error::QueryError { source, .. } => Some(source),
            Error::Any { source, .. } => Some(source.as_ref()),
        }
    }
}

fn deserialize_codegen_request(data: &[u8]) -> Result<plugin::GenerateRequest, prost::DecodeError> {
    plugin::GenerateRequest::decode(data)
}

fn serialize_codegen_response(response: &plugin::GenerateResponse) -> Vec<u8> {
    response.encode_to_vec()
}

pub(crate) fn normalize_str(value: &str) -> String {
    use regex_lite::Regex;
    use std::sync::LazyLock;
    static IDENT_PATTERN: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r#"[^a-zA-Z0-9_]"#).unwrap());

    let value = value.replace("-", "_");
    let value = value.replace(":", "_");
    let value = value.replace("/", "_");
    let value = IDENT_PATTERN.replace_all(&value, "");
    value.to_string()
}

pub(crate) fn value_ident(ident: &str) -> syn::Ident {
    let ident = normalize_str(ident).to_case(Case::Pascal);
    quote::format_ident!("{}", ident)
}

pub(crate) fn field_ident(ident: &str) -> syn::Ident {
    let ident = normalize_str(ident).to_case(Case::Snake);
    quote::format_ident!("{}", ident)
}

pub(crate) trait DbCrate {
    /// Generate returning row
    fn returning_row(&self, row: &ReturningRows) -> proc_macro2::TokenStream;
    /// Generate enum
    fn defined_enum(&self, enum_type: &DbEnum) -> proc_macro2::TokenStream;
    /// Generate query fn
    fn call_query(&self, row: &ReturningRows, query: &Query) -> proc_macro2::TokenStream;
}

#[derive(Debug, Clone, serde::Deserialize, Default)]
struct OverrideType {
    /// Override db type
    db_type: String,
    /// Override Rust type
    rs_type: String,
    /// Rust type's slice if have
    rs_slice: Option<String>,
    /// Marker is copy cheap
    copy_cheap: bool,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
struct Config {
    output: String,
    db_crate: Postgres,
    overrides: Vec<OverrideType>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            output: "queries.rs".into(),
            db_crate: Default::default(),
            overrides: Default::default(),
        }
    }
}

impl Config {
    fn from_option(buf: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(buf)
    }
}

fn generate_comment(sqlc_version: &str) -> String {
    format!(
        r"//! Code generated by {}. SHOULD NOT EDIT.
//! sqlc version: {}
//! {} version: v{}",
        env!("CARGO_PKG_NAME"),
        sqlc_version,
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
    )
}

pub fn try_main() -> Result<(), Error> {
    let mut stdin = std::io::stdin().lock();
    let mut buffer = Vec::new();
    stdin.read_to_end(&mut buffer)?;

    let request = deserialize_codegen_request(&buffer)?;
    let config = Config::from_option(&request.plugin_options)?;

    let mut db_type = DbTypeMap::new_for_postgres();
    for override_type in config.overrides {
        let owned_type = syn::parse_str::<syn::Type>(&override_type.rs_type)
            .map_err(|e| Error::any(e.into()))?;
        let slice_type = override_type
            .rs_slice
            .map(|s| syn::parse_str::<syn::Type>(&s))
            .transpose()
            .map_err(|e| Error::any(e.into()))?;
        db_type.insert_type(
            &override_type.db_type,
            RsType::new(owned_type, slice_type, override_type.copy_cheap),
        );
    }

    let defined_enums = request
        .catalog
        .as_ref()
        .map(collect_enums)
        .unwrap_or_default();

    for e in &defined_enums {
        db_type.insert_type(
            &e.name,
            RsType::new(
                syn::TypePath {
                    qself: None,
                    path: e.ident().clone().into(),
                }
                .into(),
                None,
                true,
            ),
        );
    }

    let returning_rows = request
        .queries
        .iter()
        .map(|q| ReturningRows::from_query(&db_type, q))
        .collect::<Result<Vec<_>, _>>()?;
    let queries = request
        .queries
        .iter()
        .map(|q| Query::from_query(&db_type, q))
        .collect::<Result<Vec<_>, _>>()?;

    let enums_ts = defined_enums
        .iter()
        .map(|e| config.db_crate.defined_enum(e))
        .collect::<Vec<_>>();
    let enums_tt = quote::quote! {#(#enums_ts)*};

    let queries_ts = returning_rows
        .iter()
        .zip(queries.iter())
        .map(|(r, q)| {
            let row_tt = config.db_crate.returning_row(r);
            let query_tt = config.db_crate.call_query(r, q);

            quote::quote! {
                #row_tt
                #query_tt
            }
        })
        .collect::<Vec<_>>();
    let queries_tt = quote::quote! {#(#queries_ts)*};

    let tt = quote::quote! {
        #enums_tt
        #queries_tt
    };
    let mut response = plugin::GenerateResponse::default();
    let ast = syn::parse2(tt).map_err(|e| Error::any(e.into()))?;
    let contents = format!(
        "{}\n\n{}",
        generate_comment(&request.sqlc_version),
        prettyplease::unparse(&ast)
    );
    let query_file = plugin::File {
        name: config.output,
        contents: contents.into(),
    };
    response.files.push(query_file);

    let serialized_response = serialize_codegen_response(&response);

    std::io::stdout().write_all(&serialized_response)?;

    Ok(())
}
