use super::*;
use pest::Parser;
use pest_derive::Parser;

// TODO: change .pest to be ascii independent

#[derive(Parser)]
#[grammar = "gramma.pest"]
pub struct GrammaParser;

#[derive(Debug, PartialEq)]
pub enum Query<K: DatabaseKey> {
    Create(CreateQuery),
    Delete(DeleteQuery<K>),
    Insert(InsertQuery),
    Select(SelectQuery),
}

#[derive(Debug, PartialEq)]
pub struct InsertValue {
    pub field: String,
    pub value: Value,
}

#[derive(Debug, PartialEq, Copy, Clone)]
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
            _ => None,
        }
    }
}

pub struct CreateField {
    table: String,
    field: String,
    field_type: FieldType,
}

#[derive(Debug, PartialEq)]
pub struct InsertQuery {
    pub insert_values: Vec<InsertValue>,
    pub table: String,
}

#[derive(Debug, PartialEq)]
pub struct DeleteQuery<K: DatabaseKey> {
    pub key: K,
    pub table: String,
}

#[derive(Debug, PartialEq)]
pub struct NewField {
    pub field: String,
    pub field_type: FieldType,
}

#[derive(Debug, PartialEq)]
pub struct CreateQuery {
    pub table: String,
    pub key_field: String,
    pub fields_types: Vec<NewField>,
}

#[derive(Debug, PartialEq)]
pub struct SelectQuery {
    pub fields: SelectFields,
    pub table: String,
}

#[derive(Debug, PartialEq)]
pub enum SelectFields {
    Fields(Vec<String>),
    AllFields(),
}

