use super::*;
use pest::Parser;
use pest::iterators::Pair;
use pest_derive::Parser;

// TODO: change .pest to be ascii independent

#[derive(Parser)]
#[grammar = "gramma.pest"]
struct GrammaParser;

#[derive(Debug, PartialEq)]
pub enum Query<'a, K: DatabaseKey> {
    Create(CreateQuery<'a>),
    Delete(DeleteQuery<'a, K>),
    Insert(InsertQuery<'a>),
    Select(SelectQuery<'a>),
    SaveAs(SaveAsQuery<'a>),
    ReadFrom(ReadFromQuery<'a>),
}

impl<'a, K: DatabaseKey> From<CreateQuery<'a>> for Query<'a, K> {
    fn from(value: CreateQuery<'a>) -> Self {
        Query::Create(value)
    }
}

impl<'a, K: DatabaseKey> From<DeleteQuery<'a, K>> for Query<'a, K> {
    fn from(value: DeleteQuery<'a, K>) -> Self {
        Query::Delete(value)
    }
}

impl<'a, K: DatabaseKey> From<InsertQuery<'a>> for Query<'a, K> {
    fn from(value: InsertQuery<'a>) -> Self {
        Query::Insert(value)
    }
}

impl<'a, K: DatabaseKey> From<SelectQuery<'a>> for Query<'a, K> {
    fn from(value: SelectQuery<'a>) -> Self {
        Query::Select(value)
    }
}

impl<'a, K: DatabaseKey> From<SaveAsQuery<'a>> for Query<'a, K> {
    fn from(value: SaveAsQuery<'a>) -> Self {
        Query::SaveAs(value)
    }
}

impl<'a, K: DatabaseKey> From<ReadFromQuery<'a>> for Query<'a, K> {
    fn from(value: ReadFromQuery<'a>) -> Self {
        Query::ReadFrom(value)
    }
}

#[derive(Debug, PartialEq)]
pub struct InsertValue<'a> {
    pub field: &'a str,
    pub value: CommandValue<'a>,
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

#[derive(Debug, PartialEq)]
pub struct InsertQuery<'a> {
    pub insert_values: Vec<InsertValue<'a>>,
    pub table: &'a str,
}

#[derive(Debug, PartialEq)]
pub struct DeleteQuery<'a, K: DatabaseKey> {
    pub key: K,
    pub table: &'a str,
}

#[derive(Debug, PartialEq)]
pub struct NewField<'a> {
    pub field: &'a str,
    pub field_type: FieldType,
}

#[derive(Debug, PartialEq)]
pub struct CreateQuery<'a> {
    pub table: &'a str,
    pub key_field: &'a str,
    pub fields_types: Vec<NewField<'a>>,
}

#[derive(Debug, PartialEq)]
pub struct SelectQuery<'a> {
    pub fields: SelectFields<'a>,
    pub table: &'a str,
}

#[derive(Debug, PartialEq)]
pub struct SaveAsQuery<'a> {
    pub file: &'a str,
}

#[derive(Debug, PartialEq)]
pub struct ReadFromQuery<'a> {
    pub file: &'a str,
}

#[derive(Debug, PartialEq)]
pub enum SelectFields<'a> {
    Fields(Vec<&'a str>),
    AllFields(),
}

impl<'a> Default for SelectFields<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> SelectFields<'a> {
    pub fn from(v: Vec<&str>) -> SelectFields<'_> {
        SelectFields::Fields(v)
    }

    pub fn new() -> Self {
        SelectFields::Fields(Vec::new())
    }

    pub fn new_all() -> Self {
        SelectFields::AllFields()
    }
}

fn parse_query_field(pair: Pair<'_, Rule>) -> &str {
    pair.as_str()
}

fn parse_query_fields(pairs: pest::iterators::Pairs<'_, Rule>) -> Vec<&str> {
    pairs.map(|pair| parse_query_field(pair)).collect()
}

fn parse_query_s<K: DatabaseKey>(pairs: pest::iterators::Pairs<Rule>) -> Result<Query<K>, String> {
    let mut fields = SelectFields::new();
    let mut table = "";
    for inner in pairs {
        match inner.as_rule() {
            Rule::fields => {
                fields = SelectFields::from(parse_query_fields(inner.into_inner()));
            }
            Rule::fields_all => {
                fields = SelectFields::AllFields();
            }
            Rule::table => {
                table = inner.as_str();
            }
            Rule::where_clause => {
                // TODO: implement
            }
            _ => {}
        }
    }
    Ok(Query::Select(SelectQuery { fields, table }))
}

fn parse_query_field_types(pair: Pair<Rule>) -> Vec<NewField> {
    let mut fields_types = Vec::<NewField>::new();
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::field_type {
            let mut field_name = "";
            let mut field_type = FieldType::String;

            for inner_inner in inner.into_inner() {
                match inner_inner.as_rule() {
                    Rule::field => {
                        field_name = inner_inner.as_str();
                    }
                    Rule::ftype => field_type = FieldType::from_str(inner_inner.as_str()).unwrap(),
                    _ => {}
                }
            }
            fields_types.push(NewField {
                field: field_name,
                field_type,
            })
        }
    }
    fields_types
}

