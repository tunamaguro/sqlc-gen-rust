use quote::ToTokens;

use crate::{StackError, StackErrorResult, field_ident, plugin, value_ident};

#[derive(Debug, Clone)]
pub enum QueryError {
    MissingColumnType {
        column_name: String,
        location: &'static std::panic::Location<'static>,
    },
    MissingParamColumn {
        param_number: i32,
        location: &'static std::panic::Location<'static>,
    },
    CannotMapType {
        message: String,
        location: &'static std::panic::Location<'static>,
    },
    UnknownAnnotation {
        annotation: String,
        location: &'static std::panic::Location<'static>,
    },
    Stacked {
        source: Box<Self>,
        location: &'static std::panic::Location<'static>,
    },
}

impl QueryError {
    #[track_caller]
    pub(crate) fn missing_column_type(column_name: String) -> Self {
        Self::MissingColumnType {
            column_name,
            location: std::panic::Location::caller(),
        }
    }

    #[track_caller]
    pub(crate) fn missing_param_column(param_number: i32) -> Self {
        Self::MissingParamColumn {
            param_number,
            location: std::panic::Location::caller(),
        }
    }

    #[track_caller]
    pub(crate) fn cannot_map_type(col_name: String, typ_name: String) -> Self {
        Self::CannotMapType {
            message: format!(
                "Cannot map type `{col_name}` of table `{typ_name}` to a Rust type. Consider add entry to overrides."
            ),
            location: std::panic::Location::caller(),
        }
    }

    #[track_caller]
    pub(crate) fn unknown_annotation(annotation: String) -> Self {
        Self::UnknownAnnotation {
            annotation,
            location: std::panic::Location::caller(),
        }
    }

    fn location(&self) -> &'static std::panic::Location<'static> {
        match self {
            QueryError::MissingColumnType { location, .. } => location,
            QueryError::MissingParamColumn { location, .. } => location,
            QueryError::CannotMapType { location, .. } => location,
            QueryError::UnknownAnnotation { location, .. } => location,
            QueryError::Stacked { location, .. } => location,
        }
    }
}

impl std::fmt::Display for QueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryError::MissingColumnType { column_name, .. } => {
                write!(f, "Column type not found for column: `{column_name}`")
            }
            QueryError::MissingParamColumn { param_number, .. } => {
                write!(
                    f,
                    "Parameter column not found for parameter #{param_number}"
                )
            }
            QueryError::UnknownAnnotation { annotation, .. } => {
                write!(f, "Unknown annotation `{annotation}` found")
            }
            QueryError::CannotMapType { message, .. } => message.fmt(f),
            QueryError::Stacked { source, .. } => source.fmt(f),
        }
    }
}

impl StackError for QueryError {
    fn format_stack(&self, layer: usize, buf: &mut Vec<String>) {
        let location = self.location();
        match self {
            QueryError::Stacked { .. } => {
                buf.push(format!(
                    "{}: at {}:{}",
                    layer,
                    location.file(),
                    location.line()
                ));
            }
            _ => {
                buf.push(format!(
                    "{}:{} , at {}:{}",
                    layer,
                    self,
                    location.file(),
                    location.line()
                ));
            }
        }
    }

    fn next(&self) -> Option<&dyn StackError> {
        match self {
            Self::Stacked { source, .. } => Some(source.as_ref()),
            _ => None,
        }
    }
}

impl std::error::Error for QueryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            QueryError::Stacked { source, .. } => Some(source.as_ref()),
            _ => None,
        }
    }
}

impl<T> StackErrorResult<T, QueryError> for Result<T, QueryError> {
    #[track_caller]
    fn stacked(self) -> Self {
        match self {
            Ok(v) => Ok(v),
            Err(err) => Err(QueryError::Stacked {
                source: err.into(),
                location: std::panic::Location::caller(),
            }),
        }
    }
}

#[derive(Clone)]
pub(crate) struct RsType {
    owned: syn::Type,
    slice: Option<syn::Type>,
    copy_cheap: bool,
}

impl RsType {
    pub(crate) fn new(owned: syn::Type, slice: Option<syn::Type>, copy_cheap: bool) -> Self {
        RsType {
            owned,
            slice,
            copy_cheap,
        }
    }

    /// 自己所有の型を返す
    pub(crate) fn owned(&self) -> proc_macro2::TokenStream {
        self.owned.to_token_stream()
    }

    /// スライスの型を返す。これに`&`をつけると参照になる
    pub(crate) fn slice(&self) -> proc_macro2::TokenStream {
        if let Some(ref slice) = self.slice {
            slice.to_token_stream()
        } else {
            self.owned()
        }
    }
}

