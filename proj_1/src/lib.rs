use std::collections::HashMap;
use std::hash::Hash;

pub mod parsing;
use crate::AnyDatabase::{IntDatabase, StringDatabase};
pub use parsing::*;

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
}

#[derive(Debug)]
pub enum AnyCommand<'a, 'b> {
    StringCommand(AnyCommandInternal<'a, 'b, String>),
    IntCommand(AnyCommandInternal<'a, 'b, i64>),
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
        },
        Err(e) => Err(e.to_string()),
    }
}

pub fn parse_command<'a, 'b>(db: &'a mut AnyDatabase, command: &'b str) -> Result<AnyCommand<'a, 'b>, String>  {
    match db {
        StringDatabase(db) => {
            parse_command_(db, command).map(AnyCommand::StringCommand)
        },
        IntDatabase(db) => {
            parse_command_(db, command).map(AnyCommand::IntCommand)
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
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

impl From<&CommandValue<'_>> for Value {
    fn from(value: &CommandValue) -> Self {
        match value {
            CommandValue::Bool(v) => Value::Bool(*v),
            CommandValue::String(v) => Value::String((*v).to_owned()),
            CommandValue::Int(v) => Value::Int(*v),
            CommandValue::Float(v) => Value::Float(*v),
        }
    }
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
    values: HashMap<&'a str, CommandValue<'a>>,
}

#[derive(Debug, Clone)]
struct Record {
    values: HashMap<String, Value>,
}

impl<'a> From<&'a Record> for CommandRecord<'a> {
    fn from(value: &'a Record) -> Self {
        CommandRecord {
            values: value
                .values
                .iter()
                .map(|v| (v.0.as_str(), v.1.into()))
                .collect(),
        }
    }
}

impl From<&CommandRecord<'_>> for Record {
    fn from(value: &CommandRecord) -> Self {
        Record {
            values: value
                .values
                .iter()
                .map(|v| ((*v.0).to_owned(), v.1.into()))
                .collect(),
        }
    }
}

#[derive(Debug)]
pub struct Database<K: DatabaseKey> {
    tables: HashMap<String, Table<K>>,
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
        }
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
}

#[derive(Debug)]
pub struct SelectResult<'a> {
    records: Vec<CommandRecord<'a>>,
}

#[derive(Debug)]
pub struct CreateResult {}
#[derive(Debug)]

pub struct DeleteResult {}

#[derive(Debug)]
pub struct InsertResult {}

#[derive(Debug)]
pub enum CommandResult<'a> {
    Select(SelectResult<'a>),
    Create(CreateResult),
    Delete(DeleteResult),
    Insert(InsertResult),
}

pub trait Command {
    fn execute(&mut self) -> Result<CommandResult<'_>, String>;
}

impl<'a, 'b, K: DatabaseKey> Command for SelectCommand<'a, 'b, K> {
    fn execute(&mut self) -> Result<CommandResult<'a>, String> {
        let table = self.table;
        let result: Vec<CommandRecord> = match &self.query.fields {
            SelectFields::AllFields() => table.records.iter().map(|p| p.1.into()).collect(),
            SelectFields::Fields(v) => table
                .records
                .iter()
                .map(|p| {
                    (
                        p.0,
                        CommandRecord {
                            values: p
                                .1
                                .values
                                .iter()
                                .filter(|r| v.contains(&r.0.as_str()))
                                .map(|r| (r.0.as_str(), r.1.into()))
                                .collect(),
                        },
                    )
                })
                .map(|p| p.1)
                .collect(),
        };
        Ok(CommandResult::Select(SelectResult { records: result }))
    }
}

