use super::query::*;
use super::*;
use crate::query::borrowed::QueryBorrowed;
use pest::Parser;
use pest::iterators::Pair;
use pest_derive::Parser;
use thiserror::Error;

/// Parse related error type.
#[derive(Error, Debug, PartialEq)]
pub enum ParseError {
    /// Indicates that no valid parsing rule was found which is deemed illegal by the grammar.
    #[error("No rule found where it is illegal by grammar")]
    IllegalByGrammarEmpty,

    /// Input encountered `Rule` that is deemed illegal by the grammar.
    /// Carries the unexpected rule.
    #[error("Rule {:?} found where it is illegal by grammar", .0)]
    IllegalByGrammar(Rule),

    /// Input encountered a type deemed illegal by the grammar.
    /// Carries the unexpected type.
    #[error("Type {:?} is illegal by grammar", .0)]
    IllegalByGrammarType(String),

    #[error("Operator {:?} is illegal by grammar", .0)]
    IllegalByGrammarOp(String),

    /// Invalid conversion to an integer value.
    #[error("Illegal by grammar parsing error: {0}")]
    IllegalByGrammarParseIntError(#[from] std::num::ParseIntError),

    /// Invalid string input that lacks proper string delimiters.
    #[error("Illegal by grammar parsing error: string without string delimiters")]
    IllegalByGrammarParseStringError,

    /// Invalid conversion to a floating-point value.
    #[error("Illegal by grammar parsing error: {0}")]
    IllegalByGrammarParseFloatError(#[from] std::num::ParseFloatError),

    /// Invalid conversion to a bool value.
    #[error("Illegal by grammar parsing error: {0}")]
    IllegalByGrammarParseBoolError(#[from] std::str::ParseBoolError),

    /// Error originating from the `pest` library.
    #[error("{0}")]
    ParseError(#[from] Box<pest::error::Error<Rule>>),

    #[error("{0}")]
    TryFromIntError(#[from] std::num::TryFromIntError),
}

/// Pest parser from query grammar
#[derive(Parser)]
#[grammar = "gramma.pest"]
struct GrammaParser;

pub trait ParsableStrType<'a, K>: StrType {
    fn from_pair(pair: &Pair<'a, Rule>) -> Self;
    fn from_str(inner: &'a str) -> Self;
}

impl<'a> ParsableStrType<'a, &'a str> for &'a str {
    fn from_pair(pair: &Pair<'a, Rule>) -> Self {
        pair.as_str()
    }
    fn from_str(inner: &'a str) -> Self {
        inner
    }
}

impl<'a> ParsableStrType<'a, String> for String {
    fn from_pair(pair: &Pair<'a, Rule>) -> Self {
        pair.as_str().to_string()
    }
    fn from_str(inner: &'a str) -> Self {
        inner.to_string()
    }
}

impl FieldType {
    fn from_pair(pair: &Pair<Rule>) -> Result<FieldType, ParseError> {
        let lowercase = pair.as_str().to_lowercase();
        match lowercase.as_str() {
            "bool" => Ok(FieldType::Bool),
            "string" => Ok(FieldType::String),
            "int" => Ok(FieldType::Int),
            "float" => Ok(FieldType::Float),
            t => Err(ParseError::IllegalByGrammarOp(t.into())),
        }
    }
}

fn parse_query_fields<'a, K: ParsableStrType<'a, K>>(
    pairs: pest::iterators::Pairs<'a, Rule>,
) -> Vec<K> {
    pairs.map(|pair| K::from_pair(&pair)).collect()
}

fn parse_query_op(pair: Pair<'_, Rule>) -> Result<Op, ParseError> {
    match pair.as_str() {
        "=" => Ok(Op::Eq),
        "!=" => Ok(Op::Neq),
        ">=" => Ok(Op::GreaterEq),
        "<=" => Ok(Op::LessEq),
        ">" => Ok(Op::Greater),
        "<" => Ok(Op::Less),
        s => Err(ParseError::IllegalByGrammarOp(s.into())),
    }
}

fn parse_order_by<'a, K: ParsableStrType<'a, K>>(
    pairs: pest::iterators::Pairs<'a, Rule>,
) -> Result<OrderBy<K>, ParseError> {
    let mut order_by = OrderBy::default();

    for inner in pairs {
        match inner.as_rule() {
            Rule::field => {
                order_by.field = K::from_pair(&inner);
            }
            Rule::DESC => {
                order_by.descending = true;
            }
            rule => return Err(ParseError::IllegalByGrammar(rule)),
        }
    }
    Ok(order_by)
}

fn parse_limit(pairs: pest::iterators::Pairs<Rule>) -> Result<Limit, ParseError> {
    let mut limit = Limit::default();

    for inner in pairs {
        match inner.as_rule() {
            Rule::value => {
                limit.count = parse_query_value(inner)?.try_into().map_or_else(Err, Ok)?;
            }
            rule => return Err(ParseError::IllegalByGrammar(rule)),
        }
    }
    Ok(limit)
}

fn parse_query_where<'a, K: ParsableStrType<'a, K>>(
    pairs: pest::iterators::Pairs<'a, Rule>,
) -> Result<Where<K>, ParseError> {
    let mut where_caluse: Where<K> = Where::default();

    for inner in pairs {
        match inner.as_rule() {
            Rule::field => {
                where_caluse.field = K::from_pair(&inner);
            }
            Rule::op => {
                where_caluse.op = parse_query_op(inner)?;
            }
            Rule::field_value => {
                where_caluse.value = parse_query_field_value(inner.into_inner())?;
            }
            rule => return Err(ParseError::IllegalByGrammar(rule)),
        }
    }
    Ok(where_caluse)
}

fn parse_query_s<'a, K: ParsableStrType<'a, K>>(
    pairs: pest::iterators::Pairs<'a, Rule>,
) -> Result<Query<K>, ParseError> {
    let mut select = SelectQuery::default();
    for inner in pairs {
        match inner.as_rule() {
            Rule::fields => {
                select.fields = SelectFields::from(parse_query_fields(inner.into_inner()));
            }
            Rule::fields_all => {
                select.fields = SelectFields::AllFields();
            }
            Rule::table => {
                select.table = K::from_pair(&inner);
            }
            Rule::where_clause => {
                select.where_clause = Some(parse_query_where(inner.into_inner())?)
            }
            Rule::order_by => select.order_by = Some(parse_order_by(inner.into_inner())?),
            Rule::limit => select.limit = Some(parse_limit(inner.into_inner())?),
            rule => return Err(ParseError::IllegalByGrammar(rule)),
        }
    }
    Ok(Query::Select(select))
}

fn parse_query_field_types<'a, K: ParsableStrType<'a, K>>(
    pair: Pair<'a, Rule>,
) -> Result<Vec<NewField<K>>, ParseError> {
    let mut fields_types = Vec::<NewField<K>>::new();
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::field_type {
            let mut field_name = K::default();
            let mut field_type = FieldType::String;

            for inner_inner in inner.into_inner() {
                match inner_inner.as_rule() {
                    Rule::field => {
                        field_name = K::from_pair(&inner_inner);
                    }
                    Rule::ftype => field_type = FieldType::from_pair(&inner_inner)?,
                    rule => return Err(ParseError::IllegalByGrammar(rule)),
                }
            }
            fields_types.push(NewField {
                field: field_name,
                field_type,
            })
        }
    }
    Ok(fields_types)
}

fn parse_query_c<'a, K: ParsableStrType<'a, K>>(
    pairs: pest::iterators::Pairs<'a, Rule>,
) -> Result<Query<K>, ParseError> {
    let mut table = K::default();
    let mut key_field = K::default();
    let mut fields_types = Vec::<NewField<K>>::new();
    for inner in pairs {
        match inner.as_rule() {
            Rule::table => {
                table = K::from_pair(&inner);
            }
            Rule::field => {
                key_field = K::from_pair(&inner);
            }
            Rule::fields_types => {
                fields_types = parse_query_field_types(inner)?;
            }
            rule => return Err(ParseError::IllegalByGrammar(rule)),
        }
    }
    Ok(Query::Create(CreateQuery {
        table,
        key_field,
        fields_types: NewFields(fields_types),
    }))
}

fn parse_query_field_value_bool<'a, K: ParsableStrType<'a, K>>(
    pair: Pair<'a, Rule>,
) -> Result<Value<K>, ParseError> {
    let parsed = pair.as_str().to_lowercase().parse();
    Ok(Value::Bool(parsed?))
}

fn parse_query_field_value_string<'a, K: ParsableStrType<'a, K>>(
    pair: Pair<'a, Rule>,
) -> Result<Value<K>, ParseError> {
    let s = pair.as_str();
    if s.len() < 2 {
        return Err(ParseError::IllegalByGrammarParseStringError);
    }
    let inner = &s[1..s.len() - 1];
    let v = K::from_str(inner);
    Ok(Value::String(v))
}

fn parse_query_field_value_int<'a, K: ParsableStrType<'a, K>>(
    pair: Pair<Rule>,
) -> Result<Value<K>, ParseError> {
    let parsed = pair.as_str().parse();
    Ok(Value::Int(parsed?))
}

fn parse_query_value(pair: Pair<Rule>) -> Result<i64, ParseError> {
    let parsed = pair.as_str().parse();
    Ok(parsed?)
}

fn parse_query_field_value_float<'a, K: ParsableStrType<'a, K>>(
    pair: Pair<Rule>,
) -> Result<Value<K>, ParseError> {
    let parsed = pair.as_str().parse();
    Ok(Value::Float(parsed?))
}

fn parse_query_field_value<'a, K: ParsableStrType<'a, K>>(
    pairs: pest::iterators::Pairs<'a, Rule>,
) -> Result<Value<K>, ParseError> {
    let empty_err = || Err(ParseError::IllegalByGrammarEmpty);

    pairs
        .into_iter()
        .next()
        .map_or_else(empty_err, |inner| match inner.as_rule() {
            Rule::bool => parse_query_field_value_bool(inner),
            Rule::string => parse_query_field_value_string(inner),
            Rule::int => parse_query_field_value_int(inner),
            Rule::float => parse_query_field_value_float(inner),
            rule => Err(ParseError::IllegalByGrammar(rule)),
        })
}

fn parse_query_field_value_setters<'a, K: ParsableStrType<'a, K>>(
    pair: Pair<'a, Rule>,
) -> Result<InsertValue<K>, ParseError> {
    let inner_inner = pair.into_inner();
    let mut field = K::default();
    let mut value = Value::String(K::default());
    for inner_inner in inner_inner {
        match inner_inner.as_rule() {
            Rule::field => {
                field = K::from_pair(&inner_inner);
            }
            Rule::field_value => {
                value = parse_query_field_value(inner_inner.into_inner())?;
            }
            _ => {}
        }
    }

    Ok(InsertValue { field, value })
}

fn parse_query_i<'a, K: ParsableStrType<'a, K>>(
    pairs: pest::iterators::Pairs<'a, Rule>,
) -> Result<Query<K>, ParseError> {
    let mut table = K::default();
    let mut insert_values = Vec::<InsertValue<K>>::new();
    for inner in pairs {
        match inner.as_rule() {
            Rule::table => {
                table = K::from_pair(&inner);
            }
            Rule::field_value_setters => {
                for inner in inner.into_inner() {
                    insert_values.push(parse_query_field_value_setters(inner)?);
                }
            }
            rule => return Err(ParseError::IllegalByGrammar(rule)),
        }
    }
    Ok(Query::Insert(InsertQuery {
        table,
        insert_values,
    }))
}

fn parse_query_d<'a, K: ParsableStrType<'a, K>>(
    pairs: pest::iterators::Pairs<'a, Rule>,
) -> Result<Query<K>, ParseError> {
    let mut key = Value::Int(0);
    let mut table = K::default();
    for inner in pairs {
        match inner.as_rule() {
            Rule::table => {
                table = K::from_pair(&inner);
            }
            Rule::key_value => {
                key = parse_query_field_value(inner.into_inner())?;
            }
            rule => return Err(ParseError::IllegalByGrammar(rule)),
        };
    }
    Ok(Query::Delete(DeleteQuery { key, table }))
}

fn parse_query_sa_rf<'a, K: ParsableStrType<'a, K>>(
    pairs: pest::iterators::Pairs<'a, Rule>,
) -> Result<K, ParseError> {
    let mut file = K::default();
    for inner in pairs {
        match inner.as_rule() {
            Rule::file_path => {
                file = K::from_pair(&inner);
            }
            rule => return Err(ParseError::IllegalByGrammar(rule)),
        }
    }
    Ok(file)
}

fn parse_query_sa<'a, K: ParsableStrType<'a, K>>(
    pairs: pest::iterators::Pairs<'a, Rule>,
) -> Result<Query<K>, ParseError> {
    let file = parse_query_sa_rf(pairs)?;
    Ok(Query::SaveAs(SaveAsQuery { file }))
}

fn parse_query_rf<'a, K: ParsableStrType<'a, K>>(
    pairs: pest::iterators::Pairs<'a, Rule>,
) -> Result<Query<K>, ParseError> {
    let file = parse_query_sa_rf(pairs)?;
    Ok(Query::ReadFrom(ReadFromQuery { file }))
}

/// Parse input query and translate the results into internal command representation
pub(crate) fn parse_query(query: &str) -> Result<QueryBorrowed<'_>, ParseError> {
    let mut pairs = GrammaParser::parse(Rule::Q, query).map_err(Box::new)?;

    let query_pair = pairs
        .next()
        .and_then(|p| p.into_inner().next())
        .ok_or(ParseError::IllegalByGrammarEmpty)?;

    let query_rule = query_pair.as_rule();
    let inner_pairs = query_pair.into_inner();
    match query_rule {
        Rule::S => parse_query_s(inner_pairs),
        Rule::C => parse_query_c(inner_pairs),
        Rule::I => parse_query_i(inner_pairs),
        Rule::D => parse_query_d(inner_pairs),
        Rule::SA => parse_query_sa(inner_pairs),
        Rule::RF => parse_query_rf(inner_pairs),
        rule => Err(ParseError::IllegalByGrammar(rule)),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn select_all() {
        let select = "SELECT * FROM table";
        let expected_result = QueryBorrowed::Select(SelectQuery {
            table: "table",
            fields: SelectFields::AllFields(),
            where_clause: None,
            limit: None,
            order_by: None,
        });
        let result = parse_query(select).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn select_all_where() {
        let select = "SELECT * FROM table WHERE avg_rating > 4.5";
        let expected_result = QueryBorrowed::Select(SelectQuery {
            table: "table",
            fields: SelectFields::AllFields(),
            where_clause: Some(Where {
                field: "avg_rating",
                op: Op::Greater,
                value: Value::Float(4.5),
            }),
            limit: None,
            order_by: None,
        });
        let result = parse_query(select).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn select_one_field() {
        let select = "SELECT field1 FROM table";
        let expected_result = QueryBorrowed::Select(SelectQuery {
            table: "table",
            fields: SelectFields::Fields(vec!["field1"]),
            where_clause: None,
            limit: None,
            order_by: None,
        });
        let result = parse_query(select).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn select_one_field_where() {
        let select = "SELECT field1 FROM table WHERE id <= 10";
        let expected_result = QueryBorrowed::Select(SelectQuery {
            table: "table",
            fields: SelectFields::Fields(vec!["field1"]),
            where_clause: Some(Where {
                field: "id",
                op: Op::LessEq,
                value: Value::Int(10),
            }),
            limit: None,
            order_by: None,
        });
        let result = parse_query(select).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn select_multiple_fields() {
        let select = "SELECT field1, field2, field3 FROM my_table";
        let expected_result = QueryBorrowed::Select(SelectQuery {
            table: "my_table",
            fields: SelectFields::Fields(vec!["field1", "field2", "field3"]),
            where_clause: None,
            limit: None,
            order_by: None,
        });
        let result = parse_query(select).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn create_single_field() {
        let create_query = "CREATE users KEY id FIELDS name: STRING";
        let expected_result = QueryBorrowed::Create(CreateQuery {
            table: "users",
            key_field: "id",
            fields_types: NewFields(vec![NewField {
                field: "name",
                field_type: FieldType::String,
            }]),
        });
        let result = parse_query(create_query).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn create_multiple_fields() {
        let create_query = "CREATE products KEY sku FIELDS name: STRING, price: FLOAT, stock: INT";
        let expected_result = QueryBorrowed::Create(CreateQuery {
            table: "products",
            key_field: "sku",
            fields_types: NewFields(vec![
                NewField {
                    field: "name",
                    field_type: FieldType::String,
                },
                NewField {
                    field: "price",
                    field_type: FieldType::Float,
                },
                NewField {
                    field: "stock",
                    field_type: FieldType::Int,
                },
            ]),
        });
        let result = parse_query(create_query).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn insert_all_types() {
        let insert_query =
            "INSERT name = 'test_user', age = 30, active = TRUE, score = 99.5 INTO users";
        let expected_result = Query::Insert(InsertQuery {
            table: "users",
            insert_values: vec![
                InsertValue {
                    field: "name",
                    value: Value::String("test_user"),
                },
                InsertValue {
                    field: "age",
                    value: Value::Int(30),
                },
                InsertValue {
                    field: "active",
                    value: Value::Bool(true),
                },
                InsertValue {
                    field: "score",
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
        let expected_result = Query::Insert(InsertQuery {
            table: "readings",
            insert_values: vec![
                InsertValue {
                    field: "temperature",
                    value: Value::Float(-10.5),
                },
                InsertValue {
                    field: "balance",
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
        let expected_result = Query::Delete(DeleteQuery {
            key: Value::String("user-123-abc"),
            table: "users",
        });
        let result = parse_query(delete_query).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn delete_int_key() {
        let delete_query = "DELETE 42 FROM products";
        let expected_result = Query::Delete(DeleteQuery {
            key: Value::Int(42),
            table: "products",
        });
        let result = parse_query(delete_query).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn parse_fail_select_wrong_keyword() {
        let bad_query = "SELECT * TO users";
        let result = parse_query(bad_query);
        assert!(
            result.is_err(),
            "Expected parsing to fail for incorrect keyword"
        );
    }

    #[test]
    fn parse_fail_create_incomplete() {
        let bad_query = "CREATE users KEY";
        let result = parse_query(bad_query);
        assert!(
            result.is_err(),
            "Expected parsing to fail for incomplete query"
        );
    }

    #[test]
    fn parse_fail_insert_malformed() {
        let bad_query = "INSERT name = 'test', age: 30 INTO users";
        let result = parse_query(bad_query);
        assert!(
            result.is_err(),
            "Expected parsing to fail for malformed INSERT"
        );
    }

    #[test]
    fn save_as_abs_path_unix() {
        let select = "SAVE_AS /home/guest/Documents/my_queries";
        let expected_result = Query::SaveAs(SaveAsQuery {
            file: "/home/guest/Documents/my_queries",
        });
        let result = parse_query(select).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn save_as_relative_path_unix() {
        let select = "SAVE_AS ./Documents/my_queries";
        let expected_result = Query::SaveAs(SaveAsQuery {
            file: "./Documents/my_queries",
        });
        let result = parse_query(select).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn save_as_absolute_path_windows() {
        let select = "SAVE_AS C:\\Documents\\my_queries.txt";
        let expected_result = Query::SaveAs(SaveAsQuery {
            file: "C:\\Documents\\my_queries.txt",
        });
        let result = parse_query(select).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn save_as_relative_path_windows() {
        let select = "SAVE_AS .\\Documents\\my_queries\\session_2.txt";
        let expected_result = Query::SaveAs(SaveAsQuery {
            file: ".\\Documents\\my_queries\\session_2.txt",
        });
        let result = parse_query(select).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn read_from_relative_path_unix() {
        let select = "READ_FROM ./Documents/my_queries/session_2";
        let expected_result = Query::ReadFrom(ReadFromQuery {
            file: "./Documents/my_queries/session_2",
        });
        let result = parse_query(select).unwrap();
        assert_eq!(result, expected_result);
    }
}
