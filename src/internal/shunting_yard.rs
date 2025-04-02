use log::debug;
use std::num::ParseIntError;
use thiserror::Error;

const TOKEN_FIXABLE_SEQUENCE: &str = "-(";
const TOKEN_FIX_SEQUENCE: &str = "-1*(";
const OPERATORS: [char; 6] = ['+', '-', '*', '/', '(', ')'];
const INVALID_TOKEN_PATTERNS: [&str; 15] = [
    "-*", "-/", "-)", "+*", "+/", "+)", "**", "*/", "*)", "/*", "//", "/)", "(*", "(/", ")(",
];

#[cfg_attr(test, derive(PartialEq))]
#[derive(Error, Debug)]
pub(crate) enum Error {
    #[error("Invalid RPN {0} for expression")]
    InvalidRpn(String),
    #[error("Unknown operator {0}")]
    UnknownOperator(String),
    #[error(transparent)]
    ParseInt(#[from] ParseIntError),
    #[error("Invalid expression {0}")]
    InvalidExpression(String),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
enum Operator {
    LeftBracket,
    RightBracket,
    Prod,
    Div,
    Sub,
    Add,
}

impl Operator {
    fn execute(self, v1: i64, v2: i64) -> i64 {
        match self {
            Operator::LeftBracket | Operator::RightBracket => {
                unreachable!("Hit brackets in operation execution")
            }
            Operator::Prod => v1 * v2,
            Operator::Div => v1 / v2,
            Operator::Add => v1 + v2,
            Operator::Sub => v1 - v2,
        }
    }
}

impl TryFrom<char> for Operator {
    type Error = crate::internal::shunting_yard::Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '*' => Ok(Self::Prod),
            '/' => Ok(Self::Div),
            '+' => Ok(Self::Add),
            '-' => Ok(Self::Sub),
            '(' => Ok(Self::LeftBracket),
            ')' => Ok(Self::RightBracket),
            _ => Err(Error::UnknownOperator(value.to_string())),
        }
    }
}

impl TryFrom<&str> for Operator {
    type Error = crate::internal::shunting_yard::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "*" => Ok(Self::Prod),
            "/" => Ok(Self::Div),
            "+" => Ok(Self::Add),
            "-" => Ok(Self::Sub),
            "(" => Ok(Self::LeftBracket),
            ")" => Ok(Self::RightBracket),
            _ => Err(Error::UnknownOperator(value.to_string())),
        }
    }
}

impl From<&Operator> for String {
    fn from(value: &Operator) -> Self {
        match value {
            Operator::LeftBracket | Operator::RightBracket => {
                unreachable!("Hit brackets in RPN stringify")
            }
            Operator::Prod => "*".to_owned(),
            Operator::Div => "/".to_owned(),
            Operator::Add => "+".to_owned(),
            Operator::Sub => "-".to_owned(),
        }
    }
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug)]
enum Token {
    Number(i64),
    Operator(Operator),
}

impl From<&Token> for String {
    fn from(value: &Token) -> Self {
        match value {
            Token::Number(n) => (*n).to_string(),
            Token::Operator(operator) => operator.into(),
        }
    }
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug)]
pub(super) struct ShuntingYard {
    operator_stack: Vec<Operator>,
    output_queue: Vec<Token>,
    tokens: Vec<Token>,
}

impl ShuntingYard {
    pub(super) fn new(expr: &str) -> Result<Self, Error> {
        Ok(Self {
            operator_stack: vec![],
            output_queue: vec![],
            tokens: parse_expression_to_token_list(expr)?,
        })
    }

