# ShuntingYard Rust implementation

This repo provide a simple implementation of the Shunting Yard implementation that estimate result of a math expression passing through Revese Polish Notation (aka RPN). This specific solution support the following operators:

* Arithmetic operators: [+, -, *, /]
* Negative numbers using the unary operator: '-'
* Brackets: ['(', ')']
* Ignore all whitespaces

### Layout description

The executable starts bringing up two threads:

* The first one is responsible of caching SIGINT temination signals, sending a termination trigger on a dedicated channel to termiate execution of secondary thread
* The second thread is responsible of waiting for user input calculating the result of provided expression
* The main thread at this point just wait endlessly for termination trigger

### Data validation

From the input data the parser eveluate all at once the following conditions:
* If we have non sense sequences, InvalidSyntax error is returned to the end user
* If we have consecutive sign operators, the last pushed operator is poped from the operator stack and it is replaced by the correct operator according to the following logic:
    * \+ \* \+ = +
    * \+ \* \- = -
    * \- \* \+ = -
    * \- \* \- = +

### Logic description

Staring from the string expression, the code follows these steps:

* Validate expression and get a list of Token from input:
    * Token can be of type:
      * Numeric(i64)
      * Operator(Operator)
    * Only these operators are accepted as valid:
      * Operator:
        * Add -> from('+')
        * Sub -> from('-')
        * Prod -> from('*')
        * Div -> from('/')
        * LeftBracket -> from('(')
        * RightBracket -> from(')')
    * Compute RPN from tokens list following this alghoritm:
    ```text
    This function convert a list of token to Reverse Polish Notation
    following this logic
        While there are tokens to be read:
         Read a token
         If it's a number add it to queue
         If it's an operator
                While there's an operator on the top of the stack with greater precedence that is not a left bracket:
                        Pop operators from the stack onto the output queue
                Push the current operator onto the stack
         If it's a left bracket push it onto the stack
         If it's a right bracket
              While there's not a left bracket at the top of the stack:
                       Pop operators from the stack onto the output queue.
               Pop the left bracket from the stack and discard it
    While there are operators on the stack, pop them to the queue
    ```
    * Compute result from RPN following this logic:
    ```text
    While there is a value in output queue:
        pop value:
            if it is a number:
                push to stack
            if it is an operator:
                pop last two values of the stack and apply operator, pushing result back to stack
        if stack contains only a single value, return it as it is the result
    ```
    * If anything do not work properly, the RPN is wrapped within the return Error type

### How to run, test and build

```shell
# Run setting log level (This code add logs only in info and debug)
RUST_LOG=OFF cargo run
RUST_LOG=INFO cargo run
RUST_LOG=WARN cargo run
RUST_LOG=ERROR cargo run
RUST_LOG=DEBUG cargo run
RUST_LOG=TRACE cargo run
# Test
cargo test
# Build
cargo build
# Build release
cargo build --release
```

### NOTES

* This code does not take into account left/right associativity of operator. This can cause wrong results in particular operation
* Maybe this code is missig of specific non sense patterns that can cause failures
* If during result computation, a bracket is hit, the code will paniic due to unreachable macro usage with appropriate error message

### Test table

| Index | Test Expression              | Test Result | RPN notation               |
|-------|------------------------------|-------------|----------------------------|
| 1     | 4+2                          | 6           | + 2 4                      |
| 2     | 3    * 6 - 7  + 2            | 13          | + 2 - 7 * 6 3              |
| 3     | (3+4) +  7 *2 -1-9           | 11          | - 9 - 1 + * 2 7 + 4 3      |
| 4     | (8 -1 +3)  *6 -((3+7)*2  )   | 40          | - * 2 + 7 3 * 6 + 3 - 1 8  |
| 5     | 4+18/(9-3)                   | 7           | + / - 3 9 18 4             |

#### Refs

* [Shunting Yard Alghoritm](https://brilliant.org/wiki/shunting-yard-algorithm/)