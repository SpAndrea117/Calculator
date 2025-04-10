use log::debug;

use super::{
    Error,
    eval::{Operator, Token, parse_expr},
};

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
            tokens: parse_expr(expr)?,
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

#[cfg(test)]
mod test {
    use crate::internal::eval::parse_expr;

    use super::{Error, Operator, ShuntingYard, Token};

    #[test]
    fn test_shunting_yard_data_struct_from_expression_signed_negative() {
        let expression = "4 + 18/(9--3)";

        assert_eq!(
            parse_expr(expression).unwrap(),
            vec![
                Token::Number(4),                        // 4
                Token::Operator(Operator::Add),          // +
                Token::Number(18),                       // 18
                Token::Operator(Operator::Div),          // /
                Token::Operator(Operator::LeftBracket),  // (
                Token::Number(9),                        // 9
                Token::Operator(Operator::Add),          // - * - = +
                Token::Number(3),                        // 3
                Token::Operator(Operator::RightBracket), // )
            ]
        );
    }

    #[test]
    fn test_shunting_yard_data_struct_from_expression_signed_positive() {
        let expression = "4 + 18/(9-+3)";

        assert_eq!(
            parse_expr(expression).unwrap(),
            vec![
                Token::Number(4),                        // 4
                Token::Operator(Operator::Add),          // +
                Token::Number(18),                       // 18
                Token::Operator(Operator::Div),          // /
                Token::Operator(Operator::LeftBracket),  // (
                Token::Number(9),                        // 9
                Token::Operator(Operator::Sub),          // - * + = -
                Token::Number(3),                        // 3
                Token::Operator(Operator::RightBracket), // )
            ]
        );
    }

    #[test]
    fn test_shunting_yard_data_struct_from_expression_invalid_prod() {
        let expression = "4 + 18/(9-*3)";

        assert_eq!(parse_expr(expression), Err(Error::InvalidSyntax));
    }

    #[test]
    fn test_shunting_yard_data_struct_from_expression_invalid_div() {
        let expression = "4 + 18/(9-/3)";

        assert_eq!(parse_expr(expression), Err(Error::InvalidSyntax));
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
