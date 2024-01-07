/// Module containing the function to parse math expressions.
mod rpn_stack_manipulation;
mod solve;
mod parse;
mod utils;
mod tree;
mod rpn;

use solve::*;
use parse::math_parse;
use std::collections::HashMap;

/* --------------------------------- Parsing -------------------------------- */

/// Object generated when parsing a string of math. Can be later used for
/// solving or formatting to other representations.
pub struct ParsedMath {
    // Internal representation of parsed math is the infix one. Might or might
    // not change in the future.
    internal: Vec<parse::MathValue>
}

impl ParsedMath {
    pub fn parse(expression: &str) -> Result<Self, MathParseErrors> {
        let internal = math_parse(expression)?;
        Ok(ParsedMath{internal})
    }
}

/* --------------------------------- Solving -------------------------------- */

impl ParsedMath {
    /// Does all the computation from a string with a line of math to the final
    /// resulting number. If the result can be an int, return it as
    /// `Ok(Ok(int))`. If it can only be a float, return it as `Ok(Err(floar))`.
    /// If it can't be solved, return `Err(error)`.
    pub fn solve_auto(&self, map: Option<&HashMap<String, String>>) -> Result<Result<i64, f64>, MathParseErrors> {
        let rpn = self.to_rpn()?;
        match math_solve(&rpn, map) {
            Ok(Number::Int(i))   => Ok(Ok(i)),
            Ok(Number::Float(f)) => Ok(Err(f)),
            Err(err)             => Err(err),
        }
    }

    /// Parse the expression given and apply the optional map of variable that
    /// maps variables to math expressions. Return an integer or error out if
    /// the result is a floating point number.
    pub fn solve_int(&self, variable_map: Option<&HashMap<String, String>>) -> Result<i64, MathParseErrors> {
        match self.solve_auto(variable_map)? {
            Ok(i)  => Ok(i),
            Err(f) => Err(ReturnFloatExpectedInt(f)),
        }
    }

    /// Parse the expression given and apply the optional map of variable that
    /// maps variables to math expressions. Return a floating point number. If
    /// the result is an integer, convert it as a floating point number.
    pub fn solve_float(&self, variable_map: Option<&HashMap<String, String>>) -> Result<f64, MathParseErrors> {
        match self.solve_auto(variable_map)? {
            Ok(i)  => Ok(i as f64),
            Err(f) => Ok(f),
        }
    }

    /// Solve the result as a number, for internal use.
    fn solve_number(&self, variable_map: Option<&HashMap<String, String>>) -> Result<solve::Number, MathParseErrors> {
        Ok(match self.solve_auto(variable_map)? {
            Ok(i)  => solve::Number::Int(i),
            Err(f) => solve::Number::Float(f),
        })
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

/* ------------------------------- Operations ------------------------------- */

/// Available unary operations.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum UnaryOp {
    Not,
    Minus,
    Plus
}

/// Available binary operations.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum BinaryOp {
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

/* ----------------------------------- RPN ---------------------------------- */

/// Elements that make a list of RPN instruction extracted from a math
/// expression.
#[derive(Debug, PartialEq)]
pub enum RPN {
    Name(String),
    Unary(UnaryOp),
    Binary(BinaryOp),
}

impl ParsedMath {
    /// Parse a math expression into list of instructions in Reverse Polish
    /// notation (postfix notation).
    ///
    /// Example:
    /// ```
    /// use math_parse::RPN::*;
    /// use math_parse::UnaryOp::*;
    /// use math_parse::BinaryOp::*;
    /// use math_parse::ParsedMath;
    ///
    /// assert_eq!(
    ///     ParsedMath::parse("3-4+(-5)").unwrap().to_rpn(),
    ///     Ok(vec![Name("3".to_string()), Name("4".to_string()), Binary(Subtraction), Name("5".to_string()), Unary(Minus), Binary(Addition)]));
    /// ```
    pub fn to_rpn(&self) -> Result<Vec<RPN>, MathParseErrors> {
        rpn::parse_rpn(&self.internal)
    }
}

/* ------------------------------ Tree notation ----------------------------- */

#[derive(Debug, PartialEq)]
pub enum Tree {
    Name(String),
    Unary(UnaryOp, Box<Tree>),
    Binary(BinaryOp, Box<Tree>, Box<Tree>),
}


impl ParsedMath {
    pub fn to_tree(&self) -> Result<Tree, MathParseErrors> {
        let rpn = self.to_rpn()?;
        tree::parse_to_tree(&rpn)
    }
}

/* --------------------------------- Testing -------------------------------- */

#[cfg(test)]
fn name_r(s: &str) -> RPN {
    RPN::Name(s.to_string())
}
#[cfg(test)]
fn name_p(s: &str) -> parse::MathValue {
    parse::MathValue::Name(s.to_string())
}
#[cfg(test)]
fn name_t(s: &str) -> Tree {
    Tree::Name(s.to_string())
}

#[cfg(test)]
fn math_solve_int(expression: &str) -> Result<i64, MathParseErrors> {
    ParsedMath::parse(expression)?.solve_int(None)
}
#[cfg(test)]
fn math_solve_float(expression: &str) -> Result<f64, MathParseErrors> {
    ParsedMath::parse(expression)?.solve_float(None)
}
#[cfg(test)]
fn compute(expression: &str, variable_map: Option<&HashMap<String, String>>) -> Result<solve::Number, MathParseErrors> {
    ParsedMath::parse(expression)?.solve_number(variable_map)
}
#[cfg(test)]
fn parse_rpn(expression: &str) -> Result<Vec<RPN>, MathParseErrors> {
    ParsedMath::parse(expression)?.to_rpn()
}

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
    assert_eq!(math_solve_int("3+3"), Ok(6));
    assert_eq!(math_solve_int("3.0+3.0"), Err(ReturnFloatExpectedInt(6.0)));

    assert_eq!(math_solve_float("3+3"    ), Ok(6.0));
    assert_eq!(math_solve_float("3.0+3.0"), Ok(6.0));

    assert_eq!(contains_math_char("ab+cd"), true);
    assert_eq!(contains_math_char("abcd"), false);
}

#[test]
fn test_bitwise_on_float() {
    fn test_operator(op: char) {
        let exp = format!("3.1{op}4.2");
        assert_eq!(math_solve_int(&exp), Err(BinaryOpOnFloat(3.1, op)));
    }

    let operators = ['^', '|', '&', '≪', '≫'];
    for op in &operators {
        test_operator(*op);
    }
}

#[test]
fn test_operator_hints() {
    assert_eq!(math_solve_int("3876<4"), Err(BadOperatorHint('<', "<<")));
    assert_eq!(math_solve_int("3876>4"), Err(BadOperatorHint('>', ">>")));
}

#[test]
fn test_rpn() {
    use RPN::*;
    use UnaryOp::*;
    use BinaryOp::*;
    assert_eq!(parse_rpn("8/2").unwrap(), vec![name_r("8"), name_r("2"), Binary(Division)]);
    assert_eq!(parse_rpn("-3+4").unwrap(), vec![name_r("3"), Unary(Minus), name_r("4"), Binary(Addition)]);
}

