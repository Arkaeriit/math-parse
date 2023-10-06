/// Module containing the function to parse math expressions
mod maths;

use std::collections::HashMap;

/// Parse the expression given and apply the optional map of variable that maps
/// variables to math expressions.
pub fn math_parse(expression: &str, variable_map: Option<&HashMap<String, String>>) -> Result<i64, String> {
    maths::math_compute(expression, variable_map)
}

