/// Module containing the function to parse math expressions.
mod solve;
mod parse;
mod utils;
mod rpn;

use solve::*;
use parse::math_parse;
use std::collections::HashMap;

/* --------------------------------- Solving -------------------------------- */

/// Does all the computation from a string with a line of math to the final
/// resulting number.
fn compute(expression: &str, map: Option<&HashMap<String, String>>) -> Result<Number, MathParseErrors> {
    let rpn = parse_rpn(expression)?;
    math_solve(&rpn, map)
}

/// Parse the expression given and apply the optional map of variable that maps
/// variables to math expressions. Return an integer or error out if the result
/// is a floating point number.
pub fn math_solve_int(expression: &str, variable_map: Option<&HashMap<String, String>>) -> Result<i64, MathParseErrors> {
    match compute(expression, variable_map)? {
        Number::Int(i)   => Ok(i),
        Number::Float(f) => Err(ReturnFloatExpectedInt(f)),
    }
}

/// Parse the expression given and apply the optional map of variable that maps
/// variables to math expressions. Return a floating point number. If the result
/// is an integer, convert it as a floating point number.
pub fn math_solve_float(expression: &str, variable_map: Option<&HashMap<String, String>>) -> Result<f64, MathParseErrors> {
    match compute(expression, variable_map)? {
        Number::Float(f) => Ok(f),
        Number::Int(i)   => Ok(i as f64),
    }
}

/* ---------------------------------- Misc. --------------------------------- */

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

/// Elements that make a list of RPN instruction extracted from a math
/// expression.
#[derive(Debug, PartialEq)]
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

/// Parse a math expression into a RPN list of instructions.
pub fn parse_rpn<'a>(expression: &'a str) -> Result<Vec<RPN<'a>>, MathParseErrors> {
    let parsed = math_parse(expression)?;
    rpn::parse_rpn(&parsed)
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
        let res = compute(input, Some(&variables)).unwrap();
        if let Number::Int(res) = res {
            assert_eq!(res, output);
        } else {
            panic!("Expected integer instead of float.");
        }
    };

    fn compute_float (input: &str, output: f64) {
        let res = compute(input, None).unwrap();
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
    compute_int("((0xFF&0xF)|0x10)^0x3", ((0xFF & 0xF) | 0x10) ^ 0x3);
    compute_int("(10<<5)>>(2<<1)", (10 << 5) >> (2 << 1));

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

    compute_float("4.5%2.1", 4.5 % 2.1);
    compute_float("8%2.2", 8.0 % 2.2);
    compute_float("33.4%2", 33.4 % 2.0);
}

#[test]
fn test_butchered_rpn() {
    match parse_rpn("3++") {
        Ok(x) => {
            panic!("{x:?} should not have been solved.");
        },
        Err(TrailingOperator) => {
            // Expected result
        },
        Err(x) => {
            panic!("{x:?} is not the expected error.");
        }
    }
}

#[test]
fn test_api() {
    assert_eq!(math_solve_int("3+3",     None), Ok(6));
    assert_eq!(math_solve_int("3.0+3.0", None), Err(ReturnFloatExpectedInt(6.0)));

    assert_eq!(math_solve_float("3+3",     None), Ok(6.0));
    assert_eq!(math_solve_float("3.0+3.0", None), Ok(6.0));

    assert_eq!(contains_math_char("ab+cd"), true);
    assert_eq!(contains_math_char("abcd"), false);
}

#[test]
fn test_bitwise_on_float() {
    fn test_operator(op: char) {
        let exp = format!("3.1{op}4.2");
        assert_eq!(math_solve_int(&exp, None), Err(BinaryOpOnFloat(3.1, op)));
    }

    let operators = ['^', '|', '&', '≪', '≫'];
    for op in &operators {
        test_operator(*op);
    }
}

#[test]
fn test_operator_hints() {
    assert_eq!(math_solve_int("3876<4", None), Err(BadOperatorHint('<', "<<")));
    assert_eq!(math_solve_int("3876>4", None), Err(BadOperatorHint('>', ">>")));
}

#[test]
fn test_rpn() {
    assert_eq!(parse_rpn("8/2").unwrap(), vec![RPN::Name("2"), RPN::Name("8"), RPN::Division]);
    assert_eq!(parse_rpn("-3+4").unwrap(), vec![RPN::Name("4"), RPN::Name("3"), RPN::UnaryMinus, RPN::Addition]);
}

