use std::fmt::{Debug, Display, Formatter};

mod sealed {
    pub trait Sealed {}
    impl Sealed for String {}
    impl Sealed for &str {}
}

pub trait StrType:
    sealed::Sealed + AsRef<str> + Clone + PartialEq + PartialOrd + Display + Default
{
}

impl StrType for String {}
impl StrType for &str {}

#[derive(Debug, PartialOrd, Clone)]
pub enum Value<K: StrType> {
    Bool(bool),
    String(K),
    Int(i64),
    Float(f64),
}

impl<K: StrType> Default for Value<K> {
    fn default() -> Self {
        Value::Int(0)
    }
}

impl<K: StrType> Display for Value<K> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Bool(v) => write!(f, "{}", v),
            Value::String(v) => write!(f, "\"{}\"", v),
            Value::Int(v) => write!(f, "{}", v),
            Value::Float(v) => write!(f, "{}", v),
        }
    }
}

const FLOAT_EPSILON: f64 = 1e-10;

impl<K: StrType> PartialEq for Value<K> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Float(a), Value::Float(b)) => (a - b).abs() < FLOAT_EPSILON,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Int(a), Value::Int(b)) => a == b,
            _ => false,
        }
    }
}

impl<K: StrType> Value<K> {
    pub fn field_type(&self) -> FieldType {
        match self {
            Value::Bool(_) => FieldType::Bool,
            Value::String(_) => FieldType::String,
            Value::Int(_) => FieldType::Int,
            Value::Float(_) => FieldType::Float,
        }
    }
}

/// Where clause comparison operator
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Op {
    Eq,
    Neq,
    Greater,
    GreaterEq,
    Less,
    LessEq,
}

impl Default for Op {
    fn default() -> Self {
        Op::Eq
    }
}

impl Display for Op {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Op::Eq => write!(f, "="),
            Op::Neq => write!(f, "!="),
            Op::Greater => write!(f, ">"),
            Op::GreaterEq => write!(f, ">="),
            Op::Less => write!(f, "<"),
            Op::LessEq => write!(f, "<="),
        }
    }
}

#[derive(Debug, PartialEq, Default)]
/// Represents a WHERE clause comparison used to filter records.
pub struct Where<K: StrType> {
    /// Field name in the record to evaluate.
    pub field: K,
    /// Comparison operator.
    pub op: Op,
    /// Value to compare against.
    pub value: Value<K>,
}

impl<K: StrType> Where<K> {
    /// Compare the provided `CommandValue` against the clause's stored value using
    /// the clause operator.
    ///
    /// Returns `true` if the comparison holds, otherwise `false`.
    pub fn compare_value(&self, other: &Value<K>) -> bool {
        match self.op {
            Op::Eq => other == &self.value,
            Op::Neq => other != &self.value,
            Op::LessEq => other <= &self.value,
            Op::GreaterEq => other >= &self.value,
            Op::Greater => other > &self.value,
            Op::Less => other < &self.value,
        }
    }
}

/// Parsed query AST encompassing all supported commands.
#[derive(Debug, PartialEq)]
pub enum Query<K: StrType> {
    Create(CreateQuery<K>),
    Delete(DeleteQuery<K>),
    Insert(InsertQuery<K>),
    Select(SelectQuery<K>),
    SaveAs(SaveAsQuery<K>),
    ReadFrom(ReadFromQuery<K>),
}

impl<K: StrType> Display for Query<K> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Query::Select(query) => write!(f, "{}", query),
            Query::Create(query) => write!(f, "{}", query),
            Query::Delete(query) => write!(f, "{}", query),
            Query::Insert(query) => write!(f, "{}", query),
            Query::SaveAs(query) => write!(f, "{}", query),
            Query::ReadFrom(query) => write!(f, "{}", query),
        }
    }
}

impl<K: StrType> Display for SelectFields<K> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SelectFields::Fields(fields) => match fields.len() {
                0 => write!(f, ""),
                1 => write!(f, "{}", fields[0]),
                _ => {
                    for v in &fields[0..fields.len() - 1] {
                        write!(f, "{}, ", v)?;
                    }
                    write!(f, "{}", fields[fields.len() - 1])
                }
            },
            SelectFields::AllFields() => {
                write!(f, "*")
            }
        }
    }
}

impl<K: StrType> Display for InsertValue<K> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}={}", self.field, self.value)
    }
}

impl<K: StrType> Display for SelectQuery<K> {
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

impl<K: StrType> Display for Where<K> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.field, self.op, self.value)
    }
}

impl<K: StrType> Display for CreateQuery<K> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CREATE {} KEY {} FIELDS {}",
            self.table, self.key_field, self.fields_types
        )
    }
}

impl<K: StrType> Display for InsertQuery<K> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "INSERT ")?;
        if self.insert_values.len() == 1 {
            write!(f, "{}", self.insert_values[0])?;
        } else {
            for v in &self.insert_values[0..self.insert_values.len()] {
                write!(f, "{}, ", v)?
            }
            write!(f, "{}", self.insert_values[self.insert_values.len() - 1])?;
        }
        write!(f, "INTO {}", self.table)
    }
}

