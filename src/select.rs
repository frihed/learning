use crate::parser::{ConditionTree, fieldlist, unsigned_number};
use nom::{space, alphanumeric, eof, line_ending};
use nom::{IResult,Err,ErrorKind, Needed};
use std::str;
use crate::caseless_tag;
use crate::caseless_tag_bytes;

#[derive(Debug, PartialEq)]
pub struct GroupByClause {
    columns: Vec<String>,
    having: String,//XXX: should this be an arbitrary expr?
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
    table: String,
    distinct: bool,
    fields: Vec<String>,
    where_clause: Option<ConditionTree>,
    group_by: Option<GroupByClause>,
    order: Option<OrderClause>,
    limit: Option<LimitClause>,
}

/// Parse LIMIT clause
named!(limit_clause<&[u8],LimitClause>,
    chain!(
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
    )
);

/// Parse WHERE clause of a selection
named!(where_clause<&[u8], ConditionTree>,
    chain!(
        caseless_tag!("where") ~
        space ~
        field: map_res!(alphanumeric, str::from_utf8) ~
        space? ~
        tag!("=") ~
        space? ~
        expr: map_res!(tag_s!(b"?"), str::from_utf8) , //TODO : broken!
        ||{
            ConditionTree{
             field: String::from(field),
             expr:String::from(expr),
            }
        }
    )
);
//TODO support nested queries as selection targets
named!(pub selection<&[u8], SelectStatement>,
    chain!(
        caseless_tag!("select") ~
        space ~
        fields: fieldlist ~
        space ~
        caseless_tag!("from") ~
        space ~
        table: map_res!(alphanumeric, str::from_utf8) ~
        space? ~
        cond: opt!(where_clause) ~
        space? ~
        limit: opt!(limit_clause) ~
        space? ~
        alt!(eof | tag!(";") | line_ending),
        || {
            SelectStatement {
                table: String::from(table),
                distinct: false,
                fields: fields.iter().map(|s| {String::from(*s)}).collect(),
                where_clause: cond,
                group_by: None,
                order: None,
                limit: limit,
            }
        }
    )
);


mod tests {
    use crate::parser::ConditionTree;
    use super::*;

    #[test]
    fn simple_select() {
        let qstring = "SELECT id, name FROM users;";
        let res = selection(qstring.as_bytes());
        assert_eq!(res.unwrap().1,
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
        assert_eq!(res.unwrap().1,
                   SelectStatement {
                       fields: vec!["ALL".into()],
                       table: String::from("users"),
                       ..SelectStatement::default()
                   });
    }

    #[test]
    fn spaces_optional() {
        let qstring = "SELECT id,name FROM users;";

        let res = selection(qstring.as_bytes());
        assert_eq!(res.unwrap().1,
                   SelectStatement {
                       fields: vec!["id".into(), "name".into()],
                       table: String::from("users"),
                       ..SelectStatement::default()
                   });
    }

    #[test]
    fn case_sensitivity() {
        //XXX : this test is broken, as we force the qstring to lowercase anyway!
        let qstring = "select id, name from users;";

        let res = selection(qstring.as_bytes());
        assert_eq!(res.unwrap().1,
                   SelectStatement {
                       fields: vec!["id".into(), "name".into()],
                       table: String::from("users"),
                       ..SelectStatement::default()
                   })
    }

    #[test]
    fn termination() {
        let qstring_sem = "select id, name from users;";
        let qstring_linebreak = "select id, name from users\n";

        assert_eq!(selection(qstring_sem.as_bytes()), selection(qstring_linebreak.as_bytes()));
    }

    #[test]
    fn where_clause() {
        let qstring = "select * from ContactInfo where email = ?;";

        let res = selection(qstring.as_bytes());
        assert_eq!(res.unwrap().1,
                   SelectStatement {
                       fields: vec!["ALL".into()],
                       table: String::from("ContactInfo"),
                       where_clause: Some(ConditionTree {
                           field: String::from("email"),
                           expr: String::from("?"),
                       }),
                       ..SelectStatement::default()
                   }
        )
    }

    #[test]
    fn limit_clause(){
        let qstring1 = "select * from users limit 10\n";
        let qstring2 = "select * from users limit 10 offset 10\n";

        let expected_lim1 = LimitClause {
            limit : 10,
            offset: 0,
        };
        let expected_lim2 = LimitClause {
            limit : 10 ,
            offset : 10,
        };

        let res1 = selection(qstring1.as_bytes());
        let res2= selection(qstring2.as_bytes());

        assert_eq!(res1.unwrap().1.limit, Some(expected_lim1));
        assert_eq!(res2.unwrap().1.limit, Some(expected_lim2));
    }
}