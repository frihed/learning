use crate::common::{field_list, statement_terminator, table_reference, value_liest};
use nom::multispace;
use nom::{Err, ErrorKind, IResult, Needed};
use std::str;

#[derive(Debug, Default, PartialEq)]
pub struct InsertStatement {
    pub table: String,
    pub fields: Vec<(String, String)>,
}

/// Parse rule for a SQL insert query.
/// /// TODO(malte): support REPLACE, multiple parens expr, nested selection, DEFAULT VALUES
named!(pub insertion<&[u8], InsertStatement>,
    chain!(
        caseless_tag!("insert") ~
        multispace ~
        caseless_tag!("into") ~
        multispace ~
        table: table_reference ~
        multispace ~
        fields: opt!(
            chain!(
                tag!("(") ~
                fields: field_list ~
                tag!(")") ~
                multispace,
                ||{
                    fields
                }
            )
        ) ~
        caseless_tag!("values") ~
        multispace ~
        tag!("(") ~
        values: value_liest ~
        tag!(")") ~
        statement_terminator ,
        ||{
            InsertStatement {
                table: String::from(table),
                // fields: values.iter().map(|s| String::from(*s)).collect(),
                fields: match fields {
                    Some(ref f) =>
                        f.iter().map(|s| String::from(*s))
                        .zip(values.into_iter().map(|s| String::from(s))).collect(),
                    None => values.into_iter()
                        .enumerate()
                        .map(|(i, s)| (format!("{}", i),String::from(s)))
                        .collect(),
                }
            }
        }
    )
);

#[cfg(test)]
mod tests {
    use crate::insert::*;
    use crate::SqlQuery::Insert;

    #[test]
    fn simple_insert() {
        let qstring = "INSERT INTO users VALUES (42, test);";

        let res = insertion(qstring.as_bytes());

        let (r1, r2) = res.unwrap();
        println!("left: {:?}", r1);
        assert_eq!(
            r2,
            InsertStatement {
                table: String::from("users"),
                fields: vec![("0".into(), "42".into()), ("1".into(), "test".into())],
                ..Default::default()
            }
        );
    }

    #[test]
    fn placeholder_insert() {
        let qstring = "INSERT INTO users VALUES (?,?);";

        let res = insertion(qstring.as_bytes());

        assert_eq!(
            res.unwrap().1,
            InsertStatement {
                table: String::from("users"),
                fields: vec![("0".into(), "?".into()), ("1".into(), "?".into())],
                ..Default::default()
            }
        );
    }

    #[test]
    fn single_quotes_insert() {
        let qstring = "INSERT INTO users VALUES (45.6,'abc');";

        let res = insertion(qstring.as_bytes());

        assert_eq!(
            res.unwrap().1,
            InsertStatement {
                table: String::from("users"),
                fields: vec![("0".into(), "45.6".into()), ("1".into(), "abc".into())],
                ..Default::default()
            }
        );
    }

    #[test]
    fn insert_with_field_names() {
        let qstring = "insert into users (id, name) values (42, test);";

        let res = insertion(qstring.as_bytes());

        assert_eq!(
            res.unwrap().1,
            InsertStatement {
                table: String::from("users"),
                fields: vec![("id".into(), "42".into()), ("name".into(), "test".into())],
                ..Default::default()
            }
        );
    }
}
