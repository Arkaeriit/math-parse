/// Module containing the function to parse math expressions
mod maths;

use std::collections::HashMap;

/// Parse the expression given and apply the optional map of variable that maps
/// variables to math expressions.
pub fn math_parse(expression: &str, variable_map: Option<&HashMap<String, String>>) -> Result<i64, MathParseErrors> {
    maths::math_compute(expression, variable_map)
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

    /// The math expression is empty.
    EmptyLine,

    /// An expression that should have been a number but can't be read.
    InvalidNumber(String),

    /// This error should never be raised and should be reported to the
    /// library's maintainer.
    MathParseInternalBug(String),
}

// TODO: display
