use std::collections::HashMap;
use std::io::Error;
use std::os::unix::fs::lchown;
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
    fn new() -> Self {
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
    fn new() -> i64 {
        i64::new()
    }
    fn is_equal_to(&self, other: &Self) -> bool {
        self == other
    }
    fn gramma_from_str(str: &str) -> Option<Self> {
        str.parse::<i64>().ok()
    }
}

pub trait DatabaseKey where Self: std::str::FromStr {
    fn new() -> Self;
    fn is_equal_to(&self, other: &Self) -> bool;
    fn gramma_from_str(str: &str) -> Option<Self>;
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



/* parse and translate result into custom types for further processing */
pub fn parse_query<K: DatabaseKey>(query: &str) -> Result<Query<K>, String> {
    // parse the passed string; expect one query
    let q = GrammaParser::parse(Rule::Q, &query);
    if let Err(e) = q {
        return Err(format!("{}", e));
    }

    let q = q.unwrap().next();
    if q.is_none() {
        return Err("query was empty".into());
    }

    let q = q.unwrap();
    let q_inner = q.into_inner().next();
    if q_inner.is_none() {
        return Err("query was empty".into());
    }

    let q_inner = q_inner.unwrap();
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
                            if inner_inner.as_rule() == Rule::field_type {
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
        Rule::I => {
            let mut table = String::new();
            let mut insert_values = Vec::<InsertValue>::new();
            for inner in q_inner.into_inner() {
                match inner.as_rule() {
                    Rule::table => {
                        table = inner.as_str().to_string();
                    },
                    Rule::field_value_setters => {
                        for inner in inner.into_inner() {
                            let inner_inner = inner.into_inner();
                            let mut field = String::new();
                            let mut value = Value::String("".into());
                            for inner_inner in inner_inner {
                                match inner_inner.as_rule() {
                                    Rule::field => {
                                        field = inner_inner.as_str().to_string();
                                    },
                                    Rule::field_value => {
                                        let inner = inner_inner.into_inner().next().unwrap();
                                        match inner.as_rule() {
                                            Rule::bool => {
                                                value = Value::Bool(inner.as_str().parse().unwrap());
                                            },
                                            Rule::string => {
                                                value = Value::String(inner.as_str()[1..inner.as_str().len() - 1].to_string());
                                            },
                                            Rule::int => {
                                                value = Value::Int(inner.as_str().parse().unwrap());
                                            },
                                            Rule::float => {
                                                value = Value::Float(inner.as_str().parse().unwrap());
                                            },
                                            _ => {}
                                        }
                                    },
                                    _ => { }
                                }
                            }
                            insert_values.push(InsertValue{
                                field,
                                value
                            })
                        }
                    },
                    _ => {}
                }
            }
            Ok(Query::Insert(InsertQuery{
                table,
                insert_values
            }))
        },
        Rule::D => {
            let mut key: K = K::new();
            let mut table = String::new();
            for inner in q_inner.into_inner() {
                match inner.as_rule() {
                    Rule::table => {
                        table = inner.as_str().to_string();
                    },
                    Rule::key_value => {
                        key = K::gramma_from_str(inner.as_str()).unwrap();
                    },
                    _ => {}
                };
            }
            Ok(Query::Delete(DeleteQuery{
                key,
                table
            }))
        }
        _ => {
            Err(format!("Unexpected rule {:?}", q_inner))
        }
    }
}


