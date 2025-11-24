use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::hash::Hash;
use std::io::{Read, Write};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum DbError {
    #[error("execution error")]
    ExecutionError(#[from] ExecutionError),

    #[error("parsing error: {0}")]
    ParseError(#[from] ParseError),
}

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

    #[error("Invalid key type")]
    InvalidKeyType,

    #[error("{0}")]
    IoError(std::io::ErrorKind),
}

impl From<std::io::Error> for ExecutionError {
    fn from(e: std::io::Error) -> Self {
        ExecutionError::IoError(e.kind())
    }
}

use crate::AnyDatabase::{IntDatabase, StringDatabase};

pub mod parser;
mod query;

use crate::query::borrowed::*;
pub use parser::*;
use query::*;

#[derive(Debug)]
pub struct SelectCommand<'a, 'b, K: DatabaseKey> {
    pub table: &'a Table<K>,
    pub query: SelectQueryBorrowed<'b>,
}

#[derive(Debug)]
pub struct InsertCommand<'a, 'b, K: DatabaseKey> {
    pub table: &'a mut Table<K>,
    pub query: InsertQueryBorrowed<'b>,
}

#[derive(Debug)]
pub struct CreateCommand<'a, 'b, K: DatabaseKey> {
    pub db: &'a mut Database<K>,
    pub query: CreateQueryBorrowed<'b>,
}

#[derive(Debug)]
pub struct SaveAsCommand<'a, 'b, K: DatabaseKey> {
    db: &'a Database<K>,
    pub query: SaveAsQueryBorrowed<'b>,
}

#[derive(Debug)]
pub struct ReadFromCommand<'a, 'b, K: DatabaseKey> {
    db: &'a mut Database<K>,
    pub query: ReadFromQueryBorrowed<'b>,
}

#[derive(Debug)]
pub struct DeleteCommand<'a, 'b, K: DatabaseKey> {
    pub table: &'a mut Table<K>,
    pub query: DeleteQueryBorrowed<'b>,
}

#[derive(Debug)]
pub enum AnyCommandInternal<'a, 'b, K: DatabaseKey> {
    Create(CreateCommand<'a, 'b, K>),
    Select(SelectCommand<'a, 'b, K>),
    Insert(InsertCommand<'a, 'b, K>),
    Delete(DeleteCommand<'a, 'b, K>),
    SaveAs(SaveAsCommand<'a, 'b, K>),
    ReadFrom(ReadFromCommand<'a, 'b, K>),
}

impl<'a, 'b, K: DatabaseKey> AnyCommandInternal<'a, 'b, K> {
    pub fn query(self) -> QueryBorrowed<'b> {
        match self {
            AnyCommandInternal::Create(c) => QueryBorrowed::Create(c.query),
            AnyCommandInternal::Select(c) => QueryBorrowed::Select(c.query),
            AnyCommandInternal::Insert(c) => QueryBorrowed::Insert(c.query),
            AnyCommandInternal::Delete(c) => QueryBorrowed::Delete(c.query),
            AnyCommandInternal::SaveAs(c) => QueryBorrowed::SaveAs(c.query),
            AnyCommandInternal::ReadFrom(c) => QueryBorrowed::ReadFrom(c.query),
        }
    }
}

#[derive(Debug)]
pub enum AnyCommand<'a, 'b> {
    StringCommand(AnyCommandInternal<'a, 'b, String>),
    IntCommand(AnyCommandInternal<'a, 'b, i64>),
}

impl<'a, 'b> AnyCommand<'a, 'b> {
    pub fn query(self) -> QueryBorrowed<'b> {
        match self {
            AnyCommand::StringCommand(c) => c.query(),
            AnyCommand::IntCommand(c) => c.query(),
        }
    }
}

fn parse_command_create<'a, 'b, K: DatabaseKey>(
    db: &'a mut Database<K>,
    query: CreateQueryBorrowed<'b>,
) -> Result<AnyCommandInternal<'a, 'b, K>, ExecutionError> {
    Ok(AnyCommandInternal::Create(CreateCommand { db, query }))
}

