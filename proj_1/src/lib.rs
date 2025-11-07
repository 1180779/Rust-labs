use std::collections::HashMap;
use std::io::Error;
use pest::Parser;
use crate::parsing::{GrammaParser, Rule};

#[derive(Debug)]
enum Value {
    Bool(bool),
    String(String),
    Int(i64),
    Float(f64),
}

#[derive(Debug)]
struct Record {
    values: HashMap<String, Value>,
}

struct Database<K: DatabaseKey> {
    records: std::collections::BTreeMap<K, Record>
}

enum AnyDatabase {
    StringDatabase(Database<String>),
    IntDatabase(Database<i64>)
}

impl DatabaseKey for String {
    fn is_equal_to(&self, other: &Self) -> bool {
        self == other
    }
}

impl DatabaseKey for i64 {
    fn is_equal_to(&self, other: &Self) -> bool {
        self == other
    }
}

pub trait DatabaseKey {
    fn is_equal_to(&self, other: &Self) -> bool;
}

pub struct Interpreter {

}

pub mod parsing {
    use pest::Parser;
    use pest_derive::Parser;

    #[derive(Parser)]
    #[grammar = "gramma.pest"]
    pub struct GrammaParser;
}


/* my gramma representation */

#[derive(Debug)]
pub enum Query<K: DatabaseKey> {
    Create(CreateQuery),
    Delete(DeleteQuery<K>),
    Insert(InsertQuery),
    Select(SelectQuery),
}

#[derive(Debug)]
pub struct InsertValue {
    field: String,
    value: Value,
}

#[derive(Debug)]
pub enum FieldType {
    Bool,
    String,
    Int,
    Float,
}

impl FieldType {
    fn from_str(string: &str) -> Option<FieldType> {
        let lowercase = string.to_lowercase();
        match lowercase.as_str() {
            "bool" => Some(FieldType::Bool),
            "string" => Some(FieldType::String),
            "int" => Some(FieldType::Int),
            "float" => Some(FieldType::Float),
            _ => None
        }
    }
}

pub struct CreateField {
    table: String,
    field: String,
    field_type: FieldType
}

#[derive(Debug)]
pub struct InsertQuery {
    insert_values: Vec<InsertValue>,
    table: String,
}

#[derive(Debug)]
pub struct DeleteQuery<K: DatabaseKey> {
    key: K,
    table: String,
}

#[derive(Debug)]
pub struct NewField {
    field: String,
    field_type: FieldType,
}

#[derive(Debug)]
pub struct CreateQuery {
    table: String,
    key_field: String,
    fields_types: Vec<NewField>,
}

#[derive(Debug)]
pub struct SelectQuery {
    fields: Vec<String>,
    table: String,
}

/* translate parse result into my types */
pub fn ParseQuery<K: DatabaseKey>(query: &str) -> Result<Query<K>, String> {
    let q = GrammaParser::parse(Rule::Q, &query)
        .expect("unsuccessful parse") // unwrap the parse result
        .next()
        .unwrap(); // get and unwrap the `file` rule; never fails
    let q_rule = q.as_rule();
    let q_inner = q.into_inner().next().unwrap();
    match q_inner.as_rule() {
        Rule::S => {
            let mut fields = Vec::<String>::new();
            let mut table = String::new();
            for s_inner in q_inner.into_inner() {
                match s_inner.as_rule() {
                    Rule::fields => {
                        for field in s_inner.into_inner() {
                            fields.push(field.as_str().to_string());
                        }
                    },
                    Rule::table => {
                        table = s_inner.as_str().to_string();
                    },
                    Rule::where_clause => {
                        // TODO: implement
                    },
                    _ => { }
                }
            }
            Ok(Query::Select(SelectQuery {
                fields,
                table,
            }))
        }
        Rule::C => {
            let mut table = String::new();
            let mut key_field = String::new();
            let mut fields_types = Vec::<NewField>::new();
            for c_inner in q_inner.into_inner() {
                match c_inner.as_rule() {
                    Rule::table => {
                        table = c_inner.as_str().to_string();
                    },
                    Rule::field => {
                        key_field = c_inner.as_str().to_string();
                    },
                    Rule::fields_types => {
                        let inner = c_inner.into_inner();
                        for inner_inner in inner {
                            match inner_inner.as_rule() {
                                Rule::field_type => {
                                    let mut field = String::new();
                                    let mut field_type = FieldType::String;

                                    for inner_inner in inner_inner.into_inner() {
                                        match inner_inner.as_rule() {
                                            Rule::field => {
                                                field = inner_inner.as_str().to_string();
                                            },
                                            Rule::ftype => {
                                                field_type = FieldType::from_str(inner_inner.as_str()).unwrap()
                                            },
                                            _ => { }
                                        }
                                    }
                                    fields_types.push(NewField{
                                        field,
                                        field_type
                                    })
                                },
                                _ => { }
                            }
                        }
                    },
                    _ => {},
                }
            }
            Ok(Query::Create(CreateQuery{
                table,
                key_field,
                fields_types
            }))
        }
        _ => {
            Err(format!("Unexpected rule {:?}", q_inner))
        }
    }
}