impl<K: StrType> Display for DeleteQuery<K> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "DELETE {} FROM {}", self.key, self.table)
    }
}

impl<K: StrType> Display for SaveAsQuery<K> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "SAVE_AS {}", self.file)
    }
}

impl<K: StrType> Display for ReadFromQuery<K> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "READ_FROM {}", self.file)
    }
}

/// New field definition for a `CREATE` query.
#[derive(Debug, PartialEq)]
pub struct InsertValue<K: StrType> {
    pub field: K,
    pub value: Value<K>,
}

/// Database field type.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum FieldType {
    Bool,
    String,
    Int,
    Float,
}

impl Display for FieldType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldType::Bool => write!(f, "BOOL"),
            FieldType::String => write!(f, "STRING"),
            FieldType::Int => write!(f, "INT"),
            FieldType::Float => write!(f, "FLOAT"),
        }
    }
}

/// `INSERT` query.
#[derive(Debug, PartialEq)]
pub struct InsertQuery<K: StrType> {
    pub insert_values: Vec<InsertValue<K>>,
    pub table: K,
}

/// `DELETE` query.
#[derive(Debug, PartialEq)]
pub struct DeleteQuery<K: StrType> {
    pub key: Value<K>,
    pub table: K,
}

/// New field definition for `CREATE` query.
#[derive(Debug, PartialEq)]
pub struct NewField<K: StrType> {
    pub field: K,
    pub field_type: FieldType,
}

impl<K: StrType> Display for NewField<K> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.field, self.field_type)
    }
}

#[derive(Debug, PartialEq)]
pub struct NewFields<K: StrType>(pub Vec<NewField<K>>);

impl<K: StrType> Display for NewFields<K> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.0.len() {
            0 => write!(f, ""),
            1 => write!(f, "{}", self.0[0]),
            _ => {
                for v in &self.0[0..self.0.len() - 1] {
                    write!(f, "{}, ", v)?;
                }
                write!(f, "{}", self.0[self.0.len() - 1])
            }
        }
    }
}

/// `CREATE` query.
#[derive(Debug, PartialEq)]
pub struct CreateQuery<K: StrType> {
    pub table: K,
    pub key_field: K,
    pub fields_types: NewFields<K>,
}

/// `SELECT` query.
#[derive(Debug, PartialEq, Default)]
pub struct SelectQuery<K: StrType> {
    pub fields: SelectFields<K>,
    pub table: K,
    pub where_clause: Option<Where<K>>,
    pub order_by: Option<OrderBy<K>>,
    pub limit: Option<Limit>,
}

#[derive(Debug, PartialEq, Default)]
pub struct OrderBy<K: StrType> {
    pub field: K,
    pub descending: bool,
}

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct Limit {
    pub count: usize,
}

/// `SAVE_AS` query.
#[derive(Debug, PartialEq)]
pub struct SaveAsQuery<K: StrType> {
    pub file: K,
}

/// `READ_FROM` query.
#[derive(Debug, PartialEq)]
pub struct ReadFromQuery<K: StrType> {
    pub file: K,
}

/// Fields to be selected in a `SELECT` query.
#[derive(Debug, PartialEq)]
pub enum SelectFields<K: StrType> {
    Fields(Vec<K>),
    AllFields(),
}

impl<K: StrType> Default for SelectFields<K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: StrType> SelectFields<K> {
    pub fn from(v: Vec<K>) -> SelectFields<K> {
        SelectFields::Fields(v)
    }

    pub fn new() -> Self {
        SelectFields::Fields(Vec::new())
    }

    pub fn new_all() -> Self {
        SelectFields::AllFields()
    }
}

#[derive(Debug, Clone)]
pub struct QueryRecord<K: StrType> {
    pub values: Vec<Value<K>>,
}

#[derive(Debug)]
pub struct SelectResult<K: StrType, L: StrType> {
    pub fields: Vec<K>,
    pub records: Vec<QueryRecord<L>>,
}

impl<'a> From<&'a Value<String>> for Value<&'a str> {
    fn from(v: &'a Value<String>) -> Self {
        match v {
            Value::Bool(b) => Value::Bool(*b),
            Value::String(s) => Value::String(s.as_str()),
            Value::Int(i) => Value::Int(*i),
            Value::Float(fl) => Value::Float(*fl),
        }
    }
}

impl<'a> From<&'a Value<&'a str>> for Value<String> {
    fn from(v: &'a Value<&'a str>) -> Self {
        match v {
            Value::Bool(b) => Value::Bool(*b),
            Value::String(s) => Value::String((*s).to_string()),
            Value::Int(i) => Value::Int(*i),
            Value::Float(fl) => Value::Float(*fl),
        }
    }
}

impl From<&Query<&str>> for Query<String> {
    fn from(q: &Query<&str>) -> Self {
        match q {
            Query::Create(query) => Self::Create(query.into()),
            Query::Select(query) => Self::Select(query.into()),
            Query::Insert(query) => Self::Insert(query.into()),
            Query::Delete(query) => Self::Delete(query.into()),
            Query::SaveAs(query) => Self::SaveAs(query.into()),
            Query::ReadFrom(query) => Self::ReadFrom(query.into()),
        }
    }
}

