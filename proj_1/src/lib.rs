use std::collections::HashMap;
use std::hash::Hash;

pub mod parsing;
use crate::AnyDatabase::{IntDatabase, StringDatabase};
pub use parsing::*;

/////////////////////////////////////////////
// owned query types
/////////////////////////////////////////////

#[derive(Debug)]
pub enum AnyQueryOwned {
    StringQuery(QueryOwned<String>),
    IntQuery(QueryOwned<i64>),
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

#[derive(Debug, PartialEq)]
pub struct CreateQueryOwned {
    pub table: String,
    pub key_field: String,
    pub fields_types: Vec<NewFieldOwned>,
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

#[derive(Debug, PartialEq)]
pub enum SelectFieldsOwned {
    Fields(Vec<String>),
    AllFields(),
}

#[derive(Debug, PartialEq)]
pub struct InsertValueOwned {
    pub field: String,
    pub value: Value,
}

impl<'a> From<&InsertValue<'a>> for InsertValueOwned {
    fn from(value: &InsertValue) -> Self {
        InsertValueOwned {
            field: value.field.into(),
            value: (&value.value).into(),
        }
    }
}

impl<'a> From<&CommandValue<'a>> for Value {
    fn from(value: &CommandValue) -> Self {
        match value {
            CommandValue::Bool(b) => Value::Bool(*b),
            CommandValue::String(s) => Value::String((*s).into()),
            CommandValue::Int(i) => Value::Int(*i),
            CommandValue::Float(f) => Value::Float(*f),
        }
    }
}

impl<'a> From<&SelectFields<'a>> for SelectFieldsOwned {
    fn from(value: &SelectFields<'a>) -> Self {
        match value {
            SelectFields::AllFields() => SelectFieldsOwned::AllFields(),
            SelectFields::Fields(fields) => {
                SelectFieldsOwned::Fields(fields.iter().map(|s| s.to_string()).collect())
            }
        }
    }
}

impl<'a> From<&NewField<'a>> for NewFieldOwned {
    fn from(value: &NewField<'a>) -> Self {
        NewFieldOwned {
            field: value.field.into(),
            field_type: value.field_type,
        }
    }
}

impl<'a> From<SelectFields<'a>> for SelectFieldsOwned {
    fn from(value: SelectFields<'a>) -> Self {
        match value {
            SelectFields::AllFields() => SelectFieldsOwned::AllFields(),
            SelectFields::Fields(fields) => {
                SelectFieldsOwned::Fields(fields.iter().map(|s| (*s).into()).collect())
            }
        }
    }
}

impl<'a> From<CreateQuery<'a>> for CreateQueryOwned {
    fn from(value: CreateQuery<'a>) -> Self {
        CreateQueryOwned {
            table: value.table.into(),
            key_field: value.table.into(),
            fields_types: value.fields_types.iter().map(|t| t.into()).collect(),
        }
    }
}

impl<'a, K: DatabaseKey> From<DeleteQuery<'a, K>> for DeleteQueryOwned<K> {
    fn from(value: DeleteQuery<'a, K>) -> Self {
        DeleteQueryOwned {
            table: value.table.into(),
            key: value.key,
        }
    }
}

impl<'a> From<InsertQuery<'a>> for InsertQueryOwned {
    fn from(value: InsertQuery<'a>) -> Self {
        InsertQueryOwned {
            table: value.table.into(),
            insert_values: value.insert_values.iter().map(|v| v.into()).collect(),
        }
    }
}

impl<'a> From<SelectQuery<'a>> for SelectQueryOwned {
    fn from(value: SelectQuery<'a>) -> Self {
        SelectQueryOwned {
            table: value.table.into(),
            fields: value.fields.into(),
        }
    }
}

impl<'a> From<SaveAsQuery<'a>> for SaveAsQueryOwned {
    fn from(value: SaveAsQuery<'a>) -> Self {
        SaveAsQueryOwned {
            file: value.file.into(),
        }
    }
}

impl<'a> From<ReadFromQuery<'a>> for ReadFromQueryOwned {
    fn from(value: ReadFromQuery<'a>) -> Self {
        ReadFromQueryOwned {
            file: value.file.into(),
        }
    }
}

impl<'a, K: DatabaseKey> From<Query<'a, K>> for QueryOwned<K> {
    fn from(value: Query<'a, K>) -> Self {
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

impl<'a> From<AnyQuery<'a>> for AnyQueryOwned {
    fn from(value: AnyQuery<'a>) -> Self {
        match value {
            AnyQuery::StringQuery(q) => AnyQueryOwned::StringQuery(q.into()),
            AnyQuery::IntQuery(q) => AnyQueryOwned::IntQuery(q.into()),
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
pub struct SaveAs<'a, 'b, K: DatabaseKey> {
    db: &'a Database<K>,
    pub query: SaveAsQuery<'b>,
}

#[derive(Debug)]
pub struct ReadFrom<'a, 'b, K: DatabaseKey> {
    db: &'a Database<K>,
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
    SaveAs(SaveAs<'a, 'b, K>),
    ReadFrom(ReadFrom<'a, 'b, K>),
}

impl<'a, 'b> AnyCommandInternal<'a, 'b, String> {
    fn query(self) -> AnyQuery<'b> {
        match self {
            AnyCommandInternal::Select(c) => AnyQuery::StringQuery(c.query.into()),
            AnyCommandInternal::Create(c) => AnyQuery::StringQuery(c.query.into()),
            AnyCommandInternal::Insert(c) => AnyQuery::StringQuery(c.query.into()),
            AnyCommandInternal::Delete(c) => AnyQuery::StringQuery(c.query.into()),
            AnyCommandInternal::SaveAs(c) => AnyQuery::StringQuery(c.query.into()),
            AnyCommandInternal::ReadFrom(c) => AnyQuery::StringQuery(c.query.into()),
        }
    }
}

impl<'a, 'b> AnyCommandInternal<'a, 'b, i64> {
    fn query(self) -> AnyQuery<'b> {
        match self {
            AnyCommandInternal::Select(c) => AnyQuery::IntQuery(c.query.into()),
            AnyCommandInternal::Create(c) => AnyQuery::IntQuery(c.query.into()),
            AnyCommandInternal::Insert(c) => AnyQuery::IntQuery(c.query.into()),
            AnyCommandInternal::Delete(c) => AnyQuery::IntQuery(c.query.into()),
            AnyCommandInternal::SaveAs(c) => AnyQuery::IntQuery(c.query.into()),
            AnyCommandInternal::ReadFrom(c) => AnyQuery::IntQuery(c.query.into()),
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
            AnyCommand::StringCommand(c) => c.query(),
            AnyCommand::IntCommand(c) => c.query(),
        }
    }
}

fn parse_command_create<'a, 'b, K: DatabaseKey>(
    db: &'a mut Database<K>,
    query: CreateQuery<'b>,
) -> Result<AnyCommandInternal<'a, 'b, K>, String> {
    Ok(AnyCommandInternal::Create(CreateCommand { db, query }))
}

fn parse_command_select<'a, 'b, K: DatabaseKey>(
    db: &'a Database<K>,
    query: SelectQuery<'b>,
) -> Result<AnyCommandInternal<'a, 'b, K>, String> {
    let table = db.tables.get(query.table);
    match table {
        Some(table) => Ok(AnyCommandInternal::Select(SelectCommand { table, query })),
        None => Err("Table not found".to_string()),
    }
}

fn parse_command_insert<'a, 'b, K: DatabaseKey>(
    db: &'a mut Database<K>,
    query: InsertQuery<'b>,
) -> Result<AnyCommandInternal<'a, 'b, K>, String> {
    let table = db.tables.get_mut(query.table);
    match table {
        Some(table) => Ok(AnyCommandInternal::Insert(InsertCommand { table, query })),
        None => Err("Table not found".to_string()),
    }
}

fn parse_command_delete<'a, 'b, K: DatabaseKey>(
    db: &'a mut Database<K>,
    query: DeleteQuery<'b, K>,
) -> Result<AnyCommandInternal<'a, 'b, K>, String> {
    let table = db.tables.get_mut(query.table);
    match table {
        Some(table) => Ok(AnyCommandInternal::Delete(DeleteCommand { table, query })),
        None => Err("Table not found".to_string()),
    }
}

fn parse_command_<'a, 'b, K: DatabaseKey>(
    db: &'a mut Database<K>,
    command: &'b str,
) -> Result<AnyCommandInternal<'a, 'b, K>, String> {
    let query = parse_query::<K>(command);
    match query {
        Ok(query) => match query {
            Query::Create(query) => parse_command_create(db, query),
            Query::Select(query) => parse_command_select(db, query),
            Query::Insert(query) => parse_command_insert(db, query),
            Query::Delete(query) => parse_command_delete(db, query),
            Query::SaveAs(query) => Ok(AnyCommandInternal::SaveAs(SaveAs { db, query })),
            Query::ReadFrom(query) => Ok(AnyCommandInternal::ReadFrom(ReadFrom { db, query })),
        },
        Err(e) => Err(e.to_string()),
    }
}

pub fn parse_command<'a, 'b>(
    db: &'a mut AnyDatabase,
    command: &'b str,
) -> Result<AnyCommand<'a, 'b>, String> {
    match db {
        StringDatabase(db) => parse_command_(db, command).map(AnyCommand::StringCommand),
        IntDatabase(db) => parse_command_(db, command).map(AnyCommand::IntCommand),
    }
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum CommandValue<'a> {
    Bool(bool),
    String(&'a str),
    Int(i64),
    Float(f64),
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

impl Value {
    fn field_type(&self) -> FieldType {
        match self {
            Value::Bool(_) => FieldType::Bool,
            Value::String(_) => FieldType::String,
            Value::Int(_) => FieldType::Int,
            Value::Float(_) => FieldType::Float,
        }
    }
}

#[derive(Debug, Clone)]
struct CommandRecord<'a> {
    values: Vec<CommandValue<'a>>,
}

#[derive(Debug, Clone)]
struct Record {
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
pub struct Database<K: DatabaseKey> {
    tables: HashMap<String, Table<K>>,
    command_history: Vec<AnyQueryOwned>,
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
            command_history: Vec::new(),
        }
    }

    pub fn history_push(&mut self, query: AnyQuery) {
        self.command_history.push(query.into());
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

    pub fn history_push(&mut self, query_owned: AnyQueryOwned) {
        match self {
            StringDatabase(database) => database.command_history.push(query_owned),
            IntDatabase(database) => database.command_history.push(query_owned),
        }
    }
}

#[derive(Debug)]
pub struct SelectResult<'a, 'b> {
    fields: Vec<&'b str>,
    records: Vec<CommandRecord<'a>>,
}

#[derive(Debug)]
pub struct CreateResult {}
#[derive(Debug)]

pub struct DeleteResult {}

#[derive(Debug)]
pub struct InsertResult {}

#[derive(Debug)]
pub enum CommandResult<'a, 'b> {
    Select(SelectResult<'a, 'b>),
    Create(CreateResult),
    Delete(DeleteResult),
    Insert(InsertResult),
}

pub trait Command<'a, 'b> {
    fn execute(&mut self) -> Result<CommandResult<'a, 'b>, String>;
}

impl<'a: 'b, 'b, K: DatabaseKey> Command<'a, 'b> for SelectCommand<'a, 'b, K> {
    fn execute(&mut self) -> Result<CommandResult<'a, 'b>, String> {
        let table = self.table;

        let selected_fields: Vec<&str> = match &self.query.fields {
            SelectFields::AllFields() => {
                let mut fields: Vec<&str> = table.types.keys().map(|k| k.as_str()).collect();
                fields.sort_unstable();
                fields
            }
            SelectFields::Fields(v) => {
                for field_name in v {
                    if !table.types.contains_key(*field_name) {
                        return Err(format!(
                            "Field {} not found in table {}",
                            field_name, self.query.table
                        ));
                    }
                }
                v.to_vec()
            }
        };

        let result_records = table
            .records
            .iter()
            .filter(|(key, record)| {
                self.query
                    .where_clause
                    .as_ref()
                    .is_none_or(|wc| wc.filter(key, record))
            })
            .map(|(key, record)| {
                let mut values = Vec::with_capacity(selected_fields.len());
                for &field_name in &selected_fields {
                    if field_name == table.key_field {
                        let key_val = if let Some(s_key) =
                            (key as &dyn std::any::Any).downcast_ref::<String>()
                        {
                            CommandValue::String(s_key)
                        } else if let Some(i_key) =
                            (key as &dyn std::any::Any).downcast_ref::<i64>()
                        {
                            CommandValue::Int(*i_key)
                        } else {
                            unreachable!(); // Should be only String or i64
                        };
                        values.push(key_val);
                    } else if let Some(val) = record.values.get(field_name) {
                        values.push(val.into());
                    }
                }
                CommandRecord { values }
            })
            .collect();

        Ok(CommandResult::Select(SelectResult {
            fields: selected_fields,
            records: result_records,
        }))
    }
}

impl<'a: 'b, 'b, K: DatabaseKey> Command<'a, 'b> for SaveAs<'a, 'b, K> {
    fn execute(&mut self) -> Result<CommandResult<'a, 'b>, String> {
        println!("{:?}", self.db.command_history);
        // todo!();
        // TODO: implement
        Err("not implemented".to_owned())
    }
}

impl<'a, 'b, K: DatabaseKey> Command<'a, 'b> for ReadFrom<'a, 'b, K> {
    fn execute(&mut self) -> Result<CommandResult<'a, 'b>, String> {
        println!("{:?}", self.db.command_history);
        // todo!();
        // TODO: implement
        Err("not implemented".to_owned())
    }
}

impl<'a: 'b, 'b, K: DatabaseKey> Command<'a, 'b> for CreateCommand<'a, 'b, K> {
    fn execute(&mut self) -> Result<CommandResult<'a, 'b>, String> {
        let existing = self.db.tables.get(self.query.table);
        if existing.is_some() {
            return Err(format!("Table {} already exists", self.query.table));
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
        Ok(CommandResult::Create(CreateResult {}))
    }
}

impl<'a: 'b, 'b, K: DatabaseKey> Command<'a, 'b> for InsertCommand<'a, 'b, K> {
    fn execute(&mut self) -> Result<CommandResult<'a, 'b>, String> {
        /* check if nonexistent fields are present */
        let non_existent: Vec<&InsertValue> = self
            .query
            .insert_values
            .iter()
            .filter(|p| !self.table.types.contains_key(p.field))
            .collect();
        if !non_existent.is_empty() {
            return Err(format!(
                "fields: {:?} do not exists in table: {}",
                non_existent.iter().map(|f| f.field).collect::<Vec<&str>>(),
                self.query.table
            ));
        }

        /* check that all fields are present exactly once (map number of occasions for each field) */
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
            return Err(format!(
                "Fields: {:?} in table: {} are missing",
                missing_fields, self.query.table
            ));
        }

        let duplicated_fields: Vec<&str> = number_of_occurrences
            .iter()
            .filter(|p| *p.1 > 1)
            .map(|p| *p.0)
            .collect();
        if !duplicated_fields.is_empty() {
            return Err(format!(
                "Fields: {:?} in table: {} are present more than once",
                duplicated_fields, self.query.table
            ));
        }

        /* check if types match */
        let non_matching_types: Vec<&InsertValue> = self
            .query
            .insert_values
            .iter()
            .filter(|p| p.value.field_type() != *self.table.types.get(p.field).unwrap())
            .collect();
        if !non_matching_types.is_empty() {
            return Err(format!(
                "fields: {:?} do not match their expected type",
                non_matching_types
            ));
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
        let key_value = K::get_value(key).unwrap();
        /* check if the record already exists */
        if self.table.records.contains_key(&key_value) {
            return Err(format!(
                "record with key: {} already exists in table: {}",
                key_value, self.query.table
            ));
        }

        self.table.records.insert(
            key_value,
            Record {
                values: insert_without_key,
            },
        );

        Ok(CommandResult::Insert(InsertResult {}))
    }
}

impl<'a: 'b, 'b, K: DatabaseKey> Command<'a, 'b> for DeleteCommand<'a, 'b, K> {
    fn execute(&mut self) -> Result<CommandResult<'a, 'b>, String> {
        let delete_res = self.table.records.remove(&self.query.key);
        if delete_res.is_none() {
            return Err(format!(
                "record with key: {} in table: {} not found",
                self.query.key, self.query.table
            ));
        }
        Ok(CommandResult::Delete(DeleteResult {}))
    }
}

impl<'a: 'b, 'b, K: DatabaseKey> Command<'a, 'b> for AnyCommandInternal<'a, 'b, K> {
    fn execute(&mut self) -> Result<CommandResult<'a, 'b>, String> {
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
    fn execute(&mut self) -> Result<CommandResult<'a, 'b>, String> {
        match self {
            AnyCommand::StringCommand(cmd) => cmd.execute(),
            AnyCommand::IntCommand(cmd) => cmd.execute(),
        }
    }
}

impl DatabaseKey for String {
    fn get_value(value: CommandValue) -> Option<Self> {
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
    fn get_value(value: CommandValue) -> Option<Self> {
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
    Self: std::fmt::Display,
    Self: 'static,
{
    fn get_value(value: CommandValue) -> Option<Self>;
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

        let mut insert = parse_command(&mut any_db,"INSERT id='1', name='Jan', surname='Kowalski', age=40, married=true, credit_score=7.8 INTO users").unwrap();
        insert.execute().unwrap();

        let mut insert = parse_command(&mut any_db,"INSERT id='2', name='Ignacy', surname='Nowak', age=29, married=true, credit_score=6.4 INTO users").unwrap();
        insert.execute().unwrap();

        let mut insert = parse_command(&mut any_db,"INSERT id='3', name='Konrad', surname='Adenauer', age=91, married=true, credit_score=2.1 INTO users").unwrap();
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
        assert_eq!(result.err().unwrap(), "Table users already exists");
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
            "record with key: 1 already exists in table: users"
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
            "record with key: 999 in table: users not found"
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
