use nom::{IResult, alphanumeric, line_ending, space};
use std::str;
use std::error::Error;

#[derive(Debug, PartialEq)]
pub struct SqlQuery {
    fields: Vec<String>,
    table: String,
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

named!(selection<&[u8], SqlQuery>,
    chain!(
        tag!("select") ~
        space ~
        fields: csvlist ~  //TODO cover * case
        space ~
        tag!("from") ~
        space ~
        table: map_res!(alphanumeric, str::from_utf8) ~
        alt!(tag!(";") | line_ending),
        || {
            SqlQuery {
                table: String::from(table),
                fields: fields.iter().map(|s| {String::from(*s)}).collect()
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
                   }
        );
    }

    #[test]
    fn select_all() {
        let qstring = "SELECT * from users;";

        assert_eq!(parse_query(qstring).unwrap(),
                   SqlQuery { fields: vec!["ALL".into()], table: String::from("users") });
    }

    #[test]
    fn spaces_optional(){
        let qstring = "SELECT id,name FROM users;";

        assert_eq!(parse_query(qstring).unwrap(),
            SqlQuery{
                fields:vec!["id".into(), "name".into()],
                table: String::from("users"),
            });
    }

    #[test]
    fn case_sensitivity(){
        let qstring = "select id, name from users;";

        assert_eq!(parse_query(qstring).unwrap(),
            SqlQuery{ fields: vec!["id".into(), "name".into()],
                table:String::from("users"),
            })
    }

    #[test]
    fn termination(){
        let qstring_sem = "select id, name from users;";
        let qstring_linebreak = "select id, name from users\n";

        assert_eq!(parse_query(qstring_sem), parse_query(qstring_linebreak));
    }

}