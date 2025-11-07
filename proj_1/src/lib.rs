use std::collections::HashMap;
use std::hash::Hash;

pub mod parsing;
use crate::AnyDatabase::{IntDatabase, StringDatabase};
pub use parsing::*;

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
struct Record {
    values: HashMap<String, Value>,
}

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

pub struct Table<K: DatabaseKey> {
    key_field: String,
    types: HashMap<String, FieldType>,
    records: std::collections::BTreeMap<K, Record>,
}

#[derive(Debug)]
pub enum AnyQuery {
    StringQuery(Query<String>),
    IntQuery(Query<i64>),
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

    fn execute_string(&mut self, query: Query<String>) -> Result<QueryResult, String> {
        match self {
            &mut StringDatabase(ref mut db) => db.execute(query),
            _ => Err("Query type no valid for this database type".to_string()),
        }
    }

    fn execute_i64(&mut self, query: Query<i64>) -> Result<QueryResult, String> {
        match self {
            &mut IntDatabase(ref mut db) => db.execute(query),
            _ => Err("Query type no valid for this database type".to_string()),
        }
    }

    pub fn execute(&mut self, query: AnyQuery) -> Result<QueryResult, String> {
        match query {
            AnyQuery::StringQuery(query) => self.execute_string(query),
            AnyQuery::IntQuery(query) => self.execute_i64(query),
        }
    }
}

#[derive(Debug)]
pub struct SelectResult {
    records: Vec<Record>,
}

#[derive(Debug)]
pub struct CreateResult {}
#[derive(Debug)]

pub struct DeleteResult {
    nr_rows: u64,
}

#[derive(Debug)]
pub struct InsertResult {
    nr_rows: u64,
}

#[derive(Debug)]
pub enum QueryResult {
    Select(SelectResult),
    Create(CreateResult),
    Delete(DeleteResult),
    Insert(InsertResult),
}

impl<K: DatabaseKey> Database<K> {
    fn execute(&mut self, query: Query<K>) -> Result<QueryResult, String> {
        match query {
            Query::Select(select) => Ok(QueryResult::Select(self.execute_select(select)?)),
            Query::Create(create) => Ok(QueryResult::Create(self.execute_create(create)?)),
            Query::Delete(delete) => Ok(QueryResult::Delete(self.execute_delete(delete)?)),
            Query::Insert(insert) => Ok(QueryResult::Insert(self.execute_insert(insert)?)),
        }
    }

    fn execute_select(&self, select: SelectQuery) -> Result<SelectResult, String> {
        let table = self.tables.get(&select.table);
        if table.is_none() {
            return Err(format!("table: {} not found", select.table));
        };

        let table = table.unwrap();
        let result: Vec<Record> = match select.fields {
            SelectFields::AllFields() => table.records.iter().map(|p| (*p.1).clone()).collect(),
            SelectFields::Fields(v) => table
                .records
                .iter()
                .map(|p| {
                    (
                        p.0,
                        Record {
                            values: p
                                .1
                                .values
                                .iter()
                                .filter(|r| v.contains(r.0))
                                .map(|r| ((*r.0).clone(), (*r.1).clone()))
                                .collect(),
                        },
                    )
                })
                .map(|p| p.1)
                .collect(),
        };
        Ok(SelectResult { records: result })
    }

    fn execute_create(&mut self, create: CreateQuery) -> Result<CreateResult, String> {
        let table = self.tables.get(&create.table);
        if table.is_some() {
            return Err(format!("table: {} already exists", create.table));
        }

        let mut field_types: HashMap<String, FieldType> = create
            .fields_types
            .iter()
            .map(|e| (e.field.clone(), e.field_type))
            .collect();
        field_types.insert(create.key_field.clone(), K::field_type());
        let new_table = Table {
            key_field: create.key_field,
            types: field_types,
            records: std::collections::BTreeMap::new(),
        };
        self.tables.insert(create.table, new_table);
        Ok(CreateResult {})
    }

    fn execute_insert(&mut self, insert: InsertQuery) -> Result<InsertResult, String> {
        let table = self.tables.get_mut(&insert.table);
        if table.is_none() {
            return Err(format!("table: {} not found", insert.table));
        }
        let table = table.unwrap();

        /* check if nonexistent fields are present */
        let non_existent: Vec<&InsertValue> = insert
            .insert_values
            .iter()
            .filter(|p| !table.types.contains_key(&p.field))
            .collect();
        if !non_existent.is_empty() {
            return Err(format!(
                "fields: {:?} do not exists in table: {}",
                non_existent
                    .iter()
                    .map(|f| f.field.as_str())
                    .collect::<Vec<&str>>(),
                insert.table
            ));
        }

        /* check that all fields are present exactly once (map number of occasions for each field) */
        let mut number_of_occurrences: HashMap<&str, u64> =
            table.types.iter().map(|t| (t.0.as_str(), 0)).collect();
        number_of_occurrences.insert(&table.key_field, 0);

        insert.insert_values.iter().for_each(|p| {
            let f = number_of_occurrences.get_mut(p.field.as_str());
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
                missing_fields, insert.table
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
                duplicated_fields, insert.table
            ));
        }

        /* check if types match */
        let non_matching_types: Vec<&InsertValue> = insert
            .insert_values
            .iter()
            .filter(|p| p.value.field_type() != *table.types.get(&p.field).unwrap())
            .collect();
        if !non_matching_types.is_empty() {
            return Err(format!(
                "fields: {:?} do not match their expected type",
                non_matching_types
            ));
        }

        let insert_without_key: HashMap<String, Value> = insert
            .insert_values
            .iter()
            .filter(|p| p.field != table.key_field)
            .map(|p| (p.field.clone(), p.value.clone()))
            .collect();
        let key = insert
            .insert_values
            .iter()
            .find(|p| p.field == table.key_field)
            .map(|p| p.value.clone())
            .unwrap();
        let key_value = K::get_value(key).unwrap();
        /* check if the record already exists */
        if table.records.contains_key(&key_value) {
            return Err(format!(
                "record with key: {} already exists in table: {}",
                key_value, insert.table
            ));
        }

        table.records.insert(
            key_value,
            Record {
                values: insert_without_key,
            },
        );

        Ok(InsertResult { nr_rows: 1 })
    }

    fn execute_delete(&mut self, delete: DeleteQuery<K>) -> Result<DeleteResult, String> {
        let table = self.tables.get_mut(&delete.table);
        if table.is_none() {
            return Err(format!("table: {} not found", delete.table));
        }
        let table = table.unwrap();
        let delete_res = table.records.remove(&delete.key);
        if delete_res.is_none() {
            return Err(format!(
                "record with key: {} in table: {} not found",
                delete.key, delete.table
            ));
        }
        Ok(DeleteResult { nr_rows: 1 })
    }
}

impl DatabaseKey for String {
    fn get_value(value: Value) -> Option<Self> {
        match value {
            Value::String(s) => Some(s),
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
    fn get_value(value: Value) -> Option<Self> {
        match value {
            Value::Int(i) => Some(i),
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
    fn get_value(value: Value) -> Option<Self>;
    fn field_type() -> FieldType;
    fn dbk_new() -> Self;
    fn is_equal_to(&self, other: &Self) -> bool;
    fn gramma_from_str(str: &str) -> Option<Self>;
}
