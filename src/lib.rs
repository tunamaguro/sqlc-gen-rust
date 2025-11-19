use convert_case::{Case, Casing as _};
use prost::Message as _;
use std::collections::HashMap;
use std::io::{Read as _, Write};
use syn::parse::Parser as _;

pub(crate) mod plugin {
    include!(concat!(env!("OUT_DIR"), "/plugin.rs"));
}
pub(crate) mod db_crates;
pub(crate) mod query;
use db_crates::DbCrate as _;
use query::{Query, ReturningRows, RsType, collect_enums};
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
            "{}:{} , at {}:{}",
            layer,
            self,
            location.file(),
            location.line()
        );
        buf.push(message);
    }

    fn next(&self) -> Option<&dyn StackError> {
        match self {
            Error::QueryError { source, .. } => Some(source),
            _ => None,
        }
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

#[derive(Debug, Clone, serde::Deserialize, Default)]
#[serde(default)]
struct OverrideType {
    /// Override db type
    db_type: Option<String>,
    /// Override column name
    column: Option<String>,
    /// Override Rust type
    rs_type: String,
    /// Rust type's slice if have
    rs_slice: Option<String>,
    /// Marker is copy cheap
    copy_cheap: bool,
}

#[derive(Debug, Clone, Default)]
struct FieldAnnotationOverride {
    column: String,
    attributes: Vec<String>,
}

#[derive(Debug, Clone, Default)]
struct StructAnnotationOverride {
    struct_attributes: Vec<String>,
    field_attributes: Vec<FieldAnnotationOverride>,
}

#[derive(Debug, Clone, serde::Deserialize, Default)]
#[serde(default)]
struct StructAnnotationConfig {
    #[serde(alias = "struct")]
    struct_attributes: Vec<String>,
    fields: HashMap<String, Vec<String>>,
}

impl StructAnnotationConfig {
    fn to_override(&self) -> StructAnnotationOverride {
        let field_attributes = self
            .fields
            .iter()
            .map(|(column, attributes)| FieldAnnotationOverride {
                column: column.clone(),
                attributes: attributes.clone(),
            })
            .collect();
        StructAnnotationOverride {
            struct_attributes: self.struct_attributes.clone(),
            field_attributes,
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize, Default)]
#[serde(default)]
struct AnnotationBundleConfig {
    row: Option<StructAnnotationConfig>,
    params: Option<StructAnnotationConfig>,
}

#[derive(Debug, Clone, serde::Deserialize, Default)]
#[serde(default)]
struct EnumAnnotationConfig {
    #[serde(alias = "struct")]
    struct_attributes: Vec<String>,
    variants: HashMap<String, Vec<String>>,
}

fn parse_attribute_list(values: &[String]) -> Result<Vec<syn::Attribute>, Error> {
    let mut result = Vec::new();
    for attr in values {
        let parsed = syn::Attribute::parse_outer
            .parse_str(attr)
            .map_err(|e| Error::any(Box::new(e)))?;
        result.extend(parsed);
    }
    Ok(result)
}

fn apply_struct_annotation_override(
    override_data: &StructAnnotationOverride,
    db_names: &[String],
    struct_attributes: &mut Vec<syn::Attribute>,
    field_attributes: &mut [Vec<syn::Attribute>],
) -> Result<(), Error> {
    let struct_attrs = parse_attribute_list(&override_data.struct_attributes)?;
    struct_attributes.extend(struct_attrs);

    for field_override in &override_data.field_attributes {
        let attrs = parse_attribute_list(&field_override.attributes)?;
        for (idx, db_name) in db_names.iter().enumerate() {
            let mut matches = db_name == &field_override.column;
            if !matches {
                if let Some(col) = db_name.rsplit('.').next() {
                    matches = col == field_override.column;
                }
            }
            if matches {
                field_attributes[idx].extend(attrs.iter().cloned());
            }
        }
    }

    Ok(())
}

fn apply_row_annotation_override(
    row: &mut ReturningRows,
    override_data: &StructAnnotationOverride,
) -> Result<(), Error> {
    apply_struct_annotation_override(
        override_data,
        &row.column_db_names,
        &mut row.struct_attributes,
        &mut row.field_attributes,
    )
}

fn apply_query_param_annotation_override(
    query: &mut Query,
    override_data: &StructAnnotationOverride,
) -> Result<(), Error> {
    apply_struct_annotation_override(
        override_data,
        &query.param_db_names,
        &mut query.struct_attributes,
        &mut query.field_attributes,
    )
}

fn apply_enum_annotation_config(
    enum_type: &mut query::DbEnum,
    config: &EnumAnnotationConfig,
) -> Result<(), Error> {
    let struct_attrs = parse_attribute_list(&config.struct_attributes)?;
    enum_type.struct_attributes.extend(struct_attrs);

    for (variant_name, attributes) in &config.variants {
        let attrs = parse_attribute_list(attributes)?;
        for (idx, value) in enum_type.values.iter().enumerate() {
            if value == variant_name {
                enum_type.variant_attributes[idx].extend(attrs.iter().cloned());
            }
        }
    }

    Ok(())
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
struct Config {
    output: String,
    db_crate: db_crates::SupportedDbCrate,
    overrides: Vec<OverrideType>,
    debug: bool,
    enum_derives: Vec<String>,
    row_derives: Vec<String>,
    defaults: AnnotationBundleConfig,
    query_overrides: HashMap<String, AnnotationBundleConfig>,
    enum_defaults: HashMap<String, EnumAnnotationConfig>,
    row_overrides: HashMap<String, StructAnnotationConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            output: "queries.rs".into(),
            db_crate: Default::default(),
            overrides: Default::default(),
            debug: false,
            enum_derives: Vec::new(),
            row_derives: Vec::new(),
            defaults: AnnotationBundleConfig::default(),
            query_overrides: HashMap::new(),
            enum_defaults: HashMap::new(),
            row_overrides: HashMap::new(),
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
    let config = if request.plugin_options.is_empty() {
        Config::default()
    } else {
        Config::from_option(&request.plugin_options)?
    };

    let mut db_type = config.db_crate.db_type_map();
    for override_type in config.overrides {
        let owned_type = syn::parse_str::<syn::Type>(&override_type.rs_type)
            .map_err(|e| Error::any(e.into()))?;
        let slice_type = override_type
            .rs_slice
            .map(|s| syn::parse_str::<syn::Type>(&s))
            .transpose()
            .map_err(|e| Error::any(e.into()))?;

        match (&override_type.db_type, &override_type.column) {
            (None, Some(column)) => {
                db_type.insert_column_type(
                    column,
                    RsType::new(
                        owned_type.clone(),
                        slice_type.clone(),
                        override_type.copy_cheap,
                    ),
                );
            }
            (Some(db_type_name), None) => {
                db_type.insert_db_type(
                    db_type_name,
                    RsType::new(
                        owned_type.clone(),
                        slice_type.clone(),
                        override_type.copy_cheap,
                    ),
                );
            }

            (Some(_), Some(_)) => {
                let message = "Cannot override both db_type and column name at the same time.";
                return Err(Error::any(message.into()));
            }
            (None, None) => {
                let message = "Must override either db_type or column name.";
                return Err(Error::any(message.into()));
            }
        }
    }

    let enum_derives = config
        .enum_derives
        .iter()
        .map(|d| syn::parse_str::<syn::Path>(d).map_err(|e| Error::any(e.into())))
        .collect::<Result<Vec<_>, _>>()?;
    let row_derives = config
        .row_derives
        .iter()
        .map(|d| syn::parse_str::<syn::Path>(d).map_err(|e| Error::any(e.into())))
        .collect::<Result<Vec<_>, _>>()?;

    let mut defined_enums = request
        .catalog
        .as_ref()
        .map(collect_enums)
        .unwrap_or_default();

    for e in &mut defined_enums {
        e.derives = enum_derives.clone();
    }

    if !config.enum_defaults.is_empty() {
        let mut enum_index = HashMap::new();
        for (idx, e) in defined_enums.iter().enumerate() {
            enum_index.insert(e.name.clone(), idx);
        }

        for (name, enum_config) in &config.enum_defaults {
            if let Some(&idx) = enum_index.get(name) {
                apply_enum_annotation_config(&mut defined_enums[idx], enum_config)?;
            }
        }
    }

    for e in &defined_enums {
        db_type.insert_db_type(
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

    let mut returning_rows = request
        .queries
        .iter()
        .map(|q| ReturningRows::from_query(db_type.as_ref(), q))
        .collect::<Result<Vec<_>, _>>()?;
    for r in &mut returning_rows {
        r.derives = row_derives.clone();
    }
    let mut queries = request
        .queries
        .iter()
        .map(|q| Query::from_query(db_type.as_ref(), q))
        .collect::<Result<Vec<_>, _>>()?;

    let row_defaults_override = config.defaults.row.as_ref().map(|cfg| cfg.to_override());
    if let Some(ref override_data) = row_defaults_override {
        for row in &mut returning_rows {
            apply_row_annotation_override(row, override_data)?;
        }
    }

    let params_defaults_override = config.defaults.params.as_ref().map(|cfg| cfg.to_override());
    if let Some(ref override_data) = params_defaults_override {
        for query in &mut queries {
            apply_query_param_annotation_override(query, override_data)?;
        }
    }

    if !config.query_overrides.is_empty() {
        let mut query_index = HashMap::new();
        for (idx, query) in queries.iter().enumerate() {
            query_index.insert(query.query_name.clone(), idx);
        }

        for (query_name, override_config) in &config.query_overrides {
            if let Some(&idx) = query_index.get(query_name) {
                if let Some(row_cfg) = override_config.row.as_ref() {
                    let row_override = row_cfg.to_override();
                    apply_row_annotation_override(&mut returning_rows[idx], &row_override)?;
                }
                if let Some(params_cfg) = override_config.params.as_ref() {
                    let params_override = params_cfg.to_override();
                    apply_query_param_annotation_override(&mut queries[idx], &params_override)?;
                }
            }
        }
    }

    if !config.row_overrides.is_empty() {
        let mut row_index = HashMap::new();
        for (idx, row) in returning_rows.iter().enumerate() {
            row_index.insert(row.struct_ident().to_string(), idx);
        }

        for (row_name, override_config) in &config.row_overrides {
            if let Some(&idx) = row_index.get(row_name) {
                let row_override = override_config.to_override();
                apply_row_annotation_override(&mut returning_rows[idx], &row_override)?;
            }
        }
    }

    let enums_ts = defined_enums
        .iter()
        .map(|e| config.db_crate.defined_enum(e))
        .collect::<Vec<_>>();
    let enums_tt = quote::quote! {#(#enums_ts)*};

    let queries_ts = returning_rows
        .iter()
        .zip(queries.iter())
        .map(|(r, q)| config.db_crate.generate_query(r, q))
        .collect::<Vec<_>>();
    let queries_tt = quote::quote! {#(#queries_ts)*};

    let init_tt = config.db_crate.init();
    let tt = quote::quote! {
        #init_tt
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

    if config.debug {
        let req_txt = format!("{request:#?}");
        response.files.push(plugin::File {
            name: "input.txt".into(),
            contents: req_txt.into_bytes(),
        });

        response.files.push(plugin::File {
            name: "input.bin".into(),
            contents: buffer,
        });
    }

    let serialized_response = serialize_codegen_response(&response);

    std::io::stdout().write_all(&serialized_response)?;

    Ok(())
}
