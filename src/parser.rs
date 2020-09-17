use crate::select::*;
use nom::IResult;
use std::str;

#[derive(Debug, PartialEq)]
pub enum SqlQuery {
    Select(SelectStatement),
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
    let q_lower = input.to_lowercase();

    //TODO(yy du) appropriately pass through errors from nom
    match selection(&q_lower.into_bytes()) {
        IResult::Done(_, o) => Ok(SqlQuery::Select(o)),
        IResult::Error(_) => Err("parse error"),
        IResult::Incomplete(_) => Err("incomplete query"),
    }
}
