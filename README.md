# Math-Parse

A fast library to compute the result of a math expression.

## Example of use

TODO

## Named variables

TODO

## Available operators

The following operators are available:

* `+`: Used as a binary operator for addition. Can also be used as an unary operator with no effect.
* `-` or `−`: Used as a binary operator for subtraction and as a unary operator used to negate a number.
* `*`, `×`, or `·`: Binary operator for multiplication.
* `/`, `∕`, `⁄`, or `÷`: Binary operator for division.
* `%`: Binary operator used to get the remainder of the integer division.
* `//` or `⟌`: Binary operator used for integer division.
* `!` or `~`: Unary operator used for the bitwise not operation.

## Number type

Math-Parse can handle both integer and floating points number together. Integer are read from the input string either as written in base 10 (`1245`) or prefixed by `0x` and written in base 16 (`0xabcd`). Floating point numbers are written in base 10 with a decimal dot (`123.5`).

Operations can be made between multiple types of numbers. Addition, subtraction, multiplication, and remainder will return an integer when performed between to integers and return a floating point number if at least one of their terms is a floating point number. Division will always return a floating number and integer division will always return an integer. Bitwise operations such as `!` only work with integers and will output an error if fed a floating point number.

TODO: API

