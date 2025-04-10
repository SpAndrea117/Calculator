use std::iter::Peekable;

use super::Error;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub(super) enum Operator {
    LeftBracket,
    RightBracket,
    Prod,
    Div,
    Sub,
    Add,
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

#[derive(Debug, PartialEq)]
pub(super) enum Token {
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

impl Operator {
    pub(super) fn execute(self, v1: i64, v2: i64) -> i64 {
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

///
/// Parse mathematic expression to Token list
///
pub(super) fn parse_expr(s: &str) -> Result<Vec<Token>, Error> {
    let mut result = Vec::new();
    let mut it = s.chars().peekable();
    // If we have multiple consecutive signs pop last value and replace it following this logic:
    // - * - = +
    // + * + = +
    // - * + = -
    // + * - = -
    let mut last_token = &None::<Token>;
    while let Some(&c) = it.peek() {
        match c {
            '0'..='9' => {
                it.next();
                let n = get_number(c, &mut it)?;
                result.push(Token::Number(n));
                last_token = &None;
            }
            '(' => {
                result.push(Token::Operator(Operator::LeftBracket));
                last_token = &Some(Token::Operator(Operator::LeftBracket));
                it.next();
            }
            ')' => {
                if let Some(last_operator_token) = last_token {
                    if last_operator_token == &Token::Operator(Operator::Add)
                        || last_operator_token == &Token::Operator(Operator::Div)
                        || last_operator_token == &Token::Operator(Operator::Prod)
                        || last_operator_token == &Token::Operator(Operator::Sub)
                    {
                        return Err(Error::InvalidSyntax);
                    }
                }
                result.push(Token::Operator(Operator::RightBracket));
                last_token = &Some(Token::Operator(Operator::RightBracket));
                it.next();
            }
            '+' => {
                if let Some(last_operator_token) = last_token {
                    let _ = result.pop();
                    if last_operator_token == &Token::Operator(Operator::Sub) {
                        result.push(Token::Operator(Operator::Sub));
                        last_token = &Some(Token::Operator(Operator::Sub));
                    } else {
                        result.push(Token::Operator(Operator::Add));
                        last_token = &Some(Token::Operator(Operator::Add));
                    }
                } else {
                    result.push(Token::Operator(Operator::Add));
                    last_token = &Some(Token::Operator(Operator::Add));
                }
                it.next();
            }
            '-' => {
                if let Some(last_operator_token) = last_token {
                    let _ = result.pop();
                    if last_operator_token == &Token::Operator(Operator::Sub) {
                        result.push(Token::Operator(Operator::Add));
                        last_token = &Some(Token::Operator(Operator::Add));
                    } else {
                        result.push(Token::Operator(Operator::Add));
                        last_token = &Some(Token::Operator(Operator::Add));
                    }
                } else {
                    result.push(Token::Operator(Operator::Sub));
                    last_token = &Some(Token::Operator(Operator::Sub));
                }
                it.next();
            }
            '/' => {
                if let Some(last_operator_token) = last_token {
                    if last_operator_token == &Token::Operator(Operator::Div)
                        || last_operator_token == &Token::Operator(Operator::Prod)
                        || last_operator_token == &Token::Operator(Operator::Add)
                        || last_operator_token == &Token::Operator(Operator::Sub)
                    {
                        return Err(Error::InvalidSyntax);
                    }
                }
                result.push(Token::Operator(Operator::Div));
                last_token = &Some(Token::Operator(Operator::Div));
                it.next();
            }
            '*' => {
                if let Some(last_operator_token) = last_token {
                    if last_operator_token == &Token::Operator(Operator::Div)
                        || last_operator_token == &Token::Operator(Operator::Prod)
                        || last_operator_token == &Token::Operator(Operator::Add)
                        || last_operator_token == &Token::Operator(Operator::Sub)
                    {
                        return Err(Error::InvalidSyntax);
                    }
                }
                result.push(Token::Operator(Operator::Prod));
                last_token = &Some(Token::Operator(Operator::Prod));
                it.next();
            }
            ' ' => {
                last_token = &None;
                it.next();
            }
            _ => {
                return Err(Error::InvalidExpression(format!("Unknown character {c}")));
            }
        }
    }

    Ok(result)
}

fn get_number<T: Iterator<Item = char>>(c: char, iter: &mut Peekable<T>) -> Result<i64, Error> {
    let mut number = c.to_string().parse::<i64>().map_err(Error::NumberParse)?;
    while let Some(Ok(digit)) = iter.peek().map(|c| c.to_string().parse::<i64>()) {
        number = number * 10 + digit;
        iter.next();
    }
    Ok(number)
}
