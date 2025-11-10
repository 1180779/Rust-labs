use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::hash::Hash;
use std::io::{Read, Write};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ExecutionError {
    #[error("Invalid field: {0} for table: {1}")]
    InvalidField(String, String),

    #[error("Invalid fields: {0:?} for table: {1}")]
    InvalidFields(Vec<String>, String),

    #[error("Fields: {0:?} are missing in the insert for table: {1}")]
    MissingFields(Vec<String>, String),

    #[error("Fields: {0:?} are specified more than once for table: {1}")]
    MoreThanOnceSpecifiedValues(Vec<String>, String),

    #[error("Values for fields: {0:?} do not match field types for table: {1}")]
    InvalidTypes(Vec<String>, String),

    #[error("Record with key: {0} already exists in table: {1}")]
    RecordWithKeyAlreadyExists(String, String),

    #[error("Record with key: {0} not found in table: {1}")]
    RecordWithKeyNotFound(String, String),

    #[error("Table: {0} already exists")]
    TableAlreadyExists(String),

    #[error("Table: {0} does not exists")]
    TableNotFound(String),

    #[error("{0}")]
    IoError(std::io::ErrorKind),
}

impl From<std::io::Error> for ExecutionError {
    fn from(e: std::io::Error) -> Self {
        ExecutionError::IoError(e.kind())
    }
}

pub mod error;
pub mod parser;
use crate::AnyDatabase::{IntDatabase, StringDatabase};
use crate::error::DbError;
pub use parser::*;
/////////////////////////////////////////////
// owned query types
/////////////////////////////////////////////

#[derive(Debug)]
pub enum AnyQueryOwned {
    StringQuery(QueryOwned<String>),
    IntQuery(QueryOwned<i64>),
}

impl<'a: 'b, 'b> From<&'b AnyQuery<'a>> for AnyQueryOwned {
    fn from(value: &'b AnyQuery<'a>) -> Self {
        match value {
            AnyQuery::StringQuery(q) => AnyQueryOwned::StringQuery(q.into()),
            AnyQuery::IntQuery(q) => AnyQueryOwned::IntQuery(q.into()),
        }
    }
}

impl Display for AnyQueryOwned {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AnyQueryOwned::StringQuery(q) => write!(f, "{}", q),
            AnyQueryOwned::IntQuery(q) => write!(f, "{}", q),
        }
    }
}

#[derive(Debug)]
pub enum QueryOwned<K: DatabaseKey> {
    Create(CreateQueryOwned),
    Delete(DeleteQueryOwned<K>),
    Insert(InsertQueryOwned),
    Select(SelectQueryOwned),
    SaveAs(SaveAsQueryOwned),
    ReadFrom(ReadFromQueryOwned),
}

impl<K: DatabaseKey> Display for QueryOwned<K> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryOwned::Select(query) => write!(f, "{}", query),
            QueryOwned::Create(query) => write!(f, "{}", query),
            QueryOwned::Delete(query) => write!(f, "{}", query),
            QueryOwned::Insert(query) => write!(f, "{}", query),
            QueryOwned::SaveAs(query) => write!(f, "{}", query),
            QueryOwned::ReadFrom(query) => write!(f, "{}", query),
        }
    }
}

impl Display for SelectQueryOwned {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.where_clause {
            Some(where_clause) => write!(
                f,
                "SELECT {} FROM {} WHERE {}",
                self.fields, self.table, where_clause
            ),
            None => write!(f, "SELECT {} FROM {}", self.fields, self.table),
        }
    }
}

impl Display for CreateQueryOwned {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CREATE {} KEY {} FIELDS {}",
            self.table, self.key_field, self.fields_types
        )
    }
}

impl Display for InsertQueryOwned {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut setters: String;
        if self.insert_values.len() == 1 {
            setters = self.insert_values[0].to_string();
        } else {
            setters = self.insert_values[0..self.insert_values.len()]
                .iter()
                .fold(String::new(), |a, v| {
                    format!("{}{}, ", a, v).as_str().to_string()
                })
                .to_string();
            setters.push_str(
                self.insert_values[self.insert_values.len() - 1]
                    .to_string()
                    .as_str(),
            );
        }
        write!(f, "INSERT {} INTO {}", setters, self.table)
    }
}

impl<K: DatabaseKey> Display for DeleteQueryOwned<K> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "DELETE {} FROM {}", self.key, self.table)
    }
}

impl Display for SaveAsQueryOwned {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "SAVE_AS {}", self.file)
    }
}

impl Display for ReadFromQueryOwned {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "READ_FROM {}", self.file)
    }
}

#[derive(Debug, PartialEq)]
pub struct NewFields(Vec<NewFieldOwned>);

impl Display for NewFields {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.0.len() {
            0 => write!(f, ""),
            1 => write!(f, "{}", self.0[0]),
            _ => {
                let mut fields = self.0[0..self.0.len() - 1]
                    .iter()
                    .fold(String::new(), |acc, f| {
                        format!("{}{}, ", acc, f).to_string()
                    });
                fields.push_str(self.0[self.0.len() - 1].to_string().as_str());
                write!(f, "{}", fields)
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct CreateQueryOwned {
    pub table: String,
    pub key_field: String,
    pub fields_types: NewFields,
}

#[derive(Debug, PartialEq)]
pub struct DeleteQueryOwned<K: DatabaseKey> {
    pub key: K,
    pub table: String,
}

#[derive(Debug, PartialEq)]
pub struct InsertQueryOwned {
    pub insert_values: Vec<InsertValueOwned>,
    pub table: String,
}

#[derive(Debug, PartialEq)]
pub struct SelectQueryOwned {
    pub fields: SelectFieldsOwned,
    pub table: String,
    pub where_clause: Option<WhereOwned>,
}

#[derive(Debug, PartialEq)]
pub struct WhereOwned {
    pub field: String,
    pub op: Op,
    pub value: Value,
}

impl Display for WhereOwned {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.field, self.op, self.value)
    }
}

#[derive(Debug, PartialEq)]
pub struct SaveAsQueryOwned {
    pub file: String,
}

#[derive(Debug, PartialEq)]
pub struct ReadFromQueryOwned {
    pub file: String,
}

#[derive(Debug, PartialEq)]
pub struct NewFieldOwned {
    pub field: String,
    pub field_type: FieldType,
}

impl Display for NewFieldOwned {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.field, self.field_type)
    }
}