impl From<&CreateQuery<&str>> for CreateQuery<String> {
    fn from(q: &CreateQuery<&str>) -> Self {
        CreateQuery {
            table: q.table.into(),
            key_field: q.key_field.to_string(),
            fields_types: (&q.fields_types).into(),
        }
    }
}

impl From<&SelectQuery<&str>> for SelectQuery<String> {
    fn from(q: &SelectQuery<&str>) -> Self {
        SelectQuery {
            table: q.table.into(),
            fields: (&q.fields).into(),
            where_clause: q.where_clause.as_ref().map(|w| w.into()),
            order_by: q.order_by.as_ref().map(|o| o.into()),
            limit: q.limit,
        }
    }
}

impl From<&InsertQuery<&str>> for InsertQuery<String> {
    fn from(q: &InsertQuery<&str>) -> Self {
        InsertQuery {
            table: q.table.into(),
            insert_values: q
                .insert_values
                .iter()
                .map(|iv| InsertValue {
                    field: iv.field.to_string(),
                    value: (&iv.value).into(),
                })
                .collect(),
        }
    }
}

impl From<&DeleteQuery<&str>> for DeleteQuery<String> {
    fn from(q: &DeleteQuery<&str>) -> Self {
        DeleteQuery {
            table: q.table.into(),
            key: (&q.key).into(),
        }
    }
}

impl From<&SaveAsQuery<&str>> for SaveAsQuery<String> {
    fn from(q: &SaveAsQuery<&str>) -> Self {
        SaveAsQuery {
            file: q.file.into(),
        }
    }
}

impl From<&ReadFromQuery<&str>> for ReadFromQuery<String> {
    fn from(q: &ReadFromQuery<&str>) -> Self {
        ReadFromQuery {
            file: q.file.into(),
        }
    }
}

impl From<&SelectFields<&str>> for SelectFields<String> {
    fn from(f: &SelectFields<&str>) -> Self {
        match f {
            SelectFields::Fields(fields) => {
                SelectFields::Fields(fields.iter().map(|s| s.to_string()).collect())
            }
            SelectFields::AllFields() => SelectFields::AllFields(),
        }
    }
}

impl From<&Where<&str>> for Where<String> {
    fn from(w: &Where<&str>) -> Self {
        Where {
            field: w.field.into(),
            op: w.op,
            value: (&w.value).into(),
        }
    }
}

impl From<&NewFields<&str>> for NewFields<String> {
    fn from(f: &NewFields<&str>) -> Self {
        NewFields(
            f.0.iter()
                .map(|nf| NewField {
                    field: nf.field.to_string(),
                    field_type: nf.field_type,
                })
                .collect(),
        )
    }
}

impl From<&OrderBy<&str>> for OrderBy<String> {
    fn from(value: &OrderBy<&str>) -> Self {
        OrderBy {
            field: value.field.to_string(),
            descending: value.descending,
        }
    }
}

pub mod common {
    pub use super::FieldType;
    pub use super::Op;
}

pub mod borrowed {
    pub use super::common::*;
    use super::*;

    pub type ValueBorrowed<'a> = Value<&'a str>;
    pub type SelectFieldsBorrowed<'a> = SelectFields<&'a str>;
    pub type WhereBorrowed<'a> = Where<&'a str>;
    pub type NewFieldBorrowed<'a> = NewField<&'a str>;
    pub type InsertValueBorrowed<'a> = InsertValue<&'a str>;

    pub type QueryBorrowed<'a> = Query<&'a str>;
    pub type CreateQueryBorrowed<'a> = CreateQuery<&'a str>;
    pub type SelectQueryBorrowed<'a> = SelectQuery<&'a str>;
    pub type InsertQueryBorrowed<'a> = InsertQuery<&'a str>;
    pub type DeleteQueryBorrowed<'a> = DeleteQuery<&'a str>;
    pub type SaveAsQueryBorrowed<'a> = SaveAsQuery<&'a str>;
    pub type ReadFromQueryBorrowed<'a> = ReadFromQuery<&'a str>;
}

pub mod owned {
    pub use super::common::*;
    use super::*;

    pub type ValueOwned = Value<String>;
    pub type SelectFieldsOwned = SelectFields<String>;
    pub type WhereOwned = Where<String>;
    pub type NewFieldOwned = NewField<String>;
    pub type InsertValueOwned = InsertValue<String>;

    pub type QueryOwned = Query<String>;
    pub type CreateQueryOwned = CreateQuery<String>;
    pub type SelectQueryOwned = SelectQuery<String>;
    pub type InsertQueryOwned = InsertQuery<String>;
    pub type DeleteQueryOwned = DeleteQuery<String>;
    pub type SaveAsQueryOwned = SaveAsQuery<String>;
    pub type ReadFromQueryOwned = ReadFromQuery<String>;
}
