use crate::common::{fieldlist, statement_terminator, table_reference, unsigned_number};
use crate::condition::*;
use crate::parser::ConditionExpression;
use nom::line_ending;
use nom::{multispace, space};
use nom::{Err, ErrorKind, IResult, Needed};
use std::str;
// use crate::caseless_tag;
// use crate::caseless_tag_bytes;

#[derive(Debug, PartialEq)]
pub struct GroupByClause {
    columns: Vec<String>,
    having: String, //XXX: should this be an arbitrary expr?
}

#[derive(Debug, PartialEq)]
enum OrderType {
    OrderAscending,
    OrderDescending,
}

#[derive(Debug, PartialEq)]
pub struct OrderClause {
    order_cols: Vec<String>,
    //TODO can this be an arbitray expr?
    order_type: OrderType,
}

#[derive(Debug, PartialEq)]
pub struct LimitClause {
    limit: u64,
    offset: u64,
}

#[derive(Debug, PartialEq, Default)]
pub struct SelectStatement {
    pub table: String,
    pub distinct: bool,
    pub fields: Vec<String>,
    pub where_clause: Option<ConditionExpression>,
    pub group_by: Option<GroupByClause>,
    pub order: Option<OrderClause>,
    pub limit: Option<LimitClause>,
}

/// Parse LIMIT clause
named!(limit_clause<&[u8], LimitClause>,
    dbg_dmp!(chain!(
        space ~
        caseless_tag!("limit") ~
        space ~
        limit_val: unsigned_number ~
        space? ~
        offset_val: opt!(
            chain!(
                caseless_tag!("offset") ~
                space ~
                val: unsigned_number,
                || {val}
            )
        ),
        || {
            LimitClause {
                limit: limit_val,
                offset: match offset_val {
                    None => 0,
                    Some(v) => v,
                }
            }
        }
    ))
);

/// Parse WHERE clause of a selection
named!(where_clause<&[u8], ConditionExpression>,
    dbg_dmp!(chain!(
        space ~
        caseless_tag!("where") ~
        space ~
        cond: condition_expr,
        ||{
            cond
        }
    )
    )
);

