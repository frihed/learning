use crate::parser::{ConditionTree,fieldlist};
use nom::{space,alphanumeric,eof,line_ending};
use std::str;

#[derive(Debug, PartialEq)]
pub struct GroupByClause{
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
    order_cols: Vec<String>, //TODO can this be an arbitray expr?
    order_type: OrderType,
}

#[derive(Debug, PartialEq)]
pub struct LimitClause {
    limit: i64,
    offset: i64,
}

#[derive(Debug, PartialEq, Default)]
pub struct SelectStatement{
    table: String,
    distinct: bool,
    fields: Vec<String>,
    where_clause: Option<ConditionTree>,
    group_by: Option<GroupByClause>,
    order: Option<OrderClause>,
    limit: Option<LimitClause>,
}

/// Parse WHERE clause of a selection
named!(where_clause<&[u8], ConditionTree>,
    chain!(
        tag!("where") ~
        space ~
        field: map_res!(alphanumeric, str::from_utf8) ~
        space? ~
        tag!("=") ~
        space? ~
        expr: map_res!(tag_s!(b"?"), str::from_utf8) ,
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
        tag!("select") ~
        space ~
        fields: fieldlist ~
        space ~
        tag!("from") ~
        space ~
        table: map_res!(alphanumeric, str::from_utf8) ~
        space? ~
        cond: opt!(where_clause) ~
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
                limit: None,
            }
        }
    )
);


mod tests {
    use crate::parser::ConditionTree;
    use super::*;

    #[test]
    fn simple_select() {
        let qstring = "SELECT id, name FROM users;".to_lowercase();
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
        let qstring = "SELECT * from users;".to_lowercase();

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
        let qstring = "SELECT id,name FROM users;".to_lowercase();

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
        let qstring = "select id, name from users;".to_lowercase();

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
        let qstring_sem = "select id, name from users;".to_lowercase();
        let qstring_linebreak = "select id, name from users\n".to_lowercase();

        assert_eq!(selection(qstring_sem.as_bytes()), selection(qstring_linebreak.as_bytes()));
    }

    #[test]
    fn where_clause() {
        let qstring = "select * from ContactInfo where email = ?".to_lowercase();

        let res = selection(qstring.as_bytes());
        assert_eq!(res.unwrap().1,
                   SelectStatement {
                       fields: vec!["ALL".into()],
                       table: String::from("ContactInfo").to_lowercase(),
                       where_clause: Some(ConditionTree {
                           field: String::from("email"),
                           expr: String::from("?"),
                       }),
                       ..SelectStatement::default()
                   }
        )
    }
}