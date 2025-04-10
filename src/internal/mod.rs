use std::num::ParseIntError;

use shunting_yard::ShuntingYard;
use thiserror::Error;

mod eval;
mod shunting_yard;

#[cfg_attr(test, derive(PartialEq))]
#[derive(Error, Debug)]
pub(crate) enum Error {
    #[error("Expression has invalid syntax")]
    InvalidSyntax,
    #[error("Invalid expression {0}")]
    InvalidExpression(String),
    #[error("Caller should have passed a digit")]
    NumberParse(ParseIntError),
    #[error("Invalid RPN {0} for expression")]
    InvalidRpn(String),
}

pub(super) fn estimate_expression(expr: &str) -> Result<i64, Error> {
    let mut shunting_yard = ShuntingYard::new(expr)?;
    shunting_yard.to_rpn().compute()
}

#[cfg(test)]
mod test {
    use super::estimate_expression;

    const EASY_EXPR: &str = "4+2";
    const EASY_RESULT: i64 = 6;
    const MEDIUM_EXPR: &str = "3    * 6 - 7  + 2";
    const MEDIUM_RESULT: i64 = 13;
    const HARD_EXPR: &str = "(3+4) +  7 *2 -1-9";
    const HARD_RESULT: i64 = 11;
    const HARDER_EXPR: &str = "(8 -1 +3)  *6 -((3+7)*2  )";
    const HARDER_RESULT: i64 = 40;

    #[test]
    fn test_easy_computation() {
        match estimate_expression(EASY_EXPR) {
            Ok(res) => {
                println!("Result of expression {} is {res}", EASY_EXPR.trim());
                assert_eq!(res, EASY_RESULT)
            }
            Err(e) => panic!("Expected result {EASY_RESULT}, received error {e}"),
        }
    }

    #[test]
    fn test_medium_computation() {
        match estimate_expression(MEDIUM_EXPR) {
            Ok(res) => {
                println!("Result of expression {} is {res}", MEDIUM_EXPR.trim());
                assert_eq!(res, MEDIUM_RESULT)
            }
            Err(e) => panic!("Expected result {MEDIUM_RESULT}, received error {e}"),
        }
    }

    // TODO -> This case fails since RPN generated is:
    // 3 4 + 7 2 * 1 9 - - +
    // Meanwhile the correct one should be:
    // 3 4 + 7 2 * 1 - 9 - +
    // Using lower or equal in Token::Operator branch seems to solve the problem. Investigate...
    #[test]
    fn test_hard_computation() {
        match estimate_expression(HARD_EXPR) {
            Ok(res) => {
                println!("Result of expression {} is {res}", HARD_EXPR.trim());
                assert_eq!(res, HARD_RESULT)
            }
            Err(e) => panic!("Expected result {HARD_RESULT}, received error {e}"),
        }
    }

    #[test]
    fn test_harder_computation() {
        match estimate_expression(HARDER_EXPR) {
            Ok(res) => {
                println!("Result of expression {} is {res}", HARDER_EXPR.trim());
                assert_eq!(res, HARDER_RESULT)
            }
            Err(e) => panic!("Expected result {HARDER_RESULT}, received error {e}"),
        }
    }
}