#[derive(Clone)]
pub(crate) struct RsColType {
    rs_type: RsType,
    /// maybe dim
    dim: usize,
    /// col is optional
    optional: bool,
}
pub(crate) fn make_column_type(db_type: &plugin::Identifier) -> String {
    if !db_type.schema.is_empty() {
        format!("{}.{}", db_type.schema, db_type.name)
    } else {
        db_type.name.to_string()
    }
}

pub(crate) fn make_column_name(column: &plugin::Column) -> String {
    if let Some(table) = &column.table {
        format!("{}.{}", table.name, column.name)
    } else {
        column.name.clone()
    }
}

impl RsColType {
    pub(crate) fn new_with_type(
        db_type: &dyn DbTypeMapper,
        column: &plugin::Column,
    ) -> Result<Self, QueryError> {
        let rs_type = db_type.get_column_type(column).stacked()?;
        let dim = usize::try_from(column.array_dims).unwrap_or_default();
        let optional = !column.not_null;

        Ok(Self {
            rs_type,
            dim,
            optional,
        })
    }

    /// Convert to tokens for row struct
    pub(crate) fn to_row_tokens(&self) -> proc_macro2::TokenStream {
        let base_type = self.rs_type.owned();

        // 配列の次元数に応じてVecでラップ
        let mut wrapped_type = base_type;
        for _ in 0..self.dim {
            wrapped_type = quote::quote! { Vec<#wrapped_type> };
        }

        // optionalの場合はOptionでラップ
        if self.optional {
            quote::quote! { Option<#wrapped_type> }
        } else {
            wrapped_type
        }
    }

    pub(crate) fn need_lifetime(&self) -> bool {
        let is_slice = self.dim != 0;
        let copy_expensive = !self.rs_type.copy_cheap;

        is_slice || copy_expensive
    }

    /// Convert to tokens for function parameter struct
    pub(crate) fn to_param_tokens(&self, life_time: &syn::Lifetime) -> proc_macro2::TokenStream {
        let wrapped_type = match self.dim {
            0 => {
                let slice_type = self.rs_type.slice();
                quote::quote! {#slice_type}
            }
            _ => {
                let mut base_type = self.rs_type.owned();
                for _ in 1..self.dim {
                    base_type = quote::quote! {Vec<#base_type>}
                }

                quote::quote! {[#base_type]}
            }
        };

        match (self.need_lifetime(), self.optional) {
            (true, true) => {
                quote::quote! {Option<&#life_time #wrapped_type>}
            }
            (true, false) => {
                quote::quote! {&#life_time #wrapped_type}
            }
            (false, true) => {
                quote::quote! {Option<#wrapped_type>}
            }
            (false, false) => {
                quote::quote! {#wrapped_type}
            }
        }
    }
}

#[derive(Default)]
pub(crate) struct DbTypeMap {
    /// db_type to rust type
    typ_map: std::collections::BTreeMap<String, RsType>,
    /// column name to rust type
    column_map: std::collections::BTreeMap<String, RsType>,
}

pub(crate) trait DbTypeMapper {
    fn get_column_type(&self, column: &plugin::Column) -> Result<RsType, QueryError>;
    fn insert_db_type(&mut self, db_type: &str, rs_type: RsType);
    fn insert_column_type(&mut self, column_name: &str, rs_type: RsType);
}

impl DbTypeMapper for DbTypeMap {
    fn get_column_type(&self, column: &plugin::Column) -> Result<RsType, QueryError> {
        let db_col_name = make_column_name(column);
        if let Some(rs_type) = self.column_map.get(&db_col_name) {
            return Ok(rs_type.clone());
        };

        let db_col_type = column
            .r#type
            .as_ref()
            .map(make_column_type)
            .map(|s| s.to_lowercase())
            .ok_or_else(|| QueryError::missing_column_type(db_col_name.clone()))?;

        self.typ_map
            .get(&db_col_type)
            .cloned()
            .ok_or_else(|| QueryError::cannot_map_type(db_col_type, db_col_name))
    }

    fn insert_db_type(&mut self, db_type: &str, rs_type: RsType) {
        let e = self
            .typ_map
            .entry(db_type.to_string())
            .or_insert_with(|| rs_type.clone());
        *e = rs_type;
    }

    fn insert_column_type(&mut self, column_name: &str, rs_type: RsType) {
        let e = self
            .column_map
            .entry(column_name.to_string())
            .or_insert_with(|| rs_type.clone());
        *e = rs_type;
    }
}

#[derive(Clone)]
pub(crate) struct DbEnum {
    /// name of enum
    ///
    /// ```sql
    /// CREATE TYPE book_type AS ENUM (
    ///             ^^^^^^^^^
    ///           'FICTION',
    ///           'NONFICTION'
    /// );
    /// ```
    pub(crate) name: String,

    /// values of enum
    ///
    /// ```sql
    /// CREATE TYPE book_type AS ENUM (
    ///           'FICTION',
    ///            ^^^^^^^
    ///           'NONFICTION'
    ///            ^^^^^^^^^^
    /// );
    /// ```
    pub(crate) values: Vec<String>,

    /// additional derives for enum
    pub(crate) derives: Vec<syn::Path>,
}

impl DbEnum {
    pub(crate) fn ident(&self) -> syn::Ident {
        value_ident(&self.name)
    }
}

pub(crate) fn collect_enums(catalog: &plugin::Catalog) -> Vec<DbEnum> {
    let mut res = vec![];

    for schema in &catalog.schemas {
        for s_enum in &schema.enums {
            let db_enum = DbEnum {
                name: s_enum.name.clone(),
                values: s_enum.vals.clone(),
                derives: Vec::new(),
            };
            res.push(db_enum);
        }
    }

    res
}

pub(crate) struct ReturningRows {
    /// normalized field name
    pub(crate) column_names: Vec<syn::Ident>,
    /// original field name
    pub(crate) column_names_original: Vec<syn::LitStr>,
    pub(crate) column_types: Vec<RsColType>,
    pub(crate) query_name: String,
    pub(crate) derives: Vec<syn::Path>,
}

impl ReturningRows {
    pub(crate) fn from_query(
        db_type: &dyn DbTypeMapper,
        query: &plugin::Query,
    ) -> Result<Self, QueryError> {
        let (column_names_original, column_names): (Vec<_>, Vec<_>) =
            generate_column_names(&query.columns)
                .into_iter()
                .map(|s| {
                    (
                        syn::LitStr::new(&s, proc_macro2::Span::call_site()),
                        field_ident(&s),
                    )
                })
                .unzip();
        let mut column_types = vec![];
        for column in &query.columns {
            let rs_type = RsColType::new_with_type(db_type, column).stacked()?;

            column_types.push(rs_type);
        }

        Ok(Self {
            column_names,
            column_names_original,
            column_types,
            query_name: query.name.to_string(),
            derives: Vec::new(),
        })
    }

    pub(crate) fn struct_ident(&self) -> syn::Ident {
        value_ident(&format!("{}Row", self.query_name))
    }
}

/// sqlc annotation
/// See https://docs.sqlc.dev/en/stable/reference/query-annotations.html
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Annotation {
    Exec,
    ExecResult,
    ExecRows,
    ExecLastId,
    Many,
    One,
    BatchExec,
    BatchMany,
    BatchOne,
    CopyFrom,
}

impl std::fmt::Display for Annotation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let txt = match self {
            Annotation::Exec => ":exec",
            Annotation::ExecResult => ":execresult",
            Annotation::ExecRows => ":execrows",
            Annotation::ExecLastId => ":execlastid",
            Annotation::Many => ":many",
            Annotation::One => ":one",
            Annotation::BatchExec => ":batch",
            Annotation::BatchMany => ":batchmany",
            Annotation::BatchOne => ":batchone",
            Annotation::CopyFrom => ":copyfrom",
        };
        f.write_str(txt)
    }
}

impl std::str::FromStr for Annotation {
    type Err = QueryError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let annotation = match s {
            ":exec" => Annotation::Exec,
            ":execresult" => Annotation::ExecResult,
            ":execrows" => Annotation::ExecRows,
            ":execlastid" => Annotation::ExecLastId,
            ":many" => Annotation::Many,
            ":one" => Annotation::One,
            ":batch" => Annotation::BatchExec,
            ":batchmany" => Annotation::BatchMany,
            ":batchone" => Annotation::BatchOne,
            ":copyfrom" => Annotation::CopyFrom,
            _ => return Err(QueryError::unknown_annotation(s.to_string())),
        };
        Ok(annotation)
    }
}
fn make_raw_string_literal(s: &str) -> proc_macro2::TokenStream {
    // 文字列内の"#の組み合わせを検出して、必要なハッシュ数を決定
    let mut hash_count = 0;
    let mut current_hashes = 0;
    let mut in_quote = false;

    for ch in s.chars() {
        match ch {
            '"' => {
                if in_quote {
                    hash_count = hash_count.max(current_hashes + 1);
                }
                in_quote = !in_quote;
                current_hashes = 0;
            }
            '#' if in_quote => {
                current_hashes += 1;
            }
            _ => {
                current_hashes = 0;
            }
        }
    }

    // raw string literalを構築
    let hashes = "#".repeat(hash_count);
    let raw_str = format!("r{hashes}\"{s}\"{hashes}");

    raw_str
        .parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| proc_macro2::Literal::string(s).to_token_stream())
}

pub(crate) struct Query {
    pub(crate) param_names: Vec<syn::Ident>,
    pub(crate) param_types: Vec<RsColType>,

    pub(crate) annotation: Annotation,
    pub(crate) insert_table: Option<String>,
    /// ```sql
    /// -- name: GetAuthor :one
    ///          ^^^^^^^^^
    /// SELECT * FROM authors
    /// WHERE id = $1 LIMIT 1;
    /// ```
    pub(crate) query_name: String,
    /// ```sql
    /// -- name: GetAuthor :one
    /// SELECT * FROM authors
    /// ^^^^^^^^^^^^^^^^^^^^^
    /// WHERE id = $1 LIMIT 1;
    /// ^^^^^^^^^^^^^^^^^^^^^^
    /// ```
    query_str: String,
}

impl Query {
    pub(crate) fn from_query(
        db_type: &dyn DbTypeMapper,
        query: &plugin::Query,
    ) -> Result<Self, QueryError> {
        let (param_idx, columns): (Vec<_>, Vec<_>) = query
            .params
            .iter()
            .map(|p| {
                p.column
                    .as_ref()
                    .ok_or_else(|| QueryError::missing_param_column(p.number))
                    .map(|c| (p.number, c))
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .unzip();

        let param_names = generate_column_names(columns.iter().copied())
            .into_iter()
            .map(|s| field_ident(&s))
            .collect::<Vec<_>>();
        let param_types = columns
            .iter()
            .map(|c| RsColType::new_with_type(db_type, c))
            .collect::<Result<Vec<_>, _>>()?;

        let mut param_data = param_names
            .into_iter()
            .zip(param_types)
            .zip(param_idx)
            .collect::<Vec<_>>();

        param_data.sort_by_key(|((_, _), idx)| *idx);
        let (param_names, param_types): (Vec<_>, Vec<_>) = param_data
            .into_iter()
            .map(|((name, typ), _)| (name, typ))
            .unzip();

        let annotation = query.cmd.parse::<Annotation>().stacked()?;
        let query_name = query.name.to_string();

        let query_str = query.text.clone();
        let insert_table = query.insert_into_table.as_ref().map(|t| t.name.clone());

        Ok(Self {
            param_names,
            param_types,
            annotation,
            insert_table,
            query_name,
            query_str,
        })
    }

    pub(crate) fn query_str(&self) -> proc_macro2::TokenStream {
        match self.annotation {
            Annotation::CopyFrom => {
                let params = self
                    .param_names
                    .iter()
                    .map(|x| x.to_string())
                    .reduce(|acc, x| format!("{acc},{x}"))
                    .unwrap_or_default();
                let table = self.insert_table.as_deref().unwrap_or("table");

                let q = format!("COPY {table} ({params}) FROM STDIN (FORMAT BINARY)");
                make_raw_string_literal(&q)
            }
            _ => make_raw_string_literal(&self.query_str),
        }
    }
}

/// 次の命名規則で、カラム名を生成する
///
/// 1. テーブル名とカラム名が両方とも空の時: column_1, column_2...
/// 2. 同じカラム名が存在しないとき: column_name
/// 3. 同じカラム名が存在し、テーブル名が異なる時: table_column
/// 4. テーブル名もカラム名も同一の時: table_column_1, table_column_2
fn generate_column_names<'a, I>(columns: I) -> Vec<String>
where
    I: IntoIterator<Item = &'a plugin::Column>,
{
    // Step 1: カラム情報を収集
    let column_info: Vec<_> = columns
        .into_iter()
        .map(|column| {
            let table_name = column.table.as_ref().map(|t| t.name.as_str()).unwrap_or("");
            let column_name = column.name.as_str();
            (table_name, column_name)
        })
        .collect();

    // Step 2: 空でないカラム名の出現回数をカウント
    let mut column_name_counts = std::collections::HashMap::new();
    for &(_, column_name) in &column_info {
        if !column_name.is_empty() {
            *column_name_counts.entry(column_name).or_insert(0) += 1;
        }
    }

    // Step 3: 各カラムの基本名を決定
    let mut base_names = Vec::new();
    for (i, &(table_name, column_name)) in column_info.iter().enumerate() {
        let base_name = match (table_name.is_empty(), column_name.is_empty()) {
            (true, true) => {
                // Rule 1: テーブル名とカラム名が両方とも空
                format!("column_{}", i + 1)
            }
            (_, true) => {
                // カラム名が空の場合（テーブル名の有無は関係なし）
                format!("column_{}", i + 1)
            }
            (_, false) => {
                // カラム名が存在する場合
                let count = column_name_counts.get(column_name).unwrap_or(&0);
                if *count == 1 {
                    // Rule 2: 同じカラム名が存在しない
                    column_name.to_string()
                } else {
                    // Rule 3: 同じカラム名が存在する
                    if table_name.is_empty() {
                        column_name.to_string()
                    } else {
                        format!("{table_name}_{column_name}")
                    }
                }
            }
        };
        base_names.push(base_name);
    }

    // Step 4: 最終的な名前の重複を解決（Rule 4）
    // まず重複する基本名を特定
    let mut base_name_counts = std::collections::HashMap::new();
    for base_name in &base_names {
        *base_name_counts.entry(base_name.clone()).or_insert(0) += 1;
    }

    let mut final_names = Vec::new();
    let mut name_occurrence_counts = std::collections::HashMap::new();

    for base_name in base_names {
        let total_count = base_name_counts.get(&base_name).unwrap_or(&1);
        let occurrence_count = name_occurrence_counts.entry(base_name.clone()).or_insert(0);
        *occurrence_count += 1;

        let final_name = if *total_count == 1 {
            // 重複がない場合はそのまま
            base_name
        } else {
            // 重複がある場合は最初から連番を付ける
            format!("{base_name}_{occurrence_count}")
        };
        final_names.push(final_name);
    }

    final_names
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_column(table_name: Option<&str>, column_name: &str) -> plugin::Column {
        plugin::Column {
            name: column_name.to_string(),
            table: table_name.map(|name| plugin::Identifier {
                name: name.to_string(),
                schema: String::new(),
                catalog: String::new(),
            }),
            not_null: false,
            is_array: false,
            comment: String::new(),
            length: 0,
            is_named_param: false,
            is_func_call: false,
            scope: String::new(),
            table_alias: String::new(),
            r#type: None,
            is_sqlc_slice: false,
            embed_table: None,
            original_name: String::new(),
            unsigned: false,
            array_dims: 0,
        }
    }

    #[test]
    fn test_empty_columns() {
        let columns = vec![create_test_column(None, ""), create_test_column(None, "")];

        let names = generate_column_names(&columns);
        assert_eq!(names, vec!["column_1", "column_2"]);
    }

    #[test]
    fn test_unique_column_names() {
        let columns = vec![
            create_test_column(None, "id"),
            create_test_column(None, "name"),
        ];

        let names = generate_column_names(&columns);
        assert_eq!(names, vec!["id", "name"]);
    }

    #[test]
    fn test_duplicate_column_names_different_tables() {
        let columns = vec![
            create_test_column(Some("users"), "id"),
            create_test_column(Some("posts"), "id"),
        ];

        let names = generate_column_names(&columns);
        assert_eq!(names, vec!["users_id", "posts_id"]);
    }

    #[test]
    fn test_duplicate_table_and_column() {
        let columns = vec![
            create_test_column(Some("users"), "id"),
            create_test_column(Some("users"), "id"),
        ];

        let names = generate_column_names(&columns);
        assert_eq!(names, vec!["users_id_1", "users_id_2"]);
    }

    #[test]
    fn test_mixed_scenarios() {
        let columns = vec![
            create_test_column(None, ""),            // column_1
            create_test_column(None, "name"),        // name (unique)
            create_test_column(Some("users"), "id"), // users_id_1 (重複するので連番)
            create_test_column(Some("posts"), "id"), // posts_id (重複しないのでそのまま)
            create_test_column(None, "id"),          // id (重複しないのでそのまま)
            create_test_column(Some("users"), "id"), // users_id_2 (重複するので連番)
        ];

        let names = generate_column_names(&columns);
        assert_eq!(
            names,
            vec![
                "column_1",
                "name",
                "users_id_1",
                "posts_id",
                "id",
                "users_id_2"
            ]
        );
    }

    #[test]
    fn test_complex_scenario_with_multiple_duplicates() {
        let columns = vec![
            create_test_column(Some("users"), "name"), // users_name_1
            create_test_column(Some("posts"), "name"), // posts_name
            create_test_column(Some("users"), "name"), // users_name_2
            create_test_column(None, "name"),          // name_1
            create_test_column(None, "name"),          // name_2
        ];

        let names = generate_column_names(&columns);
        assert_eq!(
            names,
            vec![
                "users_name_1",
                "posts_name",
                "users_name_2",
                "name_1",
                "name_2"
            ]
        );
    }
}
