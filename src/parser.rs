use nom::{IResult, alphanumeric, eof, line_ending, space};
use std::str;
use std::error::Error;

#[derive(Debug, PartialEq)]
pub struct SqlQuery {
    fields: Vec<String>,
    table: String,
    cond: Option<ConditionTree>,
}

#[derive(Debug, PartialEq)]
pub struct ConditionTree {
    field: String,
    expr: String,
}

named!(csvlist<&[u8], Vec<&str>>,
    many0!(
        map_res!(
            chain!(
            fieldname: alphanumeric ~
            opt!(
                chain!(
                    tag!(",") ~
                    opt!(space),
                    || {}
                )
            ),
            || {fieldname}
            ),
            str::from_utf8)
    )
    );

named!(fieldlist<&[u8], Vec<&str>>,
    alt!(
        tag!("*") => {|_| vec!["ALL".into()]}
        | csvlist
    )
);

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

named!(selection<&[u8], SqlQuery>,
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
            SqlQuery {
                table: String::from(table),
                fields: fields.iter().map(|s| {String::from(*s)}).collect(),
                cond: cond,
            }
        }
    )
);

pub fn parse_query(input: &str) -> Result<SqlQuery, &str> {
    let q_lower = input.to_lowercase();

    //TODO(malte)
    match selection(&q_lower.into_bytes()) {
        IResult::Done(_, o) => Ok(o),
        IResult::Error(_) => Err("parse error"),
        IResult::Incomplete
        (_) => Err("incomplete query"),
    }
}


mod tests {
    use nom::IResult;
    use super::*;

    #[test]
    fn simple_select() {
        let qstring = "SELECT id, name FROM users;";
        assert_eq!(parse_query(qstring).unwrap(),
                   SqlQuery {
                       fields: vec!["id".into(), "name".into()],
                       table: String::from("users"),
                       cond: None,
                   }
        );
    }

    #[test]
    fn select_all() {
        let qstring = "SELECT * from users;";

        assert_eq!(parse_query(qstring).unwrap(),
                   SqlQuery {
                       fields: vec!["ALL".into()],
                       table: String::from("users"),
                       cond: None,
                   });
    }

    #[test]
    fn spaces_optional() {
        let qstring = "SELECT id,name FROM users;";

        assert_eq!(parse_query(qstring).unwrap(),
                   SqlQuery {
                       fields: vec!["id".into(), "name".into()],
                       table: String::from("users"),
                       cond: None,
                   });
    }

    #[test]
    fn case_sensitivity() {
        let qstring = "select id, name from users;";

        assert_eq!(parse_query(qstring).unwrap(),
                   SqlQuery {
                       fields: vec!["id".into(), "name".into()],
                       table: String::from("users"),
                       cond: None,
                   })
    }

    #[test]
    fn termination() {
        let qstring_sem = "select id, name from users;";
        let qstring_linebreak = "select id, name from users\n";

        assert_eq!(parse_query(qstring_sem), parse_query(qstring_linebreak));
    }

    #[test]
    fn where_clause() {
        let qstring = "select * from ContactInfo where email = ?";

        assert_eq!(parse_query(qstring).unwrap(),
                   SqlQuery {
                       fields: vec!["ALL".into()],
                       table: String::from("ContactInfo").to_lowercase(),
                       cond: Some(ConditionTree {
                           field: String::from("email"),
                           expr: String::from("?"),
                       }),
                   }
        )
    }
}