#[derive(Debug, PartialEq)]
pub enum SelectFieldsOwned {
    Fields(Vec<String>),
    AllFields(),
}

impl Display for SelectFieldsOwned {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SelectFieldsOwned::Fields(fields) => match fields.len() {
                0 => write!(f, ""),
                1 => write!(f, "{}", fields[0]),
                _ => {
                    let mut str = fields[0..fields.len() - 1]
                        .iter()
                        .fold(String::new(), |acc, f| format!("{}{}, ", acc, f));
                    str.push_str(fields[fields.len() - 1].as_str());
                    write!(f, "{}", str)
                }
            },
            SelectFieldsOwned::AllFields() => {
                write!(f, "*")
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct InsertValueOwned {
    pub field: String,
    pub value: Value,
}

impl Display for InsertValueOwned {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}={}", self.field, self.value)
    }
}

impl<'a, 'b> From<&'b InsertValue<'a>> for InsertValueOwned {
    fn from(value: &'b InsertValue) -> Self {
        InsertValueOwned {
            field: value.field.into(),
            value: (&value.value).into(),
        }
    }
}

impl<'a: 'b, 'b> From<&'b CommandValue<'a>> for Value {
    fn from(value: &'b CommandValue) -> Self {
        match value {
            CommandValue::Bool(b) => Value::Bool(*b),
            CommandValue::String(s) => Value::String((*s).into()),
            CommandValue::Int(i) => Value::Int(*i),
            CommandValue::Float(f) => Value::Float(*f),
        }
    }
}

impl<'a: 'b, 'b> From<&'b NewField<'a>> for NewFieldOwned {
    fn from(value: &'b NewField<'a>) -> Self {
        NewFieldOwned {
            field: value.field.into(),
            field_type: value.field_type,
        }
    }
}

impl<'a: 'b, 'b> From<&'b SelectFields<'a>> for SelectFieldsOwned {
    fn from(value: &'b SelectFields<'a>) -> Self {
        match value {
            SelectFields::AllFields() => SelectFieldsOwned::AllFields(),
            SelectFields::Fields(fields) => {
                SelectFieldsOwned::Fields(fields.iter().map(|s| (*s).into()).collect())
            }
        }
    }
}

impl<'a: 'b, 'b> From<&'b CreateQuery<'a>> for CreateQueryOwned {
    fn from(value: &'b CreateQuery<'a>) -> Self {
        CreateQueryOwned {
            table: value.table.into(),
            key_field: value.key_field.into(),
            fields_types: NewFields(value.fields_types.iter().map(|t| t.into()).collect()),
        }
    }
}

impl<'a: 'b, 'b, K: DatabaseKey> From<&'b DeleteQuery<'a, K>> for DeleteQueryOwned<K> {
    fn from(value: &'b DeleteQuery<'a, K>) -> Self {
        DeleteQueryOwned {
            table: value.table.into(),
            key: value.key.clone(),
        }
    }
}

impl<'a: 'b, 'b> From<&'b InsertQuery<'a>> for InsertQueryOwned {
    fn from(value: &'b InsertQuery<'a>) -> Self {
        InsertQueryOwned {
            table: value.table.into(),
            insert_values: value.insert_values.iter().map(|v| v.into()).collect(),
        }
    }
}

impl<'a: 'b, 'b> From<&'b SelectQuery<'a>> for SelectQueryOwned {
    fn from(value: &'b SelectQuery<'a>) -> Self {
        SelectQueryOwned {
            table: value.table.into(),
            fields: (&value.fields).into(),
            where_clause: value
                .where_clause
                .as_ref()
                .map(|where_clause| where_clause.into()),
        }
    }
}

impl<'a: 'b, 'b> From<&'b SaveAsQuery<'a>> for SaveAsQueryOwned {
    fn from(value: &'b SaveAsQuery<'a>) -> Self {
        SaveAsQueryOwned {
            file: value.file.into(),
        }
    }
}

impl<'a: 'b, 'b> From<&'b ReadFromQuery<'a>> for ReadFromQueryOwned {
    fn from(value: &'b ReadFromQuery<'a>) -> Self {
        ReadFromQueryOwned {
            file: value.file.into(),
        }
    }
}

impl<'a: 'b, 'b, K: DatabaseKey> From<&'b Query<'a, K>> for QueryOwned<K> {
    fn from(value: &'b Query<'a, K>) -> Self {
        match value {
            Query::Select(q) => QueryOwned::Select(q.into()),
            Query::Delete(q) => QueryOwned::Delete(q.into()),
            Query::Insert(q) => QueryOwned::Insert(q.into()),
            Query::Create(q) => QueryOwned::Create(q.into()),
            Query::SaveAs(q) => QueryOwned::SaveAs(q.into()),
            Query::ReadFrom(q) => QueryOwned::ReadFrom(q.into()),
        }
    }
}

/////////////////////////////////////////////
// rest
/////////////////////////////////////////////

#[derive(Debug)]
pub struct SelectCommand<'a, 'b, K: DatabaseKey> {
    pub table: &'a Table<K>,
    pub query: SelectQuery<'b>,
}

