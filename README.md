# Math-Parse

A robust and polyvalent library to parse or compute of a math expressions.

Math-Parse has been made with robustness in mind. Unlike other Rust math parsing library, Math-Parse can handle arbitrary big input without stack overflow. A lot of case have been put to ensure that the program can never panic, instead, errors are reported using a custom error type.

## Example of use

```rust
let num1: i64 = MathParse::parse("(1+2)*3").unwrap().solve_int(None).unwrap();
println!("{num1}"); // Prints 9

let num2: f64 = MathParse::parse("5/8+6").unwrap().solve_float(None).unwrap();
println!("{num2}"); // Prints 6.625

let parsed = MathParse::parse("(2+3)*2/5").unwrap().to_tree();
println!("{parsed}"); // Prints (((2 + 3) * 2) / 5)
```

## API

### Parsing expressions

To parse expression, use the `MathParse` object. You can parse expressions in the usual infix notation with `MathParse::parse` and expression in Reverse Polish notation with `MathParse::parse_rpn`.

#### Available operators

The following operators are available:

* `+`: Used as a binary operator for addition. Can also be used as an unary operator with no effect.
* `-` or `−`: Used as a binary operator for subtraction and as a unary operator used to negate a number.
* `*`, `×`, or `·`: Binary operator for multiplication.
* `/`, `∕`, `⁄`, or `÷`: Binary operator for division.
* `%`: Binary operator used to get the remainder of the integer division.
* `//` or `⟌`: Binary operator used for integer division.
* `!` or `~`: Unary operator used for the bitwise not operation.
* `&`: Binary operator used for the bitwise and operation.
* `^`: Binary operator used for the bitwise xor operation.
* `|`: Binary operator used for the bitwise or operation.
* `<<` or `≪`: Binary operator for logical shift to the left.
* `>>` or `≫`: Binary operator for logical shift to the right.

#### Operator precedence

The operator precedence of Math-Parse to read infix notation is quite usual. Here is the operators sorted in decreasing precedence:

1. unary `+`, unary `-`, unary `!`
2. `×`, `/`, `%`, `//`
3. binary `+`, binary `-`
4. `<<`, `>>`
4. `&`
5. `^`
6. `|`

When multiple operators exist for a single operation, they all have the same precedence.

### Using parsed expression

#### Parsed form

Parsed expression can be transformed in a usable parsed form for further processing by the library's user. The expression can either be presented as a tree with the `.to_tree()` method or as a vector of RPN operations with the `.to_rpn()` method.

#### Solving

Parsed objects have functions to compute their result, `.solve_int` and `.solve_float`. Both take as first argument a `&str` which is a mathematical expression and as second argument an optional map of named variable.

Both functions return `Ok(num)` if the computation can be done, where `num` is an `i64` for `math_parse_int` or a `f64` for `math_parse_float`. If the computation can't be done, they return `Err(err)` where `err` is a `MathParseErrors`. The type `MathParseErrors` implements the trait `Display` which format it into an error message that can be read by a human.

Alternatively, there is the `.solve_auto` method that try to give a `i64` result but can fall back to a `f64` result.

#### Named variables

The argument of Math-Parse's solving functions is a map of named variables. It's an optional hash map of strings to strings which can map named variable in the mathematical expression to their value. Here is an example of use:

```rust
let variables = HashMap::from([
    ("a".to_string(), "1".to_string()),
    ("b".to_string(), "3*3".to_string()),
]);
let result = MathParse::parse("a+b").unwrap().solve_int(Some(&variables)).unwrap();
println!("{result}"); // Prints 10
```

As you can see, the values in the map be mathematical expressions (`b` is equal to `3*3`). This makes the map quite powerful. But as the expansion is not done recursively, the value of named variable can not contains other named variables.

## Misc.

An additional function is exposed, `contains_math_char`. This function takes a string as argument and returns true if it contains any character that is considered an operator by Math-Parse. It is meant to sanitize data used around Math-Parse.

Check [the docs](https://docs.rs/math-parse) for more information.

## Example program

You can find in this repository `src/example.rs` which is a small program that uses Math-Parse to compute the mathematical expression given as command line arguments.

