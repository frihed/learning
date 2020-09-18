use nom::{alphanumeric, digit, is_alphanumeric, line_ending, multispace, space};
use nom::{eof, is_alphabetic};
use nom::{Err, ErrorKind, IResult, Needed};
use std::str;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub enum Operator {
    Not,
    And,
    Or,
    Like,
    NotLike,
    Equal,
    NotEqual,
    Greater,
    GreaterOrEqual,
    Less,
    LessOrEqual,
}

#[inline]
pub fn is_sql_identifier(chr: u8) -> bool {
    // XXX(malte): dot should not be in here once we have proper alias handling
    is_alphanumeric(chr) || chr == '_' as u8 || chr == '.' as u8
}

#[inline]
pub fn is_fp_number(chr: u8) -> bool {
    is_alphanumeric(chr) || chr == '.' as u8
}

/// Parses a SQL identifier (alphanumeric and ".").
named!(pub fp_number<&[u8], &[u8]>,
// f64::from_str(&format!("{}.{}", integral, fractional)).unwrap()
//      integral: map_res!(take_while1!(is_digit), str::from_utf8) ~
//      fractional: map_res!(take_while!(is_digit), str::from_utf8),
    take_while1!(is_fp_number)
);

/// Parse a SQL identifier (alphanumeric and _ and .)
named!(pub sql_identifier<&[u8], &[u8]>,
    take_while1!(is_sql_identifier)
);

/// Parse an unsigned integer.
named!(pub unsigned_number<&[u8], u64>,
    map_res!(
        map_res!(digit, str::from_utf8),
        FromStr::from_str
    )
);
/// Parse a terminator that ends a SQL statement.
named!(pub statement_terminator,
    delimited!(
        opt!(multispace),
        alt_complete!(tag!(";") | line_ending | eof),
        opt!(multispace)
    )
);

/// Parse binary comparison operators
named!(pub binary_comparison_operator<&[u8], Operator>,
    dbg_dmp!(alt_complete!(
           map!(caseless_tag!("like"), |_| Operator::Like)
         | map!(caseless_tag!("not_like"), |_| Operator::NotLike)
         | map!(tag!("<>"), |_| Operator::NotEqual)
         | map!(tag!(">="), |_| Operator::GreaterOrEqual)
         | map!(tag!("<="), |_| Operator::LessOrEqual)
         | map!(tag!("="), |_| Operator::Equal)
         | map!(tag!("<"), |_| Operator::Less)
         | map!(tag!(">"), |_| Operator::Greater)
        ))
);

/// Parse logical operators
named!(pub binary_logical_operator<&[u8], Operator>,
    alt_complete!(
        map!(caseless_tag!("and") , |_| Operator::And)
        | map!(caseless_tag!("or"), |_| Operator::Or)
    )
);

/// Parse unary comparison operators
named!(pub unary_comparison_operator<&[u8], &str>,
    map_res!(
        alt_complete!(
            tag_s!(b"ISNULL")
            | tag_s!(b"NOT")
            | tag_s!(b"-") //??? number neg
        ),
        str::from_utf8
    )
);

/// Parse unary negation operators
named!(pub unary_negation_operator<&[u8], Operator>,
    alt_complete!(
        map!(caseless_tag!("not"), |_| Operator::Not)
        | map!(tag!("!"), |_| Operator::Not)
    )
);

/// Parse rule for a comma-separated list.
named!(csvlist<&[u8], Vec<&str>>,
many0!(dbg_dmp!(
    map_res!(
        chain!(
        fieldname: sql_identifier ~
        opt!(
            chain!(
                multispace? ~
                tag!(",") ~
                multispace?,
                || {}
            )
        ),
        || {fieldname}
        ),
        str::from_utf8)
))
);

/// Parse list of columns/fields.
named!(pub field_list<&[u8], Vec<&str>>,
    alt_complete!(
        tag!("*") => {|_| vec!["ALL".into()]}
        | csvlist
    )
);

// pub fn fieldlist(inp: &[u8]) -> IResult<&[u8], Vec<&str>> {
//     alt_complete!(inp,
//         tag!("*") => {|_| vec!["ALL".into()]}
//         | csvlist
//     )
// }

/// Parse list of table names.
named!(pub table_list<&[u8], Vec<&str>>,
    many0!(
        chain!(
            name: table_reference ~
            opt!(
                chain!(
                    multispace? ~
                    tag!(",") ~
                    multispace? ~
                    || {}
                )
            ),
            ||{
                name
            }
        )
    )
);

named!(pub value_liest<&[u8], Vec<&str>>,
    many0!(
        map_res!(chain!(
            val: alt_complete!(tag_s!(b"?")
            | tag_s!(b"CURRENT_TIMESTAMP")
            | delimited!(tag!("'"), alphanumeric, tag!("'")) //this is 'XXX'
            | fp_number) ~                      // this is alphanumeric or .
            opt!(
                chain!(
                    multispace? ~
                    tag!(",") ~
                    multispace? ,
                    ||{}
                )
            ),
            ||{
                val
            }
        ),
        str::from_utf8
        )
    )
);

/// Parse a reference to a named table
named!(pub table_reference<&[u8], &str>,
    chain!(
        table: map_res!( sql_identifier, str::from_utf8) ~
        alias: opt!(
            chain!(
                space ~
                caseless_tag!("as") ~
                space ~
                alias: map_res!(sql_identifier, str::from_utf8),
                ||{
                    println!("got alias: {} -> {}", table, alias);
                    alias
                }
            )
        ),
        || {
            table
        }
    )
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sql_identifiers() {
        let id1 = b"foo";
        let id2 = b"f_o_o";
        let id3 = b"foo12";
        let id4 = b":fo oo";

        assert!(sql_identifier(id1).is_done());
        assert!(sql_identifier(id2).is_done());
        assert!(sql_identifier(id3).is_done());
        assert!(sql_identifier(id4).is_err());
    }
}