#[derive(Debug)]
pub struct InsertCommand<'a, 'b, K: DatabaseKey> {
    pub table: &'a mut Table<K>,
    pub query: InsertQuery<'b>,
}

#[derive(Debug)]
pub struct CreateCommand<'a, 'b, K: DatabaseKey> {
    pub db: &'a mut Database<K>,
    pub query: CreateQuery<'b>,
}

#[derive(Debug)]
pub struct SaveAsCommand<'a, 'b, K: DatabaseKey> {
    db: &'a Database<K>,
    pub query: SaveAsQuery<'b>,
}

#[derive(Debug)]
pub struct ReadFromCommand<'a, 'b, K: DatabaseKey> {
    db: &'a mut Database<K>,
    pub query: ReadFromQuery<'b>,
}

#[derive(Debug)]
pub struct DeleteCommand<'a, 'b, K: DatabaseKey> {
    pub table: &'a mut Table<K>,
    pub query: DeleteQuery<'b, K>,
}

#[derive(Debug)]
pub enum AnyCommandInternal<'a, 'b, K: DatabaseKey> {
    Select(SelectCommand<'a, 'b, K>),
    Insert(InsertCommand<'a, 'b, K>),
    Create(CreateCommand<'a, 'b, K>),
    Delete(DeleteCommand<'a, 'b, K>),
    SaveAs(SaveAsCommand<'a, 'b, K>),
    ReadFrom(ReadFromCommand<'a, 'b, K>),
}

impl<'a, 'b, K: DatabaseKey> AnyCommandInternal<'a, 'b, K> {
    pub fn query(self) -> Query<'b, K> {
        match self {
            AnyCommandInternal::Select(c) => c.query.into(),
            AnyCommandInternal::Create(c) => c.query.into(),
            AnyCommandInternal::Insert(c) => c.query.into(),
            AnyCommandInternal::Delete(c) => c.query.into(),
            AnyCommandInternal::SaveAs(c) => c.query.into(),
            AnyCommandInternal::ReadFrom(c) => c.query.into(),
        }
    }
}

#[derive(Debug)]
pub enum AnyCommand<'a, 'b> {
    StringCommand(AnyCommandInternal<'a, 'b, String>),
    IntCommand(AnyCommandInternal<'a, 'b, i64>),
}

impl<'a, 'b> AnyCommand<'a, 'b> {
    pub fn query(self) -> AnyQuery<'b> {
        match self {
            AnyCommand::StringCommand(c) => AnyQuery::StringQuery(c.query()),
            AnyCommand::IntCommand(c) => AnyQuery::IntQuery(c.query()),
        }
    }
}

fn parse_command_create<'a, 'b, K: DatabaseKey>(
    db: &'a mut Database<K>,
    query: CreateQuery<'b>,
) -> Result<AnyCommandInternal<'a, 'b, K>, ExecutionError> {
    Ok(AnyCommandInternal::Create(CreateCommand { db, query }))
}

fn parse_command_select<'a, 'b, K: DatabaseKey>(
    db: &'a Database<K>,
    query: SelectQuery<'b>,
) -> Result<AnyCommandInternal<'a, 'b, K>, ExecutionError> {
    let table = db.tables.get(query.table);
    match table {
        Some(table) => Ok(AnyCommandInternal::Select(SelectCommand { table, query })),
        None => Err(ExecutionError::TableNotFound(query.table.into())),
    }
}

fn parse_command_insert<'a, 'b, K: DatabaseKey>(
    db: &'a mut Database<K>,
    query: InsertQuery<'b>,
) -> Result<AnyCommandInternal<'a, 'b, K>, ExecutionError> {
    let table = db.tables.get_mut(query.table);
    match table {
        Some(table) => Ok(AnyCommandInternal::Insert(InsertCommand { table, query })),
        None => Err(ExecutionError::TableNotFound(query.table.into())),
    }
}

fn parse_command_delete<'a, 'b, K: DatabaseKey>(
    db: &'a mut Database<K>,
    query: DeleteQuery<'b, K>,
) -> Result<AnyCommandInternal<'a, 'b, K>, ExecutionError> {
    let table = db.tables.get_mut(query.table);
    match table {
        Some(table) => Ok(AnyCommandInternal::Delete(DeleteCommand { table, query })),
        None => Err(ExecutionError::TableNotFound(query.table.into())),
    }
}

fn parse_command_<'a, 'b, K: DatabaseKey>(
    db: &'a mut Database<K>,
    command: &'b str,
) -> Result<AnyCommandInternal<'a, 'b, K>, DbError> {
    let query = parse_query::<K>(command)?;
    match query {
        Query::Create(query) => Ok(parse_command_create(db, query)?),
        Query::Select(query) => Ok(parse_command_select(db, query)?),
        Query::Insert(query) => Ok(parse_command_insert(db, query)?),
        Query::Delete(query) => Ok(parse_command_delete(db, query)?),
        Query::SaveAs(query) => Ok(AnyCommandInternal::SaveAs(SaveAsCommand { db, query })),
        Query::ReadFrom(query) => Ok(AnyCommandInternal::ReadFrom(ReadFromCommand { db, query })),
    }
}

pub fn parse_command<'a, 'b>(
    db: &'a mut AnyDatabase,
    command: &'b str,
) -> Result<AnyCommand<'a, 'b>, DbError> {
    match db {
        StringDatabase(db) => parse_command_(db, command).map(AnyCommand::StringCommand),
        IntDatabase(db) => parse_command_(db, command).map(AnyCommand::IntCommand),
    }
}

#[derive(Debug, PartialOrd, Clone)]
pub enum CommandValue<'a> {
    Bool(bool),
    String(&'a str),
    Int(i64),
    Float(f64),
}

