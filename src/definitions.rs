pub enum RusqlStatement {
    AlterTable(AlterTableDef),
    CreateTable(TableDef),
    Delete(DeleteDef),
    DropTable(DropTableDef),
    Insert(InsertDef),
    Select(SelectDef),
    Update(UpdateDef),
}

#[deriving(Copy, Clone)]
pub enum ColumnType {
    Integer,
    Text,
}

#[deriving(Copy, Clone)]
pub enum ColumnConstraint {
    PrimaryKey,
}

#[deriving(Show, Clone, PartialEq)]
pub enum LiteralValue {
    Integer(int),
    Text(String),
    Real(f64),
    Null,
}

pub struct TableDef {
    pub table_name: String,
    pub columns: Vec<ColumnDef>,
}

#[deriving(Copy)]
pub enum ResultColumn {
    Asterisk,
}

pub struct InsertDef {
    pub table_name: String,
    pub column_names: Option<Vec<String>>,
    pub data_source: InsertDataSource,
}

pub enum InsertDataSource {
    Values(Vec<Vec<LiteralValue>>),
    Select(SelectDef),
    DefaultValues,
    Error,
}

#[deriving(Clone)]
pub struct ColumnDef {
    pub name: String,
    pub column_type: Option<ColumnType>,
    pub column_constraints: Vec<ColumnConstraint>,
}

pub struct SelectDef {
    pub result_column: ResultColumn,
    pub table_or_subquery: Vec<String>,
    pub where_expr: Option<Expression>,
}

pub struct DropTableDef {
    pub name: String,
}

pub enum AlterTable {
    RenameTo(String),
    AddColumn(ColumnDef),
}

pub struct AlterTableDef {
    pub name: String,
    pub mode: AlterTable,
}

pub enum Expression {
    LiteralValue(LiteralValue),
    ColumnName(String),
    BinaryOperator((BinaryOperator, Box<Expression>, Box<Expression>)),
}

#[deriving(Copy)]
pub enum BinaryOperator {
    Equals,
}

pub struct DeleteDef {
    pub name: String,
    pub where_expr: Option<Expression>,
}

pub struct UpdateDef {
    pub name: String,
    pub set: Vec<(String, Expression)>,
    pub where_expr: Option<Expression>,
}