    ///
    /// This function convert a list of token to Reverse Polish Notation
    /// following this logic
    ///
    /// While there are tokens to be read:
    ///      Read a token
    ///      If it's a number add it to queue
    ///      If it's an operator
    ///             While there's an operator on the top of the stack with greater precedence that is not a left bracket:
    ///                     Pop operators from the stack onto the output queue
    ///             Push the current operator onto the stack
    ///      If it's a left bracket push it onto the stack
    ///      If it's a right bracket
    ///           While there's not a left bracket at the top of the stack:
    ///                    Pop operators from the stack onto the output queue.
    ///            Pop the left bracket from the stack and discard it
    /// While there are operators on the stack, pop them to the queue
    ///
    pub(super) fn to_rpn(&mut self) -> &mut Self {
        debug!("Estimating RPN from tokens list {:?}", self.tokens);

        let mut token_iterator = self.tokens.iter();
        while let Some(token) = token_iterator.next() {
            match token {
                Token::Number(n) => {
                    debug!("Pushing numeric value {n} onto output queue");
                    self.output_queue.insert(0, Token::Number(*n));
                }
                Token::Operator(operator) if operator == &Operator::LeftBracket => {
                    debug!("Pushing Left Bracket onto stack");
                    self.operator_stack.insert(0, *operator)
                }
                Token::Operator(operator) if operator == &Operator::RightBracket => {
                    loop {
                        let stack_top = self.operator_stack.first();
                        if stack_top.is_some_and(|st| st != &Operator::LeftBracket) {
                            let op = self.operator_stack.remove(0);
                            debug!("Popping operator {op:?} from stack onto output queue");
                            self.output_queue.insert(0, Token::Operator(op));
                        } else {
                            break;
                        }
                    }

                    self.operator_stack.remove(0);
                }
                Token::Operator(operator) => {
                    loop {
                        let stack_top = self.operator_stack.first();
                        if stack_top
                            .is_some_and(|st| st != &Operator::LeftBracket && st.le(operator))
                        {
                            let op = self.operator_stack.remove(0);
                            debug!(
                                "Popping operator {op:?} with greater precedence wrt operator {operator:?} from stack onto the otuput queue"
                            );
                            self.output_queue.insert(0, Token::Operator(op));
                        } else {
                            break;
                        }
                    }

                    self.operator_stack.insert(0, *operator);
                }
            };
        }

        let mut operator_stack_iterator = self.operator_stack.iter();
        while let Some(operator) = operator_stack_iterator.next() {
            self.output_queue.insert(0, Token::Operator(*operator));
        }

        debug!(
            "Expression in Reverse Polish Notation is: {:?}",
            self.output_queue
        );

        self
    }

    pub(super) fn compute(&mut self) -> Result<i64, Error> {
        let mut stack = vec![];
        let rpn_str = self
            .output_queue
            .iter()
            .map(|c| c.into())
            .collect::<Vec<String>>()
            .join(", ");
        while let Some(token) = self.output_queue.pop() {
            match token {
                Token::Number(n) => stack.push(n),
                Token::Operator(operator) => {
                    let v2_opt = stack.pop();
                    let v1_opt = stack.pop();

                    if let (Some(v1), Some(v2)) = (v1_opt, v2_opt) {
                        stack.push(operator.execute(v1, v2));
                    } else {
                        break;
                    }
                }
            }
        }

        match stack.first() {
            Some(v) => Ok(*v),
            None => Err(Error::InvalidRpn(rpn_str)),
        }
    }
}