const FLOAT_EPSILON: f64 = 1e-10;

impl<'a> PartialEq for CommandValue<'a> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (CommandValue::Float(a), CommandValue::Float(b)) => (a - b).abs() < FLOAT_EPSILON,
            (CommandValue::Bool(a), CommandValue::Bool(b)) => a == b,
            (CommandValue::String(a), CommandValue::String(b)) => a == b,
            (CommandValue::Int(a), CommandValue::Int(b)) => a == b,
            _ => false,
        }
    }
}

impl CommandValue<'_> {
    fn field_type(&self) -> FieldType {
        match self {
            CommandValue::Bool(_) => FieldType::Bool,
            CommandValue::String(_) => FieldType::String,
            CommandValue::Int(_) => FieldType::Int,
            CommandValue::Float(_) => FieldType::Float,
        }
    }
}

impl<'a> From<&'a Value> for CommandValue<'a> {
    fn from(value: &'a Value) -> Self {
        match value {
            Value::Bool(v) => CommandValue::Bool(*v),
            Value::String(v) => CommandValue::String(v.as_str()),
            Value::Int(v) => CommandValue::Int(*v),
            Value::Float(v) => CommandValue::Float(*v),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Bool(bool),
    String(String),
    Int(i64),
    Float(f64),
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Bool(v) => write!(f, "{}", v),
            Value::String(v) => write!(f, "\"{}\"", v),
            Value::Int(v) => write!(f, "{}", v),
            Value::Float(v) => write!(f, "{}", v),
        }
    }
}

#[derive(Debug, Clone)]
struct CommandRecord<'a> {
    values: Vec<CommandValue<'a>>,
}

#[derive(Debug, Clone)]
pub struct Record {
    values: HashMap<String, Value>,
}

impl<'a> From<&'a Record> for CommandRecord<'a> {
    fn from(value: &'a Record) -> Self {
        CommandRecord {
            values: value.values.iter().map(|v| v.1.into()).collect(),
        }
    }
}

#[derive(Debug)]
struct CommandHistory<K: DatabaseKey>(Vec<QueryOwned<K>>);

impl<K: DatabaseKey> CommandHistory<K> {
    fn new() -> Self {
        CommandHistory(vec![])
    }
}

impl<K: DatabaseKey> Display for CommandHistory<K> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let res = self.0.iter().fold(String::new(), |acc, q| {
            format!("{}{}\n", acc, q).to_string()
        });
        write!(f, "{}", res)
    }
}

#[derive(Debug)]
pub struct Database<K: DatabaseKey> {
    tables: HashMap<String, Table<K>>,
    command_history: CommandHistory<K>,
}

impl<K: DatabaseKey> Default for Database<K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: DatabaseKey> Database<K> {
    pub fn new() -> Database<K> {
        Database {
            tables: HashMap::new(),
            command_history: CommandHistory::new(),
        }
    }

    pub fn history_push(&mut self, query: QueryOwned<K>) {
        self.command_history.0.push(query);
    }
}

#[derive(Debug)]
pub struct Table<K: DatabaseKey> {
    key_field: String,
    types: HashMap<String, FieldType>,
    records: std::collections::BTreeMap<K, Record>,
}

#[derive(Debug)]
pub enum AnyQuery<'a> {
    StringQuery(Query<'a, String>),
    IntQuery(Query<'a, i64>),
}

#[derive(Debug)]
pub enum AnyDatabase {
    StringDatabase(Database<String>),
    IntDatabase(Database<i64>),
}

impl AnyDatabase {
    pub fn new_string_database() -> Self {
        StringDatabase(Database::new())
    }

    pub fn new_int_database() -> Self {
        IntDatabase(Database::new())
    }

    pub fn history_push(&mut self, query: AnyQueryOwned) -> Result<(), String> {
        match self {
            StringDatabase(database) => match query {
                AnyQueryOwned::StringQuery(query) => {
                    database.command_history.0.push(query);
                    Ok(())
                }
                AnyQueryOwned::IntQuery(_) => Err("invalid query type".to_string()),
            },
            IntDatabase(database) => match query {
                AnyQueryOwned::IntQuery(query) => {
                    database.command_history.0.push(query);
                    Ok(())
                }
                AnyQueryOwned::StringQuery(_) => Err("invalid query type".to_string()),
            },
        }
    }
}

#[derive(Debug)]
pub struct SelectResult<'a, 'b> {
    fields: Vec<&'b str>,
    records: Vec<CommandRecord<'a>>,
}

impl<'a, 'b> SelectResult<'a, 'b> {
    fn command_value_to_string(cv: &CommandValue<'_>) -> String {
        match cv {
            CommandValue::Bool(b) => format!("{}", b),
            CommandValue::String(s) => s.to_string(),
            CommandValue::Int(i) => format!("{}", i),
            CommandValue::Float(fl) => format!("{}", fl),
        }
    }

    fn pad_cell(s: &str, width: usize) -> String {
        let mut out = String::new();
        out.push(' ');
        out.push_str(s);
        if width > 1 + s.len() {
            out.push_str(&" ".repeat(width - 1 - s.len()));
        }
        out
    }

    fn build_columns(&self) -> Vec<Vec<String>> {
        let mut cols: Vec<Vec<String>> = Vec::with_capacity(self.fields.len());
        for _ in 0..self.fields.len() {
            cols.push(Vec::new());
        }

        for rec in &self.records {
            for (i, val) in rec.values.iter().enumerate().take(self.fields.len()) {
                cols[i].push(Self::command_value_to_string(val));
            }
        }

        cols
    }

    fn calculate_widths(&self, cols: &[Vec<String>]) -> Vec<usize> {
        const MARGIN: usize = 2; // spaces padding (one left, one right)
        let mut widths: Vec<usize> = self.fields.iter().map(|h| h.len()).collect();

        for (i, col) in cols.iter().enumerate() {
            for cell in col {
                if cell.len() > widths[i] {
                    widths[i] = cell.len();
                }
            }
            widths[i] += MARGIN;
        }

        widths
    }

    fn build_separator(
        widths: &[usize],
        left: char,
        mid: char,
        right: char,
        horiz: char,
    ) -> String {
        let mut sep = String::new();
        sep.push(left);
        for (i, w) in widths.iter().enumerate() {
            sep.push_str(&horiz.to_string().repeat(*w));
            if i < widths.len() - 1 {
                sep.push(mid);
            }
        }
        sep.push(right);
        sep
    }

    fn build_row(cells: &[&str], widths: &[usize]) -> String {
        let mut row = String::new();
        row.push('│');
        for (i, cell) in cells.iter().enumerate() {
            row.push_str(&Self::pad_cell(cell, widths[i]));
            row.push('│');
        }
        row
    }
}

impl<'a, 'b> Display for SelectResult<'a, 'b> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let cols = self.build_columns();
        let widths = self.calculate_widths(&cols);

        let top_sep = Self::build_separator(&widths, '┌', '┬', '┐', '─');
        writeln!(f, "{}", top_sep)?;

        let header_cells: Vec<&str> = self.fields.to_vec();
        let header = Self::build_row(&header_cells, &widths);
        writeln!(f, "{}", header)?;

        let mid_sep = Self::build_separator(&widths, '├', '┼', '┤', '─');
        writeln!(f, "{}", mid_sep)?;

        for row_idx in 0..self.records.len() {
            let row_cells: Vec<&str> = (0..self.fields.len())
                .map(|col_idx| {
                    cols.get(col_idx)
                        .and_then(|c| c.get(row_idx))
                        .map(|s| s.as_str())
                        .unwrap_or("")
                })
                .collect();
            let row = Self::build_row(&row_cells, &widths);
            writeln!(f, "{}", row)?;
        }

        let bottom_sep = Self::build_separator(&widths, '└', '┴', '┘', '─');
        writeln!(f, "{}", bottom_sep)
    }
}