/* parse and translate result into custom types for further processing */
pub fn parse_query<K: DatabaseKey>(query: &str) -> Result<Query<K>, String> {
    // parse the passed string; expect one query
    let q = GrammaParser::parse(Rule::Q, query);
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
            let mut fields = SelectFields::Fields(Vec::<String>::new());
            let mut table = String::new();
            for s_inner in q_inner.into_inner() {
                match s_inner.as_rule() {
                    Rule::fields => {
                        for field_name in s_inner.into_inner() {
                            if let SelectFields::Fields(mut fields_borrowed) = fields {
                                fields_borrowed.push(field_name.as_str().to_string());
                                fields = SelectFields::Fields(fields_borrowed);
                            }
                        }
                    }
                    Rule::fields_all => {
                        fields = SelectFields::AllFields();
                    }
                    Rule::table => {
                        table = s_inner.as_str().to_string();
                    }
                    Rule::where_clause => {
                        // TODO: implement
                    }
                    _ => {}
                }
            }
            Ok(Query::Select(SelectQuery { fields, table }))
        }
        Rule::C => {
            let mut table = String::new();
            let mut key_field = String::new();
            let mut fields_types = Vec::<NewField>::new();
            for c_inner in q_inner.into_inner() {
                match c_inner.as_rule() {
                    Rule::table => {
                        table = c_inner.as_str().to_string();
                    }
                    Rule::field => {
                        key_field = c_inner.as_str().to_string();
                    }
                    Rule::fields_types => {
                        let inner = c_inner.into_inner();
                        for inner_inner in inner {
                            if inner_inner.as_rule() == Rule::field_type {
                                let mut field_name = String::new();
                                let mut field_type = FieldType::String;

                                for inner_inner in inner_inner.into_inner() {
                                    match inner_inner.as_rule() {
                                        Rule::field => {
                                            field_name = inner_inner.as_str().to_string();
                                        }
                                        Rule::ftype => {
                                            field_type =
                                                FieldType::from_str(inner_inner.as_str()).unwrap()
                                        }
                                        _ => {}
                                    }
                                }
                                fields_types.push(NewField {
                                    field: field_name,
                                    field_type,
                                })
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Query::Create(CreateQuery {
                table,
                key_field,
                fields_types,
            }))
        }
        Rule::I => {
            let mut table = String::new();
            let mut insert_values = Vec::<InsertValue>::new();
            for inner in q_inner.into_inner() {
                match inner.as_rule() {
                    Rule::table => {
                        table = inner.as_str().to_string();
                    }
                    Rule::field_value_setters => {
                        for inner in inner.into_inner() {
                            let inner_inner = inner.into_inner();
                            let mut field_name = String::new();
                            let mut value = Value::String("".into());
                            for inner_inner in inner_inner {
                                match inner_inner.as_rule() {
                                    Rule::field => {
                                        field_name = inner_inner.as_str().to_string();
                                    }
                                    Rule::field_value => {
                                        let inner = inner_inner.into_inner().next();
                                        if inner.is_none() {
                                            return Err("field was empty".into());
                                        }

                                        let inner = inner.unwrap();
                                        match inner.as_rule() {
                                            Rule::bool => {
                                                let parsed = inner.as_str().to_lowercase().parse();
                                                if let Err(e) = parsed {
                                                    return Err(format!("{}", e));
                                                }
                                                value = Value::Bool(parsed.unwrap());
                                            }
                                            Rule::string => {
                                                if inner.as_str().len() < 2 {
                                                    return Err(format!(
                                                        "string {} is invalid!",
                                                        inner.as_str()
                                                    ));
                                                }
                                                value = Value::String(
                                                    inner.as_str()[1..inner.as_str().len() - 1]
                                                        .to_string(),
                                                );
                                            }
                                            Rule::int => {
                                                let parsed = inner.as_str().parse();
                                                if let Err(e) = parsed {
                                                    return Err(format!("{}", e));
                                                }
                                                value = Value::Int(parsed.unwrap());
                                            }
                                            Rule::float => {
                                                let parsed = inner.as_str().parse();
                                                if let Err(e) = parsed {
                                                    return Err(format!("{}", e));
                                                }
                                                value = Value::Float(parsed.unwrap());
                                            }
                                            _ => {}
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            insert_values.push(InsertValue {
                                field: field_name,
                                value,
                            })
                        }
                    }
                    _ => {}
                }
            }
            Ok(Query::Insert(InsertQuery {
                table,
                insert_values,
            }))
        }
        Rule::D => {
            let mut key: K = K::dbk_new();
            let mut table = String::new();
            for inner in q_inner.into_inner() {
                match inner.as_rule() {
                    Rule::table => {
                        table = inner.as_str().to_string();
                    }
                    Rule::key_value => {
                        key = K::gramma_from_str(inner.as_str()).unwrap();
                    }
                    _ => {}
                };
            }
            Ok(Query::Delete(DeleteQuery { key, table }))
        }
        _ => Err(format!("Unexpected rule {:?}", q_inner)),
    }
}

mod tests {
    use super::*;

    #[test]
    fn select_all() {
        let select = "SELECT * FROM table";
        let expected_result = Query::<String>::Select(SelectQuery {
            table: String::from("table"),
            fields: SelectFields::AllFields(),
        });
        let result = parse_query(select).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn select_one_field() {
        let select = "SELECT field1 FROM table";
        let expected_result = Query::<String>::Select(SelectQuery {
            table: String::from("table"),
            fields: SelectFields::Fields(vec![String::from("field1")]),
        });
        let result = parse_query(select).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn select_multiple_fields() {
        let select = "SELECT field1, field2, field3 FROM my_table";
        let expected_result = Query::<String>::Select(SelectQuery {
            table: String::from("my_table"),
            fields: SelectFields::Fields(vec![
                String::from("field1"),
                String::from("field2"),
                String::from("field3"),
            ]),
        });
        let result = parse_query(select).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn create_single_field() {
        let create_query = "CREATE users KEY id FIELDS name: STRING";
        let expected_result = Query::<String>::Create(CreateQuery {
            table: "users".to_string(),
            key_field: "id".to_string(),
            fields_types: vec![NewField {
                field: "name".to_string(),
                field_type: FieldType::String,
            }],
        });
        let result = parse_query(create_query).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn create_multiple_fields() {
        let create_query = "CREATE products KEY sku FIELDS name: STRING, price: FLOAT, stock: INT";
        let expected_result = Query::<String>::Create(CreateQuery {
            table: "products".to_string(),
            key_field: "sku".to_string(),
            fields_types: vec![
                NewField {
                    field: "name".to_string(),
                    field_type: FieldType::String,
                },
                NewField {
                    field: "price".to_string(),
                    field_type: FieldType::Float,
                },
                NewField {
                    field: "stock".to_string(),
                    field_type: FieldType::Int,
                },
            ],
        });
        let result = parse_query(create_query).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn insert_all_types() {
        let insert_query =
            "INSERT name = 'test_user', age = 30, active = TRUE, score = 99.5 INTO users";
        let expected_result = Query::<String>::Insert(InsertQuery {
            table: "users".to_string(),
            insert_values: vec![
                InsertValue {
                    field: "name".to_string(),
                    value: Value::String("test_user".to_string()),
                },
                InsertValue {
                    field: "age".to_string(),
                    value: Value::Int(30),
                },
                InsertValue {
                    field: "active".to_string(),
                    value: Value::Bool(true),
                },
                InsertValue {
                    field: "score".to_string(),
                    value: Value::Float(99.5),
                },
            ],
        });
        let result = parse_query(insert_query);
        assert_eq!(result.unwrap(), expected_result);
    }

    #[test]
    fn insert_negative_numbers() {
        let insert_query = "INSERT temperature = -10.5, balance = -50 INTO readings";
        let expected_result = Query::<String>::Insert(InsertQuery {
            table: "readings".to_string(),
            insert_values: vec![
                InsertValue {
                    field: "temperature".to_string(),
                    value: Value::Float(-10.5),
                },
                InsertValue {
                    field: "balance".to_string(),
                    value: Value::Int(-50),
                },
            ],
        });
        let result = parse_query(insert_query).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn delete_string_key() {
        let delete_query = "DELETE 'user-123-abc' FROM users";
        let expected_result = Query::<String>::Delete(DeleteQuery {
            key: "user-123-abc".to_string(),
            table: "users".to_string(),
        });
        let result = parse_query::<String>(delete_query).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn delete_int_key() {
        let delete_query = "DELETE 42 FROM products";
        let expected_result = Query::<i64>::Delete(DeleteQuery {
            key: 42,
            table: "products".to_string(),
        });
        let result = parse_query::<i64>(delete_query).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn parse_fail_incomplete_query() {
        let bad_query = "CREATE users KEY";
        let result = parse_query::<String>(bad_query);
        assert!(
            result.is_err(),
            "Expected parsing to fail for incomplete query"
        );
    }

    #[test]
    fn parse_fail_wrong_keyword() {
        let bad_query = "SELECT * TO users";
        let result = parse_query::<String>(bad_query);
        assert!(
            result.is_err(),
            "Expected parsing to fail for incorrect keyword"
        );
    }

    #[test]
    fn parse_fail_malformed_insert() {
        let bad_query = "INSERT name = 'test', age: 30 INTO users";
        let result = parse_query::<String>(bad_query);
        assert!(
            result.is_err(),
            "Expected parsing to fail for malformed INSERT"
        );
    }
}
