use nom::{IResult, alphanumeric, eof, line_ending, space};
use std::str;
use crate::select::*;

#[derive(Debug, PartialEq)]
pub enum SqlQuery {
   Select(SelectStatement),
}

#[derive(Debug, PartialEq)]
pub struct ConditionTree {
    pub field: String,
    pub expr: String,
}

named!(csvlist<&[u8], Vec<&str>>,
    many0!(
        map_res!(
            chain!(
            fieldname: alphanumeric ~
            opt!(
                chain!(
                    tag!(",") ~
                    space?,
                    || {}
                )
            ),
            || {fieldname}
            ),
            str::from_utf8)
    )
    );

named!(pub fieldlist<&[u8], Vec<&str>>,
    alt!(
        tag!("*") => {|_| vec!["ALL".into()]}
        | csvlist
    )
);


pub fn parse_query(input: &str) -> Result<SqlQuery, &str> {
    let q_lower = input.to_lowercase();

    //TODO(yy du) appropriately pass through errors from nom
    match selection(&q_lower.into_bytes()) {
        IResult::Done(_, o) => Ok(SqlQuery::Select(o)),
        IResult::Error(_) => Err("parse error"),
        IResult::Incomplete
        (_) => Err("incomplete query"),
    }
}