#[derive(Debug)]
pub enum CommandResult<'a, 'b> {
    Select(SelectResult<'a, 'b>),
    Create,
    Delete,
    Insert,
    SaveAs,
    ReadFrom,
}

impl<'a, 'b> Display for CommandResult<'a, 'b> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandResult::Select(select) => write!(f, "Done Select:\n{}", select),
            CommandResult::Create => write!(f, "Done Create"),
            CommandResult::Delete => write!(f, "Done Delete"),
            CommandResult::Insert => write!(f, "Done Insert"),
            CommandResult::SaveAs => write!(f, "Done SaveAs"),
            CommandResult::ReadFrom => write!(f, "Done ReadFrom"),
        }
    }
}

pub trait Command<'a, 'b> {
    fn execute(&mut self) -> Result<CommandResult<'a, 'b>, DbError>;
}

impl<'a: 'b, 'b, K: DatabaseKey> SelectCommand<'a, 'b, K> {
    fn execute_fields(&self) -> Result<Vec<&'b str>, DbError> {
        Ok(match &self.query.fields {
            SelectFields::AllFields() => {
                let mut fields: Vec<&str> = self.table.types.keys().map(|k| k.as_str()).collect();
                fields.sort();
                fields
            }
            SelectFields::Fields(v) => {
                for field_name in v {
                    if !self.table.types.contains_key(*field_name) {
                        return Err(ExecutionError::InvalidField(
                            (*field_name).into(),
                            self.query.table.into(),
                        )
                        .into());
                    }
                }
                v.to_vec()
            }
        })
    }

    fn execute_select(
        &self,
        selected_fields: &[&'b str],
    ) -> Result<Vec<CommandRecord<'a>>, DbError> {
        Ok(self
            .table
            .records
            .iter()
            .filter(|(_, record)| {
                self.query
                    .where_clause
                    .as_ref()
                    .is_none_or(|wc| wc.filter(record))
            })
            .map(|(key, record)| {
                let mut values = Vec::with_capacity(selected_fields.len());
                for &field_name in selected_fields {
                    if field_name == self.table.key_field {
                        values.push(K::get_command_value(key));
                    } else if let Some(val) = record.values.get(field_name) {
                        values.push(val.into());
                    }
                }
                CommandRecord { values }
            })
            .collect())
    }
}

impl<'a: 'b, 'b, K: DatabaseKey> Command<'a, 'b> for SelectCommand<'a, 'b, K> {
    fn execute(&mut self) -> Result<CommandResult<'a, 'b>, DbError> {
        let selected_fields: Vec<&str> = self.execute_fields()?;
        let result_records = self.execute_select(&selected_fields)?;

        Ok(CommandResult::Select(SelectResult {
            fields: selected_fields,
            records: result_records,
        }))
    }
}

impl<'a: 'b, 'b, K: DatabaseKey> Command<'a, 'b> for SaveAsCommand<'a, 'b, K> {
    fn execute(&mut self) -> Result<CommandResult<'a, 'b>, DbError> {
        let f = File::create(self.query.file);
        match f {
            Ok(mut file) => {
                let write_res = file.write_all(self.db.command_history.to_string().as_bytes());
                match write_res {
                    Ok(_) => Ok(CommandResult::Create),
                    Err(e) => Err(DbError::ExecutionError(e.into())),
                }
            }
            Err(e) => Err(DbError::ExecutionError(e.into())),
        }
    }
}