fn parse_command_select<'a, 'b, K: DatabaseKey>(
    db: &'a Database<K>,
    query: SelectQueryBorrowed<'b>,
) -> Result<AnyCommandInternal<'a, 'b, K>, ExecutionError> {
    let table = db.tables.get(query.table);
    match table {
        Some(table) => Ok(AnyCommandInternal::Select(SelectCommand { table, query })),
        None => Err(ExecutionError::TableNotFound(query.table.into())),
    }
}

fn parse_command_insert<'a, 'b, K: DatabaseKey>(
    db: &'a mut Database<K>,
    query: InsertQueryBorrowed<'b>,
) -> Result<AnyCommandInternal<'a, 'b, K>, ExecutionError> {
    let table = db.tables.get_mut(query.table);
    match table {
        Some(table) => Ok(AnyCommandInternal::Insert(InsertCommand { table, query })),
        None => Err(ExecutionError::TableNotFound(query.table.into())),
    }
}

fn parse_command_delete<'a, 'b, K: DatabaseKey>(
    db: &'a mut Database<K>,
    query: DeleteQueryBorrowed<'b>,
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
    let query = parse_query(command)?;
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

#[derive(Debug, Clone)]
pub struct Record {
    values: HashMap<String, Value<String>>,
}

#[derive(Debug)]
struct CommandHistory(Vec<Query<String>>);

impl CommandHistory {
    fn new() -> Self {
        CommandHistory(vec![])
    }
}

impl Display for CommandHistory {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.0.len() {
            0 => write!(f, ""),
            1 => write!(f, "{}", self.0[0]),
            _ => {
                for v in &self.0[0..self.0.len()] {
                    writeln!(f, "{}", v)?;
                }
                write!(f, "{}", self.0[self.0.len() - 1])
            }
        }
    }
}

#[derive(Debug)]
pub struct Database<K: DatabaseKey> {
    tables: HashMap<String, Table<K>>,
    command_history: CommandHistory,
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

    pub fn history_push(&mut self, query: Query<String>) {
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

    pub fn history_push(&mut self, query: Query<String>) {
        match self {
            StringDatabase(database) => {
                database.command_history.0.push(query);
            }
            IntDatabase(database) => {
                database.command_history.0.push(query);
            }
        }
    }
}

impl<K: StrType, L: StrType> SelectResult<K, L> {
    ///
    /// Pads the given string `s` to fit within a cell of specified `width`.
    ///
    /// The function ensures that the string is formatted with a leading space,
    /// followed by the original string, and padded with trailing spaces
    /// to match the desired width.
    ///
    fn pad_cell(s: &str, width: usize) -> String {
        let mut out = String::new();
        out.push(' ');
        out.push_str(s);
        if width > 1 + s.len() {
            out.push_str(&" ".repeat(width - 1 - s.len()));
        }
        out
    }

    ///
    /// Constructs a 2D vector representation of the data's columns from the records.
    ///
    /// Iterates over the `fields` and `records` to group values from each
    /// record by their corresponding field index, creating a columnar representation
    /// where each inner vector contains all the values for a specific field across all records.
    ///
    /// # Returns
    /// A `Vec<Vec<String>>` where each inner `Vec<String>` represents the values of a single
    /// field (column) across all records:
    /// - The outer vector has the same length as the number of fields.
    /// - Each inner vector corresponds to one field and contains strings representing
    ///   the values in that field for all records.
    ///
    fn build_columns(&self) -> Vec<Vec<String>> {
        let mut cols: Vec<Vec<String>> = Vec::with_capacity(self.fields.len());
        for _ in 0..self.fields.len() {
            cols.push(Vec::new());
        }

        for rec in &self.records {
            for (i, val) in rec.values.iter().enumerate().take(self.fields.len()) {
                cols[i].push(val.to_string());
            }
        }

        cols
    }