///
/// This function take input data from stdin and convert them to a token list
/// failing if something cannot be handled properly
/// If a '-' token is followed immediately by a '(' token then it will be
/// converted to '-1 *'sequence
/// It analize input string character by character applying the following rules:
///
/// If c is an operator -> Operator::try_from()
/// If c is_numeric -> c.parse::<i64>()
/// If we have a sequence of operators followed by a digit [i.e. +-4 | *-4 | /-4 | --4]
/// It should be interpreted as:
///     Token::Operator(Operator::Add),
///     Token::Number(-4)
/// If we have a sequence of operators they should result into a respective token list
/// If we have empty spaces, just ignore them
///
fn parse_expression_to_token_list(expr: &str) -> Result<Vec<Token>, Error> {
    let mut expression = expr.to_owned();

    //Remove all whitespaces
    expression.retain(|c| !c.is_whitespace());

    if expression.contains(TOKEN_FIXABLE_SEQUENCE) {
        expression = expression.replace(TOKEN_FIXABLE_SEQUENCE, TOKEN_FIX_SEQUENCE);
    }

    if !validate_token_list(&expression) {
        return Err(Error::InvalidExpression(expr.to_owned()));
    }

    let mut last = "";
    let mut second_last = "";
    let mut third_to_last = "";
    let mut indexes_to_remove = vec![];
    let mut token_str_list = expression
        .split_inclusive(|c| OPERATORS.contains(&c)) // Split inclusively input string on operators
        .map(|v| {
            if v.chars()
                .last()
                .and_then(|c| Some(OPERATORS.contains(&c)))
                .is_some()
            {
                let split_inclusive = v.split_at(v.len() - 1);
                vec![split_inclusive.0, split_inclusive.1]
            } else {
                vec![v]
            }
        }) // Split inclusive token to single token instance
        .flatten() // Flatten from Vec<Vec<&str>> to Vec<&str>
        .filter(|&p| !p.is_empty()) // Filter empty strings
        .enumerate()
        .map(|(i, v)| {
            // If there is a sequence of two operators follwed by a digit, second operator (+ | -) represents digit sign. If different operator return error
            third_to_last = second_last;
            second_last = last;
            last = v;
            if Operator::try_from(third_to_last).is_ok_and(|ttl_op| {
                (ttl_op == Operator::Add
                    || ttl_op == Operator::Sub
                    || ttl_op == Operator::LeftBracket
                    || ttl_op == Operator::Div
                    || ttl_op == Operator::Prod)
                    && Operator::try_from(second_last).is_ok_and(|sl_op| {
                        (sl_op == Operator::Add || sl_op == Operator::Sub)
                            && last.parse::<i64>().is_ok()
                    })
            }) {
                indexes_to_remove.push(i - 1);
                let mut signed_value = second_last.to_string();
                signed_value.push_str(v);
                signed_value
            } else {
                v.to_string()
            }
        })
        .collect::<Vec<_>>();

    indexes_to_remove.reverse();
    indexes_to_remove.into_iter().for_each(|i| {
        token_str_list.remove(i);
    });

    token_str_list
        .into_iter()
        .map(|s| -> Result<Token, Error> {
            if OPERATORS
                .into_iter()
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .contains(&s)
            {
                Ok(Token::Operator(Operator::try_from(s.as_str())?))
            } else {
                Ok(Token::Number(s.parse::<i64>().map_err(Error::ParseInt)?))
            }
        })
        .collect()
}

///
/// Check if expression contains invalid patterns:
/// Valid patterns are:
/// * third_to_last = + && second_last = + && last = NUMBER
/// * third_to_last = + && second_last = - && last = NUMBER
/// * third_to_last = - && second_last = + && last = NUMBER
/// * third_to_last = - && second_last = - && last = NUMBER
/// * third_to_last = ( && second_last = + && last = NUMBER
/// * third_to_last = ( && second_last = - && last = NUMBER
/// * third_to_last = * && second_last = + && last = NUMBER
/// * third_to_last = * && second_last = - && last = NUMBER
/// * third_to_last = / && second_last = + && last = NUMBER
/// * third_to_last = / && second_last = - && last = NUMBER
/// Every other pattern is considered not valid
/// Moreover it is mandatory to check that the number of open parenthesis match the number of closed paranthesis
///
fn validate_token_list(expr: &str) -> bool {
    let number_of_open_p = expr.chars().filter(|c| *c == '(').count();
    let number_of_closed_p = expr.chars().filter(|c| *c == ')').count();
    if number_of_open_p != number_of_closed_p {
        debug!(
            "Different number of right brackets {number_of_open_p} and left brackets {number_of_closed_p}"
        );
        return false;
    }
    !INVALID_TOKEN_PATTERNS.into_iter().any(|invalid_token| {
        if expr.contains(invalid_token) {
            debug!("Hit invalid sequence {invalid_token}");
        }
        expr.contains(invalid_token)
    })
}

