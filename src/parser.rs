use crate::insert::*;
use crate::select::*;
use nom::IResult;
use std::str;

#[derive(Debug, PartialEq)]
pub enum SqlQuery {
    Select(SelectStatement),
    Insert(InsertStatement),
}

#[derive(Debug, PartialEq)]
pub enum ConditionBase {
    Field(String),
    Literal(String),
    Placeholder, // ?
}

#[derive(Debug, PartialEq)]
pub struct ConditionTree {
    pub operator: String,
    pub left: Option<Box<ConditionExpression>>,
    pub right: Option<Box<ConditionExpression>>,
}

#[derive(Debug, PartialEq)]
pub enum ConditionExpression {
    ComparisonOp(ConditionTree),
    LogicalOp(ConditionTree),
    Expr(ConditionBase),
}

pub fn parse_query(input: &str) -> Result<SqlQuery, &str> {
    let q_bytes = String::from(input).into_bytes();

    //TODO(yy du) appropriately pass through errors from nom
    match insertion(&q_bytes) {
        IResult::Done(_, o) => return Ok(SqlQuery::Insert(o)),
        _ => (),
    };

    match selection(&q_bytes) {
        IResult::Done(_, o) => return Ok(SqlQuery::Select(o)),
        _ => (),
    };

    Err("failed to parse query")
}