    ///
    /// Calculates the widths of each column based on the headers and content of the corresponding columns.
    ///
    /// This function determines the maximum width required for each column in order to properly format
    /// a table-like structure. It calculates the width of each column based on the length of the header
    /// fields (`self.fields`) and the content inside the `cols` parameter.
    /// Additional padding is then added to each column to account for the margin.
    ///
    /// # Parameters
    ///
    /// * `cols`: A slice of vectors, where each inner vector contains strings corresponding to
    ///   the content of a column in the table.
    ///
    /// # Returns
    ///
    /// A `Vec<usize>` where each value represents the width of the corresponding column, including padding.
    ///
    fn calculate_widths(&self, cols: &[Vec<String>]) -> Vec<usize> {
        const MARGIN: usize = 2; // spaces padding (one left, one right)
        let mut widths: Vec<usize> = self.fields.iter().map(|h| h.as_ref().len()).collect();

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

    ///
    /// Constructs a table row separator based on provided column widths and border characters.
    ///
    /// # Parameters
    /// - `widths`: A slice of `usize` values representing the width of each column in the table.
    /// - `left`: A `char` to use for the left border of the separator.
    /// - `mid`: `A char` to use as the separator between columns.
    /// - `right`: A `char` to use for the right border of the separator.
    /// - `horiz`: A `char` to use for the horizontal filling between column separators.
    ///
    /// # Returns
    /// A `String` representing a horizontal separator for a table, constructed with the provided
    /// widths and characters.
    ///
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


    /// Constructs a formatted table row as a string by aligning each cell's content
    /// to the specified column widths and separating them with vertical bars (`│`).
    ///
    /// # Arguments
    ///
    /// * `cells` - A slice of string slices (`&[&str]`) representing the content of each cell in the row.
    /// * `widths` - A slice of column widths (`&[usize]`), where each value specifies the width of the corresponding column.
    ///
    /// # Returns
    ///
    /// A `String` representing the formatted table row. Each cell's content is padded
    /// to match the specified column width and enclosed between vertical bars (`│`).
    ///
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

impl<K: StrType, L: StrType> Display for SelectResult<K, L> {
    ///
    /// Formats the SelectResult structure as a table and writes it to the provided formatter.
    ///
    /// # Parameters
    /// - `f`: A mutable reference to a [`Formatter`].
    ///
    /// # Returns
    /// On success, this function returns `Ok(())`. If there is an error written to the `Formatter`,
    ///  an `std::fmt::Error` is returned.
    ///
    /// # Table Structure
    /// The table is rendered with UTF-8 box-drawing characters for aesthetics. Specifically:
    /// - Corners use characters such as `┌`, `┐`, `└`, and `┘`.
    /// - Horizontal separators are created with `─`.
    /// - Vertical separators use `┬` and `┼` among others to separate columns.
    ///
    /// # Example Output
    /// ```plaintext
    /// ┌──────┬───────┬───────┐
    /// │ Col1 │ Col2  │ Col3  │
    /// ├──────┼───────┼───────┤
    /// │ Val1 │ Val2  │ Val3  │
    /// │ Val4 │ Val5  │ Val6  │
    /// └──────┴───────┴───────┘
    /// ```
    ///
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let cols = self.build_columns();
        let widths = self.calculate_widths(&cols);

        let top_sep = Self::build_separator(&widths, '┌', '┬', '┐', '─');
        writeln!(f, "{}", top_sep)?;

        let header_cells: Vec<&str> = self.fields.iter().map(|h| h.as_ref()).collect();
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
pub enum CommandResult<K: StrType, L: StrType> {
    Select(SelectResult<K, L>),
    Create,
    Delete,
    Insert,
    SaveAs,
    ReadFrom,
}

impl<K: StrType, L: StrType> Display for CommandResult<K, L> {
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

pub trait Command<K: StrType, L: StrType> {
    fn execute(&mut self) -> Result<CommandResult<K, L>, DbError>;
}

impl Where<&str> {
    ///
    /// Apply the where-clause to a `Record`.
    ///
    /// Looks up the field named by the clause in `value.values`. If present,
    /// converts the stored value into a `CommandValue` and compares it using
    /// `compare_value`. Missing fields result in `false`.
    ///
    pub fn filter(&self, value: &Record) -> bool {
        value
            .values
            .get(self.field)
            .map(|v| self.compare_value(&v.into()))
            .unwrap_or(false)
    }
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
    ) -> Result<Vec<QueryRecord<&'a str>>, DbError> {
        let mut sorted_records = self.get_sorted_records();
        self.apply_order_by(&mut sorted_records);

        let mut records: Vec<_> = sorted_records
            .iter()
            .map(|(key, record)| {
                let mut values = Vec::with_capacity(selected_fields.len());
                for &field_name in selected_fields {
                    if field_name == self.table.key_field {
                        values.push(K::get_command_value(key));
                    } else if let Some(val) = record.values.get(field_name) {
                        values.push(val.into());
                    }
                }
                QueryRecord { values }
            })
            .collect();

        if let Some(limit) = self.query.limit {
            records.truncate(limit.count);
        }
        Ok(records)
    }

    fn get_sorted_records(&self) -> Vec<(&'a K, &'a Record)> {
        self
            .table
            .records
            .iter()
            .filter(|(_, record)| {
                self.query
                    .where_clause
                    .as_ref()
                    .is_none_or(|wc| wc.filter(record))
            })
            .collect()
    }

    fn apply_order_by(&self, sorted_records: &mut Vec<(&K, &Record)>) {
        if let Some(order_by) = &self.query.order_by {
            sorted_records.sort_by(|a, b| {
                let a_value = a.1.values.get(order_by.field);
                let b_value = b.1.values.get(order_by.field);
                match (a_value, b_value) {
                    (None, None) => std::cmp::Ordering::Equal,
                    (None, Some(_)) => std::cmp::Ordering::Equal,
                    (Some(_), None) => std::cmp::Ordering::Equal,
                    (Some(a_value), Some(b_value)) => {
                        let ordering = a_value
                            .partial_cmp(b_value)
                            .unwrap_or(std::cmp::Ordering::Equal);
                        match order_by.descending {
                            true => ordering.reverse(),
                            false => ordering,
                        }
                    }
                }
            });
        }
    }
}

impl<'a: 'b, 'b, K: DatabaseKey> Command<&'b str, &'a str> for SelectCommand<'a, 'b, K> {
    fn execute(&mut self) -> Result<CommandResult<&'b str, &'a str>, DbError> {
        let selected_fields: Vec<&'b _> = self.execute_fields()?;
        let result_records: Vec<QueryRecord<&'a _>> = self.execute_select(&selected_fields)?;

        Ok(CommandResult::Select(SelectResult {
            fields: selected_fields,
            records: result_records,
        }))
    }
}

impl<'a: 'b, 'b, K: DatabaseKey> Command<&'b str, &'a str> for SaveAsCommand<'a, 'b, K> {
    fn execute(&mut self) -> Result<CommandResult<&'b str, &'a str>, DbError> {
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

impl<'a, 'b, K: DatabaseKey> Command<&'b str, &'a str> for ReadFromCommand<'a, 'b, K> {
    fn execute(&mut self) -> Result<CommandResult<&'b str, &'a str>, DbError> {
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

impl<'a: 'b, 'b, K: DatabaseKey> Command<&'b str, &'a str> for CreateCommand<'a, 'b, K> {
    fn execute(&mut self) -> Result<CommandResult<&'b str, &'a str>, DbError> {
        let existing = self.db.tables.get(self.query.table);
        if existing.is_some() {
            return Err(ExecutionError::TableAlreadyExists(self.query.table.into()).into());
        }

        let mut field_types: HashMap<String, FieldType> = self
            .query
            .fields_types
            .0
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

impl<'a: 'b, 'b, K: DatabaseKey> InsertCommand<'a, 'b, K> {
    fn check_non_existent(&self) -> Result<(), DbError> {
        let non_existent: Vec<&InsertValue<&str>> = self
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
        Ok(())
    }

    fn count_field_occurrences(&self) -> HashMap<&str, u64> {
        let mut number_of_occurrences: HashMap<&str, u64> =
            self.table.types.iter().map(|t| (t.0.as_str(), 0)).collect();
        number_of_occurrences.insert(&self.table.key_field, 0);

        self.query.insert_values.iter().for_each(|p| {
            let f = number_of_occurrences.get_mut(p.field);
            if let Some(c) = f {
                *c += 1;
            }
        });
        number_of_occurrences
    }

    fn check_missing_fields(
        &self,
        number_of_occurrences: &HashMap<&str, u64>,
    ) -> Result<(), DbError> {
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
        Ok(())
    }

    fn check_duplicated_fields(
        &self,
        number_of_occurrences: &HashMap<&str, u64>,
    ) -> Result<(), DbError> {
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
        Ok(())
    }

    fn check_non_matching_types(&self) -> Result<(), DbError> {
        let non_matching_types: Vec<String> = self
            .query
            .insert_values
            .iter()
            .filter(|p| {
                self.table
                    .types
                    .get(p.field)
                    .is_none_or(|t| &p.value.field_type() != t)
            })
            .map(|p| p.field.to_string())
            .collect();
        if !non_matching_types.is_empty() {
            return Err(
                ExecutionError::InvalidTypes(non_matching_types, self.query.table.into()).into(),
            );
        }
        Ok(())
    }

    fn separate_key_and_other_fields(
        &self,
    ) -> (&InsertValue<&'b str>, HashMap<String, Value<String>>) {
        let mut fields: HashMap<String, Value<String>> =
            HashMap::with_capacity(self.query.insert_values.len());
        let mut key_field: &InsertValue<&'b str> = &self.query.insert_values[0];
        for field in &self.query.insert_values {
            if field.field == self.table.key_field {
                key_field = field;
            } else {
                fields.insert(field.field.into(), (&field.value).into());
            }
        }
        (key_field, fields)
    }
}

impl<'a: 'b, 'b, K: DatabaseKey> Command<&'b str, &'a str> for InsertCommand<'a, 'b, K> {
    fn execute(&mut self) -> Result<CommandResult<&'b str, &'a str>, DbError> {
        self.check_non_existent()?;
        let number_of_occurrences = self.count_field_occurrences();
        self.check_missing_fields(&number_of_occurrences)?;
        self.check_duplicated_fields(&number_of_occurrences)?;
        self.check_non_matching_types()?;

        let (key_field, other_fields) = self.separate_key_and_other_fields();
        let key_value = K::from_command_value(&key_field.value)?;

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
                values: other_fields,
            },
        );

        Ok(CommandResult::Insert)
    }
}

impl<'a: 'b, 'b, K: DatabaseKey> Command<&'b str, &'a str> for DeleteCommand<'a, 'b, K> {
    fn execute(&mut self) -> Result<CommandResult<&'b str, &'a str>, DbError> {
        let delete_res = self
            .table
            .records
            .remove(&K::from_command_value(&self.query.key)?);
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

impl<'a: 'b, 'b, K: DatabaseKey> Command<&'b str, &'a str> for AnyCommandInternal<'a, 'b, K> {
    fn execute(&mut self) -> Result<CommandResult<&'b str, &'a str>, DbError> {
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

impl<'a: 'b, 'b> Command<&'b str, &'a str> for AnyCommand<'a, 'b> {
    fn execute(&mut self) -> Result<CommandResult<&'b str, &'a str>, DbError> {
        match self {
            AnyCommand::StringCommand(cmd) => cmd.execute(),
            AnyCommand::IntCommand(cmd) => cmd.execute(),
        }
    }
}

impl DatabaseKey for String {
    fn get_command_value(value: &Self) -> Value<&'_ str> {
        Value::String(value)
    }

    fn from_command_value(value: &Value<&str>) -> Result<Self, DbError> {
        match value {
            Value::String(s) => Ok((*s).to_owned()),
            _ => Err(ExecutionError::InvalidKeyType.into()),
        }
    }

    fn field_type() -> FieldType {
        FieldType::String
    }
}

impl DatabaseKey for i64 {
    fn get_command_value(value: &Self) -> Value<&str> {
        Value::Int(*value)
    }