impl<'a, 'b, K: DatabaseKey> Command<'a, 'b> for ReadFromCommand<'a, 'b, K> {
    fn execute(&mut self) -> Result<CommandResult<'a, 'b>, DbError> {
        let f = File::open(self.query.file);
        match f {
            Ok(mut file) => {
                let mut file_content = String::new();
                let read_res = file.read_to_string(&mut file_content);
                if let Err(e) = read_res {
                    return Err(DbError::ExecutionError(e.into()));
                }
                for line in file_content.lines() {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }
                    let mut command = parse_command_(self.db, line)?;
                    let result = command.execute()?;
                    println!("{}", result);
                    let query = (&command.query()).into();
                    self.db.history_push(query);
                }
                Ok(CommandResult::ReadFrom)
            }
            Err(e) => Err(DbError::ExecutionError(e.into())),
        }
    }
}

impl<'a: 'b, 'b, K: DatabaseKey> Command<'a, 'b> for CreateCommand<'a, 'b, K> {
    fn execute(&mut self) -> Result<CommandResult<'a, 'b>, DbError> {
        let existing = self.db.tables.get(self.query.table);
        if existing.is_some() {
            return Err(ExecutionError::TableAlreadyExists(self.query.table.into()).into());
        }

        let mut field_types: HashMap<String, FieldType> = self
            .query
            .fields_types
            .iter()
            .map(|e| (e.field.to_owned(), e.field_type))
            .collect();
        field_types.insert(self.query.key_field.to_owned(), K::field_type());
        let new_table = Table {
            key_field: self.query.key_field.to_owned(),
            types: field_types,
            records: std::collections::BTreeMap::new(),
        };
        self.db
            .tables
            .insert(self.query.table.to_owned(), new_table);
        Ok(CommandResult::Create)
    }
}

impl<'a: 'b, 'b, K: DatabaseKey> Command<'a, 'b> for InsertCommand<'a, 'b, K> {
    fn execute(&mut self) -> Result<CommandResult<'a, 'b>, DbError> {
        let non_existent: Vec<&InsertValue> = self
            .query
            .insert_values
            .iter()
            .filter(|p| !self.table.types.contains_key(p.field))
            .collect();
        if !non_existent.is_empty() {
            return Err(ExecutionError::InvalidFields(
                non_existent.iter().map(|f| f.field.to_string()).collect(),
                self.query.table.into(),
            )
            .into());
        }

        let mut number_of_occurrences: HashMap<&str, u64> =
            self.table.types.iter().map(|t| (t.0.as_str(), 0)).collect();
        number_of_occurrences.insert(&self.table.key_field, 0);

        self.query.insert_values.iter().for_each(|p| {
            let f = number_of_occurrences.get_mut(p.field);
            if f.is_none() {
                return;
            }
            let f = f.unwrap();
            *f += 1;
        });
        let missing_fields: Vec<&str> = number_of_occurrences
            .iter()
            .filter(|p| *p.1 == 0)
            .map(|p| *p.0)
            .collect();
        if !missing_fields.is_empty() {
            return Err(ExecutionError::MissingFields(
                missing_fields
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>(),
                self.query.table.into(),
            )
            .into());
        }

        let duplicated_fields: Vec<String> = number_of_occurrences
            .iter()
            .filter(|p| *p.1 > 1)
            .map(|p| p.0.to_string())
            .collect();
        if !duplicated_fields.is_empty() {
            return Err(ExecutionError::MoreThanOnceSpecifiedValues(
                duplicated_fields,
                self.query.table.into(),
            )
            .into());
        }

        let non_matching_types: Vec<String> = self
            .query
            .insert_values
            .iter()
            .filter(|p| p.value.field_type() != *self.table.types.get(p.field).unwrap())
            .map(|p| p.field.to_string())
            .collect();
        if !non_matching_types.is_empty() {
            return Err(
                ExecutionError::InvalidTypes(non_matching_types, self.query.table.into()).into(),
            );
        }

        let insert_without_key: HashMap<String, Value> = self
            .query
            .insert_values
            .iter()
            .filter(|p| p.field != self.table.key_field)
            .map(|p| (p.field.to_owned(), (&p.value).into()))
            .collect();
        let key = self
            .query
            .insert_values
            .iter()
            .find(|p| p.field == self.table.key_field)
            .map(|p| p.value.clone())
            .unwrap();
        let key_value = K::from_command_value(key).unwrap();

        if self.table.records.contains_key(&key_value) {
            return Err(ExecutionError::RecordWithKeyAlreadyExists(
                key_value.to_string(),
                self.query.table.to_string(),
            )
            .into());
        }

        self.table.records.insert(
            key_value,
            Record {
                values: insert_without_key,
            },
        );

        Ok(CommandResult::Insert)
    }
}

impl<'a: 'b, 'b, K: DatabaseKey> Command<'a, 'b> for DeleteCommand<'a, 'b, K> {
    fn execute(&mut self) -> Result<CommandResult<'a, 'b>, DbError> {
        let delete_res = self.table.records.remove(&self.query.key);
        if delete_res.is_none() {
            return Err(ExecutionError::RecordWithKeyNotFound(
                self.query.key.to_string(),
                self.query.table.into(),
            )
            .into());
        }
        Ok(CommandResult::Delete)
    }
}

impl<'a: 'b, 'b, K: DatabaseKey> Command<'a, 'b> for AnyCommandInternal<'a, 'b, K> {
    fn execute(&mut self) -> Result<CommandResult<'a, 'b>, DbError> {
        match self {
            AnyCommandInternal::Select(select) => select.execute(),
            AnyCommandInternal::Insert(insert) => insert.execute(),
            AnyCommandInternal::Delete(delete) => delete.execute(),
            AnyCommandInternal::Create(create) => create.execute(),
            AnyCommandInternal::SaveAs(save_as) => save_as.execute(),
            AnyCommandInternal::ReadFrom(read_from) => read_from.execute(),
        }
    }
}

