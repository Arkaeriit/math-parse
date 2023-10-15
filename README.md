# Math-Parse

A fast library to parse and then compute the result of a math expression.

## Example of use

```rust
let num1: i64 = math_parse::math_parse_int("(1+2)*3", None).unwrap();
println!("{num1}"); // Prints 9

let num2: f64 = math_parse::math_parse_float("5/8+6", None).unwrap();
println!("{num2}"); // Prints 6.625
```

## API

Math-Parse only exposes two principal functions, `math_parse_int` and `math_parse_float`. Both take as first argument a `&str` which is a mathematical expression and as second argument an optional map of named variable.

Both functions return `Ok(num)` if the computation can be done, where `num` is an `i64` for `math_parse_int` or a `f64` for `math_parse_float`. If the computation can't be done, they return `Err(err)` where `err` is a `MathParseErrors`. The type `MathParseErrors` implements the trait `Display` which format it into an error message that can be read by a human.

An additional third function is exposed, `contains_math_char`. This function takes a string as argument and returns true if it contains any character that is considered an operator by Math-Parse. It is meant to sanitize data used around Math-Parse.

Check https://docs.rs/math-parse for more information.

## Named variables

The second argument of Math-Parse's functions is a map of named variables. It's an optional hash map of strings to strings which can map named variable in the mathematical expression to their value. Here is an example of use:

```rust
let variables = HashMap::from([
    ("a".to_string(), "1".to_string()),
    ("b".to_string(), "3*3".to_string()),
]);
let result = math_parse_int("a+b", Some(&variables));
println!("{result}"); // Prints 10
```

As you can see, the values in the map be mathematical expressions (`c` is equal to `3*3`). This makes the map quite powerful. But as the expansion is not done recursively, the value of named variable can not contains other named variables.

## Available operators

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

## Number type

Math-Parse can handle both integer and floating points number together. Integer are read from the input string either as written in base 10 (`1245`) or prefixed by `0x` and written in base 16 (`0xabcd`). Floating point numbers are written in base 10 with a decimal dot (`123.5`).

Operations can be made between multiple types of numbers. Addition, subtraction, multiplication, and remainder will return an integer when performed between to integers and return a floating point number if at least one of their terms is a floating point number. Division will always return a floating number and integer division will always return an integer. Bitwise operations such as `!` only work with integers and will output an error if fed a floating point number.

## Operator precedence

The operator precedence of Math-Parse is quite usual. Here is the operators sorted in decreasing precedence:

1. unary `+`, unary `-`, unary `!`
2. `×`, `/`, `%`, `//`
3. binary `+`, binary `-`
4. `<<`, `>>`
4. `&`
5. `^`
6. `|`

When multiple operators exist for a single operation, they all have the same precedence.

## Invalid operations

Some operations are invalid such as a division by zero, or a logical shift with a negative number. If such an operation is encountered, an error will be returned.

## Example program

You can find in this repo `src/example.rs` which is a small program that uses Math-Parse to compute the mathematical expression given as command line arguments.