    fn from_command_value(value: &Value<&str>) -> Result<Self, DbError> {
        match value {
            Value::Int(i) => Ok(*i),
            _ => Err(ExecutionError::InvalidKeyType.into()),
        }
    }

    fn field_type() -> FieldType {
        FieldType::Int
    }
}

pub trait DatabaseKey
where
    Self: std::str::FromStr,
    Self: Ord,
    Self: Display,
    Self: Clone,
{
    fn get_command_value(value: &Self) -> Value<&str>;
    fn from_command_value(value: &Value<&str>) -> Result<Self, DbError>;
    fn field_type() -> FieldType;
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    pub fn sample_int_db_from_string() -> Database<i64> {
        let mut any_db = IntDatabase(Database::<i64>::new());

        let mut create = parse_command(&mut any_db, "CREATE books KEY id FIELDS title: STRING, author: STRING, year: INT").unwrap();
        create.execute().unwrap();

        let mut insert = parse_command(&mut any_db, "INSERT id=1, title='The Great Gatsby', author='F. Scott Fitzgerald', year=1925 INTO books").unwrap();
        insert.execute().unwrap();

        let mut insert = parse_command(&mut any_db, "INSERT id=2, title='1984', author='George Orwell', year=1949 INTO books").unwrap();
        insert.execute().unwrap();

        let mut insert = parse_command(&mut any_db, "INSERT id=3, title='To Kill a Mockingbird', author='Harper Lee', year=1960 INTO books").unwrap();
        insert.execute().unwrap();

        let IntDatabase(db) = any_db else {
            unreachable!()
        };
        db
    }

    fn assert_int_db_structure_unchanged(db: &Database<i64>) {
        assert!(db.tables.contains_key("books"));
        let table = db.tables.get("books").unwrap();
        assert_eq!(table.key_field, "id");
        assert_eq!(table.types.get("id"), Some(&FieldType::Int));
        assert_eq!(table.types.get("title"), Some(&FieldType::String));
        assert_eq!(table.types.get("author"), Some(&FieldType::String));
        assert_eq!(table.types.get("year"), Some(&FieldType::Int));
    }

    pub fn sample_string_db_from_string() -> Database<String> {
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

    fn assert_string_db_structure_unchanged(db: &Database<String>) {
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

    mod create {
        use super::*;

        #[test]
        fn test_string_create_table_from_string() {
            let db = sample_string_db_from_string();
            assert_string_db_structure_unchanged(&db);
        }

        #[test]
        fn test_int_create_table_from_string() {
            let db = sample_int_db_from_string();
            assert_int_db_structure_unchanged(&db);
        }

        #[test]
        fn test_int_create_table_from_code() {
            let mut db = Database::<i64>::new();
            let create = CreateQuery {
                table: "books",
                key_field: "id",
                fields_types: NewFields(vec![
                    NewField {
                        field: "title",
                        field_type: FieldType::String,
                    },
                    NewField {
                        field: "author",
                        field_type: FieldType::String,
                    },
                    NewField {
                        field: "year",
                        field_type: FieldType::Int,
                    },
                ]),
            };
            let mut command = CreateCommand { db: &mut db, query: create };
            let result = command.execute();
            assert!(result.is_ok());
        }

        #[test]
        fn test_string_create_table_already_exists_from_code() {
            let mut db = sample_string_db_from_string();
            let query = CreateQuery {
                table: "users",
                key_field: "id",
                fields_types: NewFields(vec![NewField {
                    field: "pet_name",
                    field_type: FieldType::String,
                }]),
            };
            let mut command = CreateCommand { db: &mut db, query };

            let result = command.execute();

            assert!(result.is_err());
            assert_eq!(
                result.err().unwrap(),
                DbError::ExecutionError(ExecutionError::TableAlreadyExists("users".into()))
            );
        }
    }

    mod select {
        use super::*;

        #[test]
        fn test_string_select_all_fields_from_code() {
            let db = sample_string_db_from_string();

            let select_query = SelectQuery {
                table: "users",
                fields: SelectFields::AllFields(),
                where_clause: None,
                limit: None,
                order_by: None,
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
        fn test_int_select_specific_fields_from_string() {
            let db = sample_int_db_from_string();

            let select_query = "SELECT title, year FROM books";
            let Query::Select(query) = parse_query(select_query).unwrap() else {
                panic!("Expected SelectQuery");
            };
            let mut select_command = SelectCommand {
                table: db.tables.get("books").unwrap(),
                query,
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
        fn test_string_select_specific_fields_from_code() {
            let db = sample_string_db_from_string();

            let select_query = SelectQuery {
                table: "users",
                fields: SelectFields::Fields(vec!["name", "age", "married"]),
                where_clause: None,
                limit: None,
                order_by: None,
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
                    assert_eq!(record.values[2].field_type(), FieldType::Bool);
                }
            } else {
                panic!("Expected a SelectResult");
            }
        }
    }

    mod select_where {
        use super::*;

        fn execute_select_with_where<'a: 'b, 'b>(
            db: &'a Database<String>,
            where_clause: Where<&'b str>,
        ) -> SelectResult<&'b str, &'a str> {
            let select_query = SelectQuery {
                table: "users",
                fields: SelectFields::Fields(vec!["name", "surname", "age"]),
                where_clause: Some(where_clause),
                limit: None,
                order_by: None,
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
            let db = sample_string_db_from_string();
            let where_clause = Where {
                field: "age",
                op: Op::Greater,
                value: Value::Int(30),
            };

            let select_result = execute_select_with_where(&db, where_clause);

            assert_eq!(select_result.records.len(), 2);
            assert!(
                select_result
                    .records
                    .iter()
                    .all(|r| r.values[2] > Value::Int(30))
            );
        }

        #[test]
        fn test_select_where_surname_equals() {
            let db = sample_string_db_from_string();
            let where_clause = Where {
                field: "surname",
                op: Op::Eq,
                value: Value::String("Nowak"),
            };

            let select_result = execute_select_with_where(&db, where_clause);

            assert_eq!(select_result.records.len(), 1);
            let record = &select_result.records[0];
            assert_eq!(record.values[1], Value::String("Nowak"));
        }
    }

    mod select_order_by_limit {
        use super::*;

        #[test]
        fn test_string_select_limit_from_string() {
            let db = sample_string_db_from_string();

            let select_query = "SELECT name, age FROM users LIMIT 2";
            let Query::Select(query) = parse_query(select_query).unwrap() else {
                panic!("Expected SelectQuery");
            };
            let mut select_command = SelectCommand {
                table: db.tables.get("users").unwrap(),
                query,
            };
            let result = select_command.execute();
            assert!(result.is_ok());

            if let Ok(CommandResult::Select(select_result)) = result {
                assert_eq!(select_result.records.len(), 2);
            } else {
                panic!("Expected a SelectResult");
            }
        }

        #[test]
        fn test_int_select_order_by_desc_limit_from_code() {
            let db = sample_int_db_from_string();

            let select_query = SelectQuery {
                table: "books",
                fields: SelectFields::Fields(vec!["title", "year"]),
                where_clause: None,
                limit: Some(Limit { count: 2 }),
                order_by: Some(OrderBy {
                    field: "year",
                    descending: true,
                }),
            };
            let mut select_command = SelectCommand {
                table: db.tables.get("books").unwrap(),
                query: select_query,
            };
            let result = select_command.execute();
            assert!(result.is_ok());

            if let Ok(CommandResult::Select(select_result)) = result {
                assert_eq!(select_result.records.len(), 2);
                assert!(select_result.records[0].values[1] >= select_result.records[1].values[1]);
            } else {
                panic!("Expected a SelectResult");
            }
        }
    }

    mod insert {
        use super::*;

        #[test]
        fn test_int_insert_from_string() {
            let mut db = sample_int_db_from_string();
            let insert_query = "INSERT id=9999, title='Brave New World', author='Aldous Huxley', year=1932 INTO books";
            let Query::Insert(query) = parse_query(insert_query).unwrap() else {
                panic!("Expected InsertQuery");
            };
            let mut insert_command = InsertCommand {
                table: db.tables.get_mut("books").unwrap(),
                query,
            };
            let result = insert_command.execute();
            assert!(result.is_ok());
            let table = db.tables.get("books").unwrap();
            assert!(table.records.contains_key(&9999));
            let record = table.records.get(&9999).unwrap();
            assert_eq!(
                record.values.get("title"),
                Some(&Value::String("Brave New World".to_string()))
            );
            assert_eq!(
                record.values.get("author"),
                Some(&Value::String("Aldous Huxley".to_string()))
            );
            assert_eq!(record.values.get("year"), Some(&Value::Int(1932)));
        }

        #[test]
        fn test_string_insert_from_code() {
            let mut db = sample_string_db_from_string();
            let table = db.tables.get_mut("users").unwrap();

            let insert_query = InsertQuery {
                table: "users",
                insert_values: vec![
                    InsertValue {
                        field: "id",
                        value: Value::String("9999"),
                    },
                    InsertValue {
                        field: "name",
                        value: Value::String("John"),
                    },
                    InsertValue {
                        field: "surname",
                        value: Value::String("Doe"),
                    },
                    InsertValue {
                        field: "age",
                        value: Value::Int(42),
                    },
                    InsertValue {
                        field: "married",
                        value: Value::Bool(true),
                    },
                    InsertValue {
                        field: "credit_score",
                        value: Value::Float(9.49),
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
            assert_eq!(
                record.values.get("credit_score"),
                Some(&Value::Float(9.49))
            );
        }

        #[test]
        fn test_string_insert_duplicate_key_from_code() {
            let mut db = sample_string_db_from_string();

            let insert_query = InsertQuery {
                table: "users",
                insert_values: vec![
                    InsertValue {
                        field: "id",
                        value: Value::String("1"),
                    },
                    InsertValue {
                        field: "name",
                        value: Value::String("Jane"),
                    },
                    InsertValue {
                        field: "surname",
                        value: Value::String("Dane"),
                    },
                    InsertValue {
                        field: "age",
                        value: Value::Int(40),
                    },
                    InsertValue {
                        field: "married",
                        value: Value::Bool(false),
                    },
                    InsertValue {
                        field: "credit_score",
                        value: Value::Float(543.21),
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
    }

    mod delete {
        use super::*;

        #[test]
        fn test_int_delete_from_string() {
            let mut db = sample_int_db_from_string();

            let delete_query = "DELETE 1 FROM books";
            let Query::Delete(query) = parse_query(delete_query).unwrap() else {
                panic!("Expected DeleteQuery");
            };
            let mut delete_command = DeleteCommand {
                table: db.tables.get_mut("books").unwrap(),
                query,
            };
            let result = delete_command.execute();
            assert!(result.is_ok());

            let table = db.tables.get("books").unwrap();
            assert!(!table.records.contains_key(&1));
            assert!(table.records.contains_key(&2));
            assert!(table.records.contains_key(&3));
        }

        #[test]
        fn test_string_delete_from_code() {
            let mut db = sample_string_db_from_string();

            let delete_query = DeleteQuery {
                table: "users",
                key: Value::String("1"),
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
        fn test_string_delete_non_existent_key_from_code() {
            let mut db = sample_string_db_from_string();

            let delete_query = DeleteQuery {
                table: "users",
                key: Value::String("999"),
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
                    Value::String("999").to_string(),
                    "users".into()
                ))
            );
            let table = db.tables.get("users").unwrap();
            assert_eq!(table.records.len(), 3);
        }
    }
}