/// Parse rule from SQL selection query.
//TODO support nested queries as selection targets
named!(pub selection<&[u8], SelectStatement>,
    chain!(
        caseless_tag!("select") ~
        space ~
        distinct: opt!(caseless_tag!("distinct")) ~
        space? ~
        fields: fieldlist ~
        delimited!(multispace, caseless_tag!("from"), multispace) ~
        table: table_reference ~
        cond: opt!(where_clause) ~
        limit: opt!(limit_clause) ~
        statement_terminator,
        || {
            println!("end of select statuement, where:{:?}", cond);
            SelectStatement {
                table: String::from(table),
                distinct: distinct.is_some(),
                fields: fields.iter().map(|s| {String::from(*s)}).collect(),
                where_clause: cond,
                group_by: None,
                order: None,
                limit: limit,
            }
        }
    )
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{ConditionBase, ConditionTree};

    #[test]
    fn simple_select() {
        let qstring = "SELECT id, name FROM users;";
        let res = selection(qstring.as_bytes());
        assert_eq!(
            res.unwrap().1,
            SelectStatement {
                fields: vec!["id".into(), "name".into()],
                table: String::from("users"),
                ..SelectStatement::default()
            }
        );
    }

    #[test]
    fn select_all() {
        let qstring = "SELECT * from users;";

        let res = selection(qstring.as_bytes());
        assert_eq!(
            res.unwrap().1,
            SelectStatement {
                fields: vec!["ALL".into()],
                table: String::from("users"),
                ..SelectStatement::default()
            }
        );
    }

    #[test]
    fn spaces_optional() {
        let qstring = "SELECT id,name FROM users;";

        let res = selection(qstring.as_bytes());
        assert_eq!(
            res.unwrap().1,
            SelectStatement {
                fields: vec!["id".into(), "name".into()],
                table: String::from("users"),
                ..SelectStatement::default()
            }
        );
    }

    #[test]
    fn case_sensitivity() {
        //XXX : this test is broken, as we force the qstring to lowercase anyway!
        let qstring = "select id, name from users;";

        let res = selection(qstring.as_bytes());
        assert_eq!(
            res.unwrap().1,
            SelectStatement {
                fields: vec!["id".into(), "name".into()],
                table: String::from("users"),
                ..SelectStatement::default()
            }
        )
    }

    #[test]
    fn termination() {
        let qstring_sem = "select id, name from users;";
        let qstring_linebreak = "select id, name from users\n";

        assert_eq!(
            selection(qstring_sem.as_bytes()),
            selection(qstring_linebreak.as_bytes())
        );
    }

    #[test]
    fn where_clause() {
        let qstring = "select * from ContactInfo where email = ?;";

        let res = selection(qstring.as_bytes());

        let expected_where_cond = Some(ConditionExpression::ComparisonOp(ConditionTree {
            operator: String::from("="),
            left: Some(Box::new(ConditionExpression::Base(ConditionBase::Field(
                String::from("email"),
            )))),
            right: Some(Box::new(ConditionExpression::Base(
                ConditionBase::Placeholder,
            ))),
        }));
        assert_eq!(
            res.unwrap().1,
            SelectStatement {
                fields: vec!["ALL".into()],
                table: String::from("ContactInfo"),
                where_clause: expected_where_cond,
                ..SelectStatement::default()
            }
        )
    }

    #[test]
    fn limit_clause() {
        let qstring1 = "select * from users limit 10;";
        let qstring2 = "select * from users limit 10 offset 10;";

        let expected_lim1 = LimitClause {
            limit: 10,
            offset: 0,
        };
        let expected_lim2 = LimitClause {
            limit: 10,
            offset: 10,
        };

        let res1 = selection(qstring1.as_bytes());
        let res2 = selection(qstring2.as_bytes());

        assert_eq!(res1.unwrap().1.limit, Some(expected_lim1));
        assert_eq!(res2.unwrap().1.limit, Some(expected_lim2));
    }

    #[test]
    fn table_alias() {
        let qstring1 = "select * from PaperTag as t;";

        let res1 = selection(qstring1.as_bytes());

        assert_eq!(
            res1.unwrap().1,
            SelectStatement {
                table: String::from("PaperTag"),
                fields: vec!["ALL".into()],
                ..Default::default()
            }
        );
    }

    #[test]
    fn distinct() {
        let qstring = "select distinct tag from PaperTag where paperId=?;";

        let res = selection(qstring.as_bytes());

        let expected_where_cond = Some(ConditionExpression::ComparisonOp(ConditionTree {
            operator: String::from("="),
            left: Some(Box::new(ConditionExpression::Base(ConditionBase::Field(
                String::from("paperId"),
            )))),
            right: Some(Box::new(ConditionExpression::Base(
                ConditionBase::Placeholder,
            ))),
        }));

        assert_eq!(
            res.unwrap().1,
            SelectStatement {
                table: String::from("PaperTag"),
                distinct: true,
                fields: vec!["tag".into()],
                where_clause: expected_where_cond,
                ..Default::default()
            }
        );
    }

    #[test]
    fn simple_condition_expr() {
        let qstring = "select infoJson from PaperStorage where paperId=? and paperStorageId=?;";

        let res = selection(qstring.as_bytes());

        let left_comp = Some(Box::new(ConditionExpression::ComparisonOp(ConditionTree {
            operator: String::from("="),
            left: Some(Box::new(ConditionExpression::Base(ConditionBase::Field(
                String::from("paperId"),
            )))),
            right: Some(Box::new(ConditionExpression::Base(
                ConditionBase::Placeholder,
            ))),
        })));

        let right_comp = Some(Box::new(ConditionExpression::ComparisonOp(ConditionTree {
            left: Some(Box::new(ConditionExpression::Base(ConditionBase::Field(
                String::from("paperStorageId"),
            )))),
            right: Some(Box::new(ConditionExpression::Base(
                ConditionBase::Placeholder,
            ))),
            operator: String::from("="),
        })));

        let expected_where_cond = Some(ConditionExpression::LogicalOp(ConditionTree {
            operator: String::from("and"),
            left: left_comp,
            right: right_comp,
        }));
        assert_eq!(
            res.unwrap().1,
            SelectStatement {
                table: String::from("PaperStorage"),
                fields: vec!["infoJson".into()],
                where_clause: expected_where_cond,
                ..Default::default()
            }
        );
    }
}
