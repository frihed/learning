use crate::parser::{binary_comparison_operator, binary_logical_operator, unary_negation_operator};
use crate::parser::{ConditionBase, ConditionExpression, ConditionTree};
use nom::{alphanumeric, multispace};
use std::str;

/// 处理表达式的合并， 返回合并所得的 ConditionExpression
/// 参数: initial 第一个表达式 remainder: 之后的 运算符和 表达式
fn fold_cond_exprs(
    initial: ConditionExpression,
    remainder: Vec<(&str, ConditionExpression)>,
) -> ConditionExpression {
    remainder.into_iter().fold(initial, |acc, pair| {
        let (oper, expr) = pair;
        match oper {
            "=" | ">" | "<" | ">=" | "<=" => ConditionExpression::ComparisonOp(ConditionTree {
                operator: String::from(oper.clone()),
                left: Some(Box::new(acc)),
                right: Some(Box::new(expr)),
            }),
            "and" | "or" => ConditionExpression::LogicalOp(ConditionTree {
                operator: String::from(oper.clone()),
                left: Some(Box::new(acc)),
                right: Some(Box::new(expr)),
            }),
            o => {
                println!("unsupported op {}", o);
                unimplemented!()
            }
        }
    })
}

/// Parse a conditional expression into a condition tree structure
named!(pub condition_expr<&[u8], ConditionExpression>,
    dbg_dmp!(
        chain!(
        neg_opt: opt!(unary_negation_operator) ~
        initial : boolean_primary ~
        remainder: many0!(
            complete!(chain!(
                log_op: delimited!(opt!(multispace), binary_logical_operator, opt!(multispace)) ~
                right_expr: condition_expr,
                ||{
                    println!("logical comparison");
                    (log_op, right_expr)
                }
            )
            )
        ),
        || {
            if let Some(ref no) = neg_opt {
                ConditionExpression::LogicalOp(
                    ConditionTree {
                        operator: String::from(neg_opt.unwrap()),
                        left: Some(Box::new(fold_cond_exprs(initial, remainder))),
                        right: None,
                    }
                )
            }else {
                println!("remainder: {:?}", remainder);
                fold_cond_exprs(initial, remainder)
            }
        }
        )
    )
);

// Parse binary comparison
named!(boolean_primary<&[u8], ConditionExpression>,
    dbg_dmp!(
        chain!(
            initial: predicate ~
            remainder: many0!(
                complete!(
                chain!(
                    op: delimited!(opt!(multispace), binary_comparison_operator, opt!(multispace)) ~
                    right_expr: boolean_primary,
                    || {
                        (op, right_expr)
                    }
                )
                )

            ),
            || {
                println!("binary comparison");
                fold_cond_exprs(initial, remainder)
            }
        )
    )
);

// Parse ? or field
named!(predicate<&[u8], ConditionExpression>,
    alt_complete!(
        chain!(
            delimited!(opt!(multispace), tag!("?"), opt!(multispace)),
            ||{
                println!("pred: placeholder");
                ConditionExpression::Expr(
                    ConditionBase::Placeholder
                )
            }
        )
        |
        chain!(
            field: delimited!(opt!(multispace), alphanumeric, opt!(multispace)),
            ||{
                println!("pred: field {:?}", str::from_utf8(field).unwrap());
                ConditionExpression::Expr(
                        ConditionBase::Field(String::from(str::from_utf8(field).unwrap()))
                )
            }
        )
    )
);