impl<'a: 'b, 'b> Command<'a, 'b> for AnyCommand<'a, 'b> {
    fn execute(&mut self) -> Result<CommandResult<'a, 'b>, DbError> {
        match self {
            AnyCommand::StringCommand(cmd) => cmd.execute(),
            AnyCommand::IntCommand(cmd) => cmd.execute(),
        }
    }
}

impl DatabaseKey for String {
    fn get_command_value(value: &Self) -> CommandValue<'_> {
        CommandValue::String(value)
    }

    fn from_command_value(value: CommandValue) -> Option<Self> {
        match value {
            CommandValue::String(s) => Some(s.to_owned()),
            _ => None,
        }
    }

    fn field_type() -> FieldType {
        FieldType::String
    }

    fn dbk_new() -> Self {
        String::new()
    }

    fn is_equal_to(&self, other: &Self) -> bool {
        self == other
    }

    fn gramma_from_str(str: &str) -> Option<Self> {
        if str.len() >= 2 {
            return Some(str[1..str.len() - 1].to_string());
        }
        None
    }
}

impl DatabaseKey for i64 {
    fn get_command_value(value: &Self) -> CommandValue<'_> {
        CommandValue::Int(*value)
    }

    fn from_command_value(value: CommandValue) -> Option<Self> {
        match value {
            CommandValue::Int(i) => Some(i),
            _ => None,
        }
    }

    fn field_type() -> FieldType {
        FieldType::Int
    }

    fn dbk_new() -> i64 {
        0
    }

    fn is_equal_to(&self, other: &Self) -> bool {
        self == other
    }

    fn gramma_from_str(str: &str) -> Option<Self> {
        str.parse::<i64>().ok()
    }
}

pub trait DatabaseKey
where
    Self: std::str::FromStr,
    Self: Ord,
    Self: Display,
    Self: Clone,
{
    fn get_command_value(value: &Self) -> CommandValue<'_>;
    fn from_command_value(value: CommandValue) -> Option<Self>;
    fn field_type() -> FieldType;
    fn dbk_new() -> Self;
    fn is_equal_to(&self, other: &Self) -> bool;
    fn gramma_from_str(str: &str) -> Option<Self>;
}