impl<'a, 'b, K: DatabaseKey> Command for CreateCommand<'a, 'b, K> {
    fn execute(&mut self) -> Result<CommandResult<'_>, String> {
        let existing = self.db.tables.get(self.query.table);
        if existing.is_some() { return Err(format!("Table {} already exists", self.query.table)) }

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

impl<'a, 'b, K: DatabaseKey> Command for InsertCommand<'a, 'b, K> {
    fn execute(&mut self) -> Result<CommandResult<'_>, String> {
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

impl<'a, 'b, K: DatabaseKey> Command for DeleteCommand<'a, 'b, K> {
    fn execute(&mut self) -> Result<CommandResult<'_>, String> {
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

impl<'a, 'b, K: DatabaseKey> Command for AnyCommandInternal<'a, 'b, K> {
    fn execute(&mut self) -> Result<CommandResult<'_>, String> {
        match self {
            AnyCommandInternal::Select(select) => select.execute(),
            AnyCommandInternal::Insert(insert) => insert.execute(),
            AnyCommandInternal::Delete(delete) => delete.execute(),
            AnyCommandInternal::Create(create) => create.execute(),
        }
    }
}

impl<'a, 'b> Command for AnyCommand<'a, 'b> {
    fn execute(&mut self) -> Result<CommandResult<'_>, String> {
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
{
    fn get_value(value: CommandValue) -> Option<Self>;
    fn field_type() -> FieldType;
    fn dbk_new() -> Self;
    fn is_equal_to(&self, other: &Self) -> bool;
    fn gramma_from_str(str: &str) -> Option<Self>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn db_sample() -> Database<String> {
        let mut db = Database::<String>::new();
        let query = CreateQuery {
            table: "users",
            key_field: "id",
            fields_types: vec![
                NewField {
                    field: "name",
                    field_type: FieldType::String,
                },
                NewField {
                    field: "surname",
                    field_type: FieldType::String,
                },
                NewField {
                    field: "age",
                    field_type: FieldType::Int,
                },
                NewField {
                    field: "married",
                    field_type: FieldType::Bool,
                },
                NewField {
                    field: "credit score",
                    field_type: FieldType::Float,
                },
            ],
        };
        let mut command = CreateCommand { db: &mut db, query };
        let res = command.execute();
        assert!(res.is_ok());
        db
    }

    fn db_sample_with_data() -> Database<String> {
        let mut db = db_sample();
        let insert_query1 = InsertQuery {
            table: "users",
            insert_values: vec![
                InsertValue { field: "id", value: CommandValue::String("1") },
                InsertValue { field: "name", value: CommandValue::String("Alice") },
                InsertValue { field: "surname", value: CommandValue::String("A") },
                InsertValue { field: "age", value: CommandValue::Int(30) },
                InsertValue { field: "married", value: CommandValue::Bool(true) },
                InsertValue { field: "credit score", value: CommandValue::Float(100.0) },
            ],
        };
        let mut cmd1 = InsertCommand { table: db.tables.get_mut("users").unwrap(), query: insert_query1 };
        cmd1.execute().unwrap();

        let insert_query2 = InsertQuery {
            table: "users",
            insert_values: vec![
                InsertValue { field: "id", value: CommandValue::String("2") },
                InsertValue { field: "name", value: CommandValue::String("Bob") },
                InsertValue { field: "surname", value: CommandValue::String("B") },
                InsertValue { field: "age", value: CommandValue::Int(25) },
                InsertValue { field: "married", value: CommandValue::Bool(false) },
                InsertValue { field: "credit score", value: CommandValue::Float(200.0) },
            ],
        };
        let mut cmd2 = InsertCommand { table: db.tables.get_mut("users").unwrap(), query: insert_query2 };
        cmd2.execute().unwrap();
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
        assert_eq!(table.types.get("credit score"), Some(&FieldType::Float));
    }

    #[test]
    fn test_create_table() {
        let mut db = db_sample();
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
                InsertValue { field: "id", value: CommandValue::String("1") },
                InsertValue { field: "name", value: CommandValue::String("John") },
                InsertValue { field: "surname", value: CommandValue::String("Doe") },
                InsertValue { field: "age", value: CommandValue::Int(42) },
                InsertValue { field: "married", value: CommandValue::Bool(true) },
                InsertValue { field: "credit score", value: CommandValue::Float(123.45) },
            ],
        };
        let mut insert_command = InsertCommand { table, query: insert_query };

        let result = insert_command.execute();
        assert!(result.is_ok());

        let table = db.tables.get("users").unwrap();
        assert!(table.records.contains_key("1"));
        let record = table.records.get("1").unwrap();
        assert_eq!(record.values.get("name"), Some(&Value::String("John".to_string())));
        assert_eq!(record.values.get("surname"), Some(&Value::String("Doe".to_string())));
        assert_eq!(record.values.get("age"), Some(&Value::Int(42)));
        assert_eq!(record.values.get("married"), Some(&Value::Bool(true)));
        assert_eq!(record.values.get("credit score"), Some(&Value::Float(123.45)));
    }

    #[test]
    fn test_insert_duplicate_key() {
        let mut db = db_sample_with_data();

        let insert_query = InsertQuery {
            table: "users",
            insert_values: vec![
                InsertValue { field: "id", value: CommandValue::String("1") },
                InsertValue { field: "name", value: CommandValue::String("Jane") },
                InsertValue { field: "surname", value: CommandValue::String("Dane") },
                InsertValue { field: "age", value: CommandValue::Int(40) },
                InsertValue { field: "married", value: CommandValue::Bool(false) },
                InsertValue { field: "credit score", value: CommandValue::Float(543.21) },
            ],
        };
        let mut command = InsertCommand { table: db.tables.get_mut("users").unwrap(), query: insert_query };
        let result = command.execute();

        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), "record with key: 1 already exists in table: users");
    }

    #[test]
    fn test_select_all_fields() {
        let db = db_sample_with_data();

        let select_query = SelectQuery {
            table: "users",
            fields: SelectFields::AllFields(),
        };
        let mut select_command = SelectCommand {
            table: db.tables.get("users").unwrap(),
            query: select_query,
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
    fn test_select_specific_fields() {
        let db = db_sample_with_data();

        let select_query = SelectQuery {
            table: "users",
            fields: SelectFields::Fields(vec!["name", "age"]),
        };
        let mut select_command = SelectCommand {
            table: db.tables.get("users").unwrap(),
            query: select_query,
        };
        let result = select_command.execute();
        assert!(result.is_ok());

        if let Ok(CommandResult::Select(select_result)) = result {
            assert_eq!(select_result.records.len(), 2);
            let record1 = &select_result.records[0];
            assert_eq!(record1.values.len(), 2);
            assert!(record1.values.contains_key("name"));
            assert!(record1.values.contains_key("age"));
            let record2 = &select_result.records[1];
            assert_eq!(record2.values.len(), 2);
            assert!(record2.values.contains_key("name"));
            assert!(record2.values.contains_key("age"));
        } else {
            panic!("Expected a SelectResult");
        }
    }

    #[test]
    fn test_delete() {
        let mut db = db_sample_with_data();

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
        let mut db = db_sample_with_data();

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
        assert_eq!(table.records.len(), 2);
    }
}
