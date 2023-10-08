/// Module containing the function to parse math expressions.
mod maths;

use std::collections::HashMap;

/// Parse the expression given and apply the optional map of variable that maps
/// variables to math expressions.
pub fn math_parse(expression: &str, variable_map: Option<&HashMap<String, String>>) -> Result<i64, MathParseErrors> {
    if let maths::Number::Int(ret) = maths::math_compute(expression, variable_map)? {
        Ok(ret)
    } else {
        todo!("Error for floats.");
    }
}

/* --------------------------------- Errors --------------------------------- */

/// Type used to represent any errors that can happen in the parsing of a math
/// expression.
#[derive(Debug, PartialEq)]
pub enum MathParseErrors {
    /// A parenthesis was opened but never closed.
    UnclosedParenthesis,

    /// A closing parenthesis was used with no matching open parenthesis.
    UnopenedParenthesis,

    /// The math expression is empty. Or the right hand side of an operator is
    /// empty.
    EmptyLine,

    /// An expression that should have been a number but can't be read.
    InvalidNumber(String),

    /// An operator is not where it should be. Like a "*" after a "+", or the
    /// left hand side of an operator being empty.
    MisplacedOperator(char),

    /// An operator is the last element of a line of math.
    TrailingOperator,

    /// A float could not be converted to an int.
    IntConversion(f64),

    /// A binary operation have been tried on a float.
    BinaryOpOnFloat(f64, char),

    /// This error should never be raised and should be reported to the
    /// library's maintainer.
    MathParseInternalBug(String),
}

use MathParseErrors::*;
use std::fmt;

impl fmt::Display for MathParseErrors {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UnclosedParenthesis => write!(f, "A parenthesis was opened but never closed."),
            UnopenedParenthesis => write!(f, "A closing parenthesis was used with no matching open parenthesis."),
            EmptyLine => write!(f, "The math expression is empty. Or the right hand side of an operator is empty."),
            InvalidNumber(s) => write!(f, "The expression `{s}` that should have been a number but can't be read."),
            MisplacedOperator(c) => write!(f, "The operator `{c}`is not where it should be. Or the left hand side of an operator being empty."),
            TrailingOperator => write!(f, "An operator is the last element of a line of math."),
            IntConversion(fp) => write!(f, "The floating point number {fp} could not be converted to an int which is needed."),
            BinaryOpOnFloat(fp, c) => write!(f, "The bitwise operation `{c}` is being performed on the floating point number `{fp}`."),
            MathParseInternalBug(s) => write!(f, "There is a bug in the math-parse library. The error message is the following:\n{s}\nPlease, report it with the input given to the library to the developer of math-parse over here: https://github.com/Arkaeriit/math-parse"),
        }
    }
}

