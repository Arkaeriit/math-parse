/// Module containing the function to parse math expressions.
mod maths;
mod parse;
mod utils;
mod rpn;
use maths::*;

use std::collections::HashMap;

/// Parse the expression given and apply the optional map of variable that maps
/// variables to math expressions. Return an integer or error out if the result
/// is a floating point number.
pub fn math_parse_int(expression: &str, variable_map: Option<&HashMap<String, String>>) -> Result<i64, MathParseErrors> {
    match math_compute(expression, variable_map)? {
        Number::Int(i)   => Ok(i),
        Number::Float(f) => Err(ReturnFloatExpectedInt(f)),
    }
}

/// Parse the expression given and apply the optional map of variable that maps
/// variables to math expressions. Return a floating point number. If the result
/// is an integer, convert it as a floating point number.
pub fn math_parse_float(expression: &str, variable_map: Option<&HashMap<String, String>>) -> Result<f64, MathParseErrors> {
    match math_compute(expression, variable_map)? {
        Number::Float(f) => Ok(f),
        Number::Int(i)   => Ok(i as f64),
    }
}

/// Return true if the given string contains any character that are used as
/// operators inside of math-parse
pub fn contains_math_char(s: &str) -> bool {
    parse::contains_math_char(s)
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

    /// We wanted to return an int but we got a float instead.
    ReturnFloatExpectedInt(f64),

    /// A given operator was invalid, but we can suggest an other instead.
    BadOperatorHint(char, &'static str),

    /// There was an unwanted zero.
    UnexpectedZero,

    /// There was an unwanted negative number.
    UnexpectedNegative,

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
            ReturnFloatExpectedInt(fp) => write!(f, "An integer was wanted but the floating point number `{fp}` was returned instead."),
            BadOperatorHint(c, s) => write!(f, "The operator '{c}' is invalid. Did you meant '{s}'?"),
            UnexpectedZero => write!(f, "There is a 0 in an operation where it is invalid such as a division or a remainder."),
            UnexpectedNegative => write!(f, "There is a negative number in an operation where it is invalid such as a logical shift."),
            MathParseInternalBug(s) => write!(f, "There is a bug in the math-parse library. The error message is the following:\n{s}\nPlease, report it with the input given to the library to the developer of math-parse over here: https://github.com/Arkaeriit/math-parse"),
        }
    }
}

/* ----------------------------------- RPN ---------------------------------- */

#[derive(Debug)]
pub enum RPN<'a> {
    Name(&'a str),
    UnaryNot,
    UnaryMinus,
    UnaryPlus,
    Multiplication,
    Division,
    IntegerDivision,
    Reminder,
    Addition,
    Subtraction,
    ShiftLeft,
    ShiftRight,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
}

/* --------------------------------- Testing -------------------------------- */

#[test]
fn test_math_compute() {
    let a = 3;
    let b = 9;
    let c = 3*5;
    let variables = HashMap::from([
        ("a".to_string(), "3".to_string()),
        ("b".to_string(), "9".to_string()),
        ("c".to_string(), "(((3)*(5)))".to_string()),
    ]);
    
    let compute_int = |input: &str, output: i64| {
        let res = math_compute(input, Some(&variables)).unwrap();
        if let Number::Int(res) = res {
            assert_eq!(res, output);
        } else {
            panic!("Expected integer instead of float.");
        }
    };

    fn compute_float (input: &str, output: f64) {
        let res = math_compute(input, None).unwrap();
        if let Number::Float(res) = res {
            assert_eq!(res, output);
        } else {
            panic!("Expected float instead of integer.");
        }
    }
    
    compute_int("((3+3)·b+8)*(a-1)", ((3+3)*b+8)*(a-1));
    compute_int("0", 0);
    compute_int("-a+b−c", -a+b-c);
    compute_int("-−-+++-a", ----a);
    compute_int("3%8+99", 3%8+99);
    compute_int("10.0//3.0", 10/3);
    compute_int("!-4", !-4);
    compute_int("((3+4)*(8+(4-1)))-(43+8//2+1)", ((3+4) * (8+(4-1))) - (43+8/2+1));

    compute_float("4×9/4", 4.0*9.0/4.0);
    compute_float("4×9/4.0", 4.0*9.0/4.0);
    compute_float("4.0*9/4", 4.0*9.0/4.0);
    compute_float("4.0·9.0/4", 4.0*9.0/4.0);
    compute_float("4*9.0/4", 4.0*9.0/4.0);
    compute_float("4*9.0/4.0", 4.0*9.0/4.0);
    compute_float("4.0+9-4", 4.0+9.0-4.0);
    compute_float("4+9-4.0", 4.0+9.0-4.0);
    compute_float("4.0+9-4", 4.0+9.0-4.0);
    compute_float("4.0+9.0-4", 4.0+9.0-4.0);
    compute_float("4+9.0-4", 4.0+9.0-4.0);
    compute_float("4+9.0-4.0", 4.0+9.0-4.0);
}