/////////////////////////////////////////////
// tests
/////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    pub fn db_sample() -> Database<String> {
        let mut any_db = StringDatabase(Database::<String>::new());

        let mut create = parse_command(&mut any_db, "CREATE users KEY id FIELDS name: STRING, surname: STRING, age: INT, married: BOOL, credit_score: FLOAT").unwrap();
        create.execute().unwrap();

        let mut insert = parse_command(&mut any_db, "INSERT id='1', name='Jan', surname='Kowalski', age=40, married=true, credit_score=7.8 INTO users").unwrap();
        insert.execute().unwrap();

        let mut insert = parse_command(&mut any_db, "INSERT id='2', name='Ignacy', surname='Nowak', age=29, married=true, credit_score=6.4 INTO users").unwrap();
        insert.execute().unwrap();

        let mut insert = parse_command(&mut any_db, "INSERT id='3', name='Konrad', surname='Adenauer', age=91, married=true, credit_score=2.1 INTO users").unwrap();
        insert.execute().unwrap();

        let StringDatabase(db) = any_db else {
            unreachable!()
        };
        db
    }

    fn assert_db_sample_structure_unchanged(db: &Database<String>) {
        assert!(db.tables.contains_key("users"));
        let table = db.tables.get("users").unwrap();
        assert_eq!(table.key_field, "id");
        assert_eq!(table.types.get("id"), Some(&FieldType::String));
        assert_eq!(table.types.get("name"), Some(&FieldType::String));
        assert_eq!(table.types.get("surname"), Some(&FieldType::String));
        assert_eq!(table.types.get("age"), Some(&FieldType::Int));
        assert_eq!(table.types.get("married"), Some(&FieldType::Bool));
        assert_eq!(table.types.get("credit_score"), Some(&FieldType::Float));
    }

    #[test]
    fn test_create_table() {
        // TODO: fix
        let db = db_sample();
        assert_db_sample_structure_unchanged(&db);
    }

    #[test]
    fn test_create_table_already_exists() {
        let mut db = db_sample();
        let query = CreateQuery {
            table: "users",
            key_field: "id",
            fields_types: vec![NewField {
                field: "pet_name",
                field_type: FieldType::String,
            }],
        };
        let mut command = CreateCommand { db: &mut db, query };

        let result = command.execute();

        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            DbError::ExecutionError(ExecutionError::TableAlreadyExists("users".into()))
        );
        assert_db_sample_structure_unchanged(&db);
    }

    #[test]
    fn test_insert() {
        let mut db = db_sample();
        let table = db.tables.get_mut("users").unwrap();

        let insert_query = InsertQuery {
            table: "users",
            insert_values: vec![
                InsertValue {
                    field: "id",
                    value: CommandValue::String("9999"),
                },
                InsertValue {
                    field: "name",
                    value: CommandValue::String("John"),
                },
                InsertValue {
                    field: "surname",
                    value: CommandValue::String("Doe"),
                },
                InsertValue {
                    field: "age",
                    value: CommandValue::Int(42),
                },
                InsertValue {
                    field: "married",
                    value: CommandValue::Bool(true),
                },
                InsertValue {
                    field: "credit_score",
                    value: CommandValue::Float(9.49),
                },
            ],
        };
        let mut insert_command = InsertCommand {
            table,
            query: insert_query,
        };

        let result = insert_command.execute();
        assert!(result.is_ok());

        let table = db.tables.get("users").unwrap();
        assert!(table.records.contains_key("9999"));
        let record = table.records.get("9999").unwrap();
        assert_eq!(
            record.values.get("name"),
            Some(&Value::String("John".to_string()))
        );
        assert_eq!(
            record.values.get("surname"),
            Some(&Value::String("Doe".to_string()))
        );
        assert_eq!(record.values.get("age"), Some(&Value::Int(42)));
        assert_eq!(record.values.get("married"), Some(&Value::Bool(true)));
        // float comparisons will fail; TODO: implement equality with margin of error
        // assert_eq!(
        //     record.values.get("credit score"),
        //     Some(&Value::Float(9.49))
        // );
    }

    #[test]
    fn test_insert_duplicate_key() {
        let mut db = db_sample();

        let insert_query = InsertQuery {
            table: "users",
            insert_values: vec![
                InsertValue {
                    field: "id",
                    value: CommandValue::String("1"),
                },
                InsertValue {
                    field: "name",
                    value: CommandValue::String("Jane"),
                },
                InsertValue {
                    field: "surname",
                    value: CommandValue::String("Dane"),
                },
                InsertValue {
                    field: "age",
                    value: CommandValue::Int(40),
                },
                InsertValue {
                    field: "married",
                    value: CommandValue::Bool(false),
                },
                InsertValue {
                    field: "credit_score",
                    value: CommandValue::Float(543.21),
                },
            ],
        };
        let mut command = InsertCommand {
            table: db.tables.get_mut("users").unwrap(),
            query: insert_query,
        };
        let result = command.execute();

        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            DbError::ExecutionError(ExecutionError::RecordWithKeyAlreadyExists(
                "1".into(),
                "users".into()
            ))
        );
    }

    #[test]
    fn test_select_all_fields() {
        let db = db_sample();

        let select_query = SelectQuery {
            table: "users",
            fields: SelectFields::AllFields(),
            where_clause: None,
        };
        let mut select_command = SelectCommand {
            table: db.tables.get("users").unwrap(),
            query: select_query,
        };
        let result = select_command.execute();
        assert!(result.is_ok());

        if let Ok(CommandResult::Select(select_result)) = result {
            assert_eq!(select_result.records.len(), 3);
        } else {
            panic!("Expected a SelectResult");
        }
    }

    #[test]
    fn test_select_specific_fields() {
        let db = db_sample();

        let select_query = SelectQuery {
            table: "users",
            fields: SelectFields::Fields(vec!["name", "age"]),
            where_clause: None,
        };
        let mut select_command = SelectCommand {
            table: db.tables.get("users").unwrap(),
            query: select_query,
        };
        let result = select_command.execute();
        assert!(result.is_ok());

        if let Ok(CommandResult::Select(select_result)) = result {
            assert_eq!(select_result.records.len(), 3);
            for record in select_result.records {
                assert_eq!(record.values[0].field_type(), FieldType::String);
                assert_eq!(record.values[1].field_type(), FieldType::Int);
            }
        } else {
            panic!("Expected a SelectResult");
        }
    }

    #[test]
    fn test_delete() {
        let mut db = db_sample();

        let delete_query = DeleteQuery {
            table: "users",
            key: "1".to_string(),
        };
        let mut delete_command = DeleteCommand {
            table: db.tables.get_mut("users").unwrap(),
            query: delete_query,
        };
        let result = delete_command.execute();
        assert!(result.is_ok());

        let table = db.tables.get("users").unwrap();
        assert!(!table.records.contains_key("1"));
        assert!(table.records.contains_key("2"));
    }

    #[test]
    fn test_delete_non_existent_key() {
        let mut db = db_sample();

        let delete_query = DeleteQuery {
            table: "users",
            key: "999".to_string(),
        };
        let mut delete_command = DeleteCommand {
            table: db.tables.get_mut("users").unwrap(),
            query: delete_query,
        };
        let result = delete_command.execute();
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            DbError::ExecutionError(ExecutionError::RecordWithKeyNotFound(
                "999".into(),
                "users".into()
            ))
        );
        let table = db.tables.get("users").unwrap();
        assert_eq!(table.records.len(), 3);
    }
}

#[cfg(test)]
mod select_where_clause_tests {
    use super::*;
    use tests::db_sample;

    fn execute_select_with_where<'a: 'b, 'b>(
        db: &'a Database<String>,
        where_clause: Where<'b>,
    ) -> SelectResult<'a, 'b> {
        let select_query = SelectQuery {
            table: "users",
            fields: SelectFields::Fields(vec!["name", "surname", "age"]),
            where_clause: Some(where_clause),
        };
        let mut select_command = SelectCommand {
            table: db.tables.get("users").unwrap(),
            query: select_query,
        };
        let result = select_command.execute();
        assert!(result.is_ok());

        if let Ok(CommandResult::Select(select_result)) = result {
            select_result
        } else {
            panic!("Expected a SelectResult");
        }
    }

    #[test]
    fn test_select_where_age_greater_than() {
        let db = db_sample();
        let where_clause = Where {
            field: "age",
            op: Op::Greater,
            value: CommandValue::Int(30),
        };

        let select_result = execute_select_with_where(&db, where_clause);

        assert_eq!(select_result.records.len(), 2);
        assert!(
            select_result
                .records
                .iter()
                .all(|r| r.values[2] > CommandValue::Int(30))
        );
    }

    #[test]
    fn test_select_where_surname_equals() {
        let db = db_sample();
        let where_clause = Where {
            field: "surname",
            op: Op::Eq,
            value: CommandValue::String("Nowak"),
        };

        let select_result = execute_select_with_where(&db, where_clause);

        assert_eq!(select_result.records.len(), 1);
        let record = &select_result.records[0];
        assert_eq!(record.values[1], CommandValue::String("Nowak"));
    }
}