fn parse_query_c<K: DatabaseKey>(pairs: pest::iterators::Pairs<Rule>) -> Result<Query<K>, String> {
    let mut table = "";
    let mut key_field = "";
    let mut fields_types = Vec::<NewField>::new();
    for inner in pairs {
        match inner.as_rule() {
            Rule::table => {
                table = inner.as_str();
            }
            Rule::field => {
                key_field = inner.as_str();
            }
            Rule::fields_types => {
                fields_types = parse_query_field_types(inner);
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

fn parse_query_field_value_bool(pair: Pair<Rule>) -> Option<Result<CommandValue, String>> {
    let parsed = pair.as_str().to_lowercase().parse();
    if let Err(e) = parsed {
        return Some(Err(format!("{}", e)));
    }
    Some(Ok(CommandValue::Bool(parsed.unwrap())))
}

fn parse_query_field_value_string(pair: Pair<Rule>) -> Option<Result<CommandValue, String>> {
    if pair.as_str().len() < 2 {
        return Some(Err(format!("string {} is invalid!", pair.as_str())));
    }
    Some(Ok(CommandValue::String(
        &pair.as_str()[1..pair.as_str().len() - 1],
    )))
}

fn parse_query_field_value_int(pair: Pair<Rule>) -> Option<Result<CommandValue, String>> {
    let parsed = pair.as_str().parse();
    if let Err(e) = parsed {
        return Some(Err(format!("{}", e)));
    }
    Some(Ok(CommandValue::Int(parsed.unwrap())))
}

fn parse_query_field_value_float(pair: Pair<Rule>) -> Option<Result<CommandValue, String>> {
    let parsed = pair.as_str().parse();
    if let Err(e) = parsed {
        return Some(Err(format!("{}", e)));
    }
    Some(Ok(CommandValue::Float(parsed.unwrap())))
}

fn parse_query_field_value(pair: Pair<Rule>) -> Option<Result<CommandValue, String>> {
    match pair.as_rule() {
        Rule::bool => parse_query_field_value_bool(pair),
        Rule::string => parse_query_field_value_string(pair),
        Rule::int => parse_query_field_value_int(pair),
        Rule::float => parse_query_field_value_float(pair),
        _ => None,
    }
}

fn parse_query_field_value_setters(pair: Pair<Rule>) -> Result<InsertValue, String> {
    let inner_inner = pair.into_inner();
    let mut field = "";
    let mut value = CommandValue::String("");
    for inner_inner in inner_inner {
        match inner_inner.as_rule() {
            Rule::field => {
                field = inner_inner.as_str();
            }
            Rule::field_value => {
                let inner = inner_inner.into_inner().next();
                if inner.is_none() {
                    return Err("field was empty".into());
                }

                let inner = inner.unwrap();
                if let Some(inner_res) = parse_query_field_value(inner) {
                    match inner_res {
                        Ok(v) => value = v,
                        Err(e) => return Err(e.to_string()),
                    }
                }
            }
            _ => {}
        }
    }

    Ok(InsertValue { field, value })
}

fn parse_query_i<K: DatabaseKey>(pairs: pest::iterators::Pairs<Rule>) -> Result<Query<K>, String> {
    let mut table = "";
    let mut insert_values = Vec::<InsertValue>::new();
    for inner in pairs {
        match inner.as_rule() {
            Rule::table => {
                table = inner.as_str();
            }
            Rule::field_value_setters => {
                for inner in inner.into_inner() {
                    let res = parse_query_field_value_setters(inner);
                    match res {
                        Ok(v) => insert_values.push(v),
                        Err(e) => return Err(e.to_string()),
                    }
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

/// Parses a query from a `Pair<Rule>` and constructs a `Query<K>` object.
/// It is assumed that Rule is of variant Rule::D (that is pair.as_rule() returns Rule::D
fn parse_query_d<K: DatabaseKey>(pairs: pest::iterators::Pairs<Rule>) -> Result<Query<K>, String> {
    let mut key: K = K::dbk_new();
    let mut table = "";
    for inner in pairs {
        match inner.as_rule() {
            Rule::table => {
                table = inner.as_str();
            }
            Rule::key_value => {
                key = K::gramma_from_str(inner.as_str()).unwrap();
            }
            _ => {}
        };
    }
    Ok(Query::Delete(DeleteQuery { key, table }))
}

fn parse_query_sa<K: DatabaseKey>(pairs: pest::iterators::Pairs<Rule>) -> Result<Query<K>, String> {
    let mut file = "";
    for inner in pairs {
        if inner.as_rule() == Rule::file_path {
            file = inner.as_str();
        };
    }
    Ok(Query::SaveAs(SaveAsQuery { file }))
}

fn parse_query_rf<K: DatabaseKey>(pairs: pest::iterators::Pairs<Rule>) -> Result<Query<K>, String> {
    let mut file = "";
    for inner in pairs {
        if inner.as_rule() == Rule::file_path {
            file = inner.as_str();
        };
    }
    Ok(Query::ReadFrom(ReadFromQuery { file }))
}

/// Parse input query and translate the results into internal command representation
pub(crate) fn parse_query<K: DatabaseKey>(query: &str) -> Result<Query<'_, K>, String> {
    let mut pairs = GrammaParser::parse(Rule::Q, query).map_err(|e| e.to_string())?;

    let query_pair = pairs
        .next()
        .and_then(|p| p.into_inner().next())
        .ok_or("Query was empty or invalid")?;

    let query_rule = query_pair.as_rule();
    let inner_pairs = query_pair.into_inner();
    match query_rule {
        Rule::S => parse_query_s(inner_pairs),
        Rule::C => parse_query_c(inner_pairs),
        Rule::I => parse_query_i(inner_pairs),
        Rule::D => parse_query_d(inner_pairs),
        Rule::SA => parse_query_sa(inner_pairs),
        Rule::RF => parse_query_rf(inner_pairs),
        rule => Err(format!("Unexpected rule {:?}", rule)),
    }
}

mod tests {
    use super::*;

    #[test]
    fn select_all() {
        let select = "SELECT * FROM table";
        let expected_result = Query::<String>::Select(SelectQuery {
            table: "table",
            fields: SelectFields::AllFields(),
        });
        let result = parse_query(select).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn select_one_field() {
        let select = "SELECT field1 FROM table";
        let expected_result = Query::<String>::Select(SelectQuery {
            table: "table",
            fields: SelectFields::Fields(vec!["field1"]),
        });
        let result = parse_query(select).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn select_multiple_fields() {
        let select = "SELECT field1, field2, field3 FROM my_table";
        let expected_result = Query::<String>::Select(SelectQuery {
            table: "my_table",
            fields: SelectFields::Fields(vec!["field1", "field2", "field3"]),
        });
        let result = parse_query(select).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn create_single_field() {
        let create_query = "CREATE users KEY id FIELDS name: STRING";
        let expected_result = Query::<String>::Create(CreateQuery {
            table: "users",
            key_field: "id",
            fields_types: vec![NewField {
                field: "name",
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
            table: "products",
            key_field: "sku",
            fields_types: vec![
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
            table: "users",
            insert_values: vec![
                InsertValue {
                    field: "name",
                    value: CommandValue::String("test_user"),
                },
                InsertValue {
                    field: "age",
                    value: CommandValue::Int(30),
                },
                InsertValue {
                    field: "active",
                    value: CommandValue::Bool(true),
                },
                InsertValue {
                    field: "score",
                    value: CommandValue::Float(99.5),
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
            table: "readings",
            insert_values: vec![
                InsertValue {
                    field: "temperature",
                    value: CommandValue::Float(-10.5),
                },
                InsertValue {
                    field: "balance",
                    value: CommandValue::Int(-50),
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
            table: "users",
        });
        let result = parse_query::<String>(delete_query).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn delete_int_key() {
        let delete_query = "DELETE 42 FROM products";
        let expected_result = Query::<i64>::Delete(DeleteQuery {
            key: 42,
            table: "products",
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

    #[test]
    fn save_as_abs_path_unix() {
        let select = "SAVE_AS /home/guest/Documents/my_queries";
        let expected_result = Query::<String>::SaveAs(SaveAsQuery {
            file: "/home/guest/Documents/my_queries",
        });
        let result = parse_query(select).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn save_as_relative_path_unix() {
        let select = "SAVE_AS ./Documents/my_queries";
        let expected_result = Query::<String>::SaveAs(SaveAsQuery {
            file: "./Documents/my_queries",
        });
        let result = parse_query(select).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn save_as_relative_path_spaces_unix() {
        let select = "SAVE_AS ./Documents/my queries/session_2";
        let expected_result = Query::<String>::SaveAs(SaveAsQuery {
            file: "./Documents/my queries/session_2",
        });
        let result = parse_query(select).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn save_as_absolute_path_windows() {
        let select = "SAVE_AS C:\\Documents\\my_queries.txt";
        let expected_result = Query::<String>::SaveAs(SaveAsQuery {
            file: "C:\\Documents\\my_queries.txt",
        });
        let result = parse_query(select).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn save_as_relative_path_windows() {
        let select = "SAVE_AS .\\Documents\\my_queries\\session_2.txt";
        let expected_result = Query::<String>::SaveAs(SaveAsQuery {
            file: ".\\Documents\\my_queries\\session_2.txt",
        });
        let result = parse_query(select).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn read_from() {
        let select = "READ_FROM ./Documents/my_queries/session_2";
        let expected_result = Query::<String>::ReadFrom(ReadFromQuery {
            file: "./Documents/my_queries/session_2",
        });
        let result = parse_query(select).unwrap();
        assert_eq!(result, expected_result);
    }
}