#[cfg(test)]
mod test {
    use crate::internal::shunting_yard::parse_expression_to_token_list;

    use super::{Error, Operator, ShuntingYard, Token};

    #[test]
    fn test_shunting_yard_data_struct_from_expression_signed_negative() {
        let expression = "4 + 18/(9--3)";

        assert_eq!(
            parse_expression_to_token_list(expression).unwrap(),
            vec![
                Token::Number(4),                        // 4
                Token::Operator(Operator::Add),          // +
                Token::Number(18),                       // 18
                Token::Operator(Operator::Div),          // /
                Token::Operator(Operator::LeftBracket),  // (
                Token::Number(9),                        // 9
                Token::Operator(Operator::Sub),          // -
                Token::Number(-3),                       // -3
                Token::Operator(Operator::RightBracket), // )
            ]
        );
    }

    #[test]
    fn test_shunting_yard_data_struct_from_expression_signed_positive() {
        let expression = "4 + 18/(9-+3)";

        assert_eq!(
            parse_expression_to_token_list(expression).unwrap(),
            vec![
                Token::Number(4),                        // 4
                Token::Operator(Operator::Add),          // +
                Token::Number(18),                       // 18
                Token::Operator(Operator::Div),          // /
                Token::Operator(Operator::LeftBracket),  // (
                Token::Number(9),                        // 9
                Token::Operator(Operator::Sub),          // -
                Token::Number(3),                        // 3
                Token::Operator(Operator::RightBracket), // )
            ]
        );
    }

    #[test]
    fn test_shunting_yard_data_struct_from_expression_invalid_prod() {
        let expression = "4 + 18/(9-*3)";

        match parse_expression_to_token_list(expression) {
            Ok(token_list) => panic!("Invalid expression has been parsed to {token_list:#?}"),
            Err(e) => {
                println!("Failed to pasrse expression due to error {e}");
                assert_eq!(e, Error::InvalidExpression(expression.to_string()))
            }
        };
    }

    #[test]
    fn test_shunting_yard_data_struct_from_expression_invalid_div() {
        let expression = "4 + 18/(9-/3)";

        match parse_expression_to_token_list(expression) {
            Ok(token_list) => panic!("Invalid expression has been parsed to {token_list:#?}"),
            Err(e) => {
                println!("Failed to pasrse expression due to error {e}");
                assert_eq!(e, Error::InvalidExpression(expression.to_string()))
            }
        };
    }

    #[test]
    fn test_rpn() {
        let mut shunting_yard = ShuntingYard {
            operator_stack: vec![],
            output_queue: vec![],
            tokens: vec![
                Token::Number(4),                        // 4
                Token::Operator(Operator::Add),          // +
                Token::Number(18),                       // 18
                Token::Operator(Operator::Div),          // /
                Token::Operator(Operator::LeftBracket),  // (
                Token::Number(9),                        // 9
                Token::Operator(Operator::Sub),          // -
                Token::Number(3),                        // 3
                Token::Operator(Operator::RightBracket), // )
            ],
        };

        assert_eq!(
            shunting_yard.to_rpn().output_queue,
            vec![
                Token::Operator(Operator::Add),
                Token::Operator(Operator::Div),
                Token::Operator(Operator::Sub),
                Token::Number(3),
                Token::Number(9),
                Token::Number(18),
                Token::Number(4),
            ]
        );
    }

    #[test]
    fn test_result_computation() {
        let mut shunting_yard = ShuntingYard {
            operator_stack: vec![],
            output_queue: vec![],
            tokens: vec![
                Token::Number(4),                        // 4
                Token::Operator(Operator::Add),          // +
                Token::Number(18),                       // 18
                Token::Operator(Operator::Div),          // /
                Token::Operator(Operator::LeftBracket),  // (
                Token::Number(9),                        // 9
                Token::Operator(Operator::Sub),          // -
                Token::Number(3),                        // 3
                Token::Operator(Operator::RightBracket), // )
            ],
        };

        assert_eq!(shunting_yard.to_rpn().compute().unwrap(), 7);
    }
}
