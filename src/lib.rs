/// Module containing the function to parse math expressions.
mod rpn_stack_manipulation;
mod number_conversion;
mod parse_rpn;
mod tokenize;
mod solve;
mod parse;
mod tree;
mod rpn;

use solve::*;
use parse::math_parse;
use std::collections::HashMap;
use number_conversion::*;

/* --------------------------------- Parsing -------------------------------- */

/// Object generated when parsing a string of math. Can be later used for
/// solving or formatting to other representations.
pub struct MathParse {
    // Internal representation of parsed math is the RPN one. Might or might
    // not change in the future.
    internal: Vec<RPN>
}

impl MathParse {
    /// Parse a math expression in infix notation.
    ///
    /// ```
    /// math_parse::MathParse::parse("3 + 4").unwrap();
    /// ```
    pub fn parse(expression: &str) -> Result<Self, MathParseErrors> {
        let parsed_tree = math_parse(expression)?;
        let internal = rpn::parse_rpn(&parsed_tree)?;
        Ok(MathParse{internal})
    }

    /// Parse a math expression in postfix notation (RPN).
    ///
    /// ```
    /// math_parse::MathParse::parse_rpn("3 4 +").unwrap();
    /// ```
    pub fn parse_rpn(expression: &str) -> Result<Self, MathParseErrors> {
        let internal = parse_rpn::parse_rpn(expression)?;
        Ok(MathParse{internal})
    }
}

/* --------------------------------- Solving -------------------------------- */

impl MathParse {
    /// Does all the computation from a string with a line of math to the final
    /// resulting number. If the result can be an int, return it as
    /// `Ok(Ok(int))`. If it can only be a float, return it as `Ok(Err(floar))`.
    /// If it can't be solved, return `Err(error)`.
    ///
    /// ```
    /// use math_parse::MathParse;
    /// use math_parse::MathParseErrors::*;
    ///
    /// assert_eq!(
    ///     MathParse::parse("3 + 8.0").unwrap().solve_auto(None),
    ///     Ok(Ok(11)));
    /// assert_eq!(
    ///     MathParse::parse("3 - 8.5").unwrap().solve_auto(None),
    ///     Ok(Err(-5.5)));
    /// assert_eq!(
    ///     MathParse::parse("34 + bcd").unwrap().solve_auto(None),
    ///     Err(InvalidNumber("bcd".to_string())));
    /// ```
    ///
    /// A optional map of variable name can be taken as argument.
    pub fn solve_auto(&self, map: Option<&HashMap<String, String>>) -> Result<Result<i64, f64>, MathParseErrors> {
        let map_function = |s: &str| -> Option<String> {
            match map {
                None => None,
                Some(x) => match x.get(s) {
                    Some(x) => Some(x.clone()),
                    None => None,
                },
            }
        };

        match math_solve(&self.internal, &map_function) {
            Err(err)             => Err(err),
            Ok(Number::Int(i))   => Ok(Ok(i)),
            Ok(Number::Float(f)) => Ok(
                if let Ok(i) = f_to_i_strict(f) {
                    Ok(i)
                } else {
                    Err(f)
                }
            ),
        }
    }

    /// Does all the computation from a string with a line of math to the final
    /// resulting number. If the result can be an int, return it as
    /// `Ok(int)`. If it can't be solved as an int, return `Err(error)`.
    ///
    /// ```
    /// use math_parse::MathParse;
    /// use math_parse::MathParseErrors::*;
    ///
    /// assert_eq!(
    ///     MathParse::parse("3 + 8.0").unwrap().solve_int(None),
    ///     Ok(11));
    /// assert_eq!(
    ///     MathParse::parse("3 - 8.5").unwrap().solve_int(None),
    ///     Err(ReturnFloatExpectedInt(-5.5)));
    /// ```
    ///
    /// A optional map of variable name can be taken as argument:
    /// ```
    /// use math_parse::MathParse;
    ///
    /// let variables = std::collections::HashMap::from([
    ///     ("a".to_string(), "1".to_string()),
    ///     ("b".to_string(), "3*3".to_string()),
    /// ]);
    /// let result = MathParse::parse("a+b").unwrap().solve_int(Some(&variables)).unwrap();
    /// assert_eq!(result, 10);
    /// ```
    pub fn solve_int(&self, variable_map: Option<&HashMap<String, String>>) -> Result<i64, MathParseErrors> {
        match self.solve_auto(variable_map)? {
            Ok(i)  => Ok(i),
            Err(f) => Ok(f_to_i_strict(f)?),
        }
    }

    /// Does all the computation from a string with a line of math to the final
    /// resulting number. The result is returned as a `Ok(f64)`.
    /// If it can't be solved, return `Err(error)`.
    ///
    /// ```
    /// use math_parse::MathParse;
    /// use math_parse::MathParseErrors::*;
    ///
    /// assert_eq!(
    ///     MathParse::parse("3 + 8").unwrap().solve_float(None),
    ///     Ok(11.0));
    /// assert_eq!(
    ///     MathParse::parse("3 - 8.5").unwrap().solve_float(None),
    ///     Ok(-5.5));
    /// ```
    ///
    /// A optional map of variable name can be taken as argument.
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
///
/// Example:
/// ```
/// use math_parse::contains_math_char;
/// assert_eq!(contains_math_char("abcd"), false);
/// assert_eq!(contains_math_char("ab+cd"), true);
/// ```
pub fn contains_math_char(s: &str) -> bool {
    tokenize::contains_math_char(s)
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

    /// An operator is not valid in the context of RPN parsing.
    InvalidRPNOperator(char),

    /// The number of elements on the RPN stack is not valid.
    UnbalancedStack,

    /// This error should never be raised and should be reported to the
    /// library's maintainer.
    MathParseInternalBug(String),
}

use MathParseErrors::*;
use std::fmt;

impl fmt::Display for MathParseErrors {
    /// From a `MathParseError`, makes an error message that could even be
    /// shown to the final user.
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
            InvalidRPNOperator(c) => write!(f, "The operators {c} is not valid when parsing RPN expressions."),
            UnbalancedStack => write!(f, "The RPN stack does not contains a valid number of elements. There is too much or not enough operators."),
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
use crate::UnaryOp::*;

impl UnaryOp {
    fn from_char(c: char) -> Result<Self, MathParseErrors> {
        match c {
            '!' => Ok(Not),
            '-' => Ok(Minus),
            '+' => Ok(Plus),
            x   => Err(MathParseInternalBug(format!("{x} is not a valid unary operator."))),
        }
    }
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Not   => write!(f, "!"),
            Minus => write!(f, "-"),
            Plus  => write!(f, "+"),
        }
    }
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
use crate::BinaryOp::*;

impl BinaryOp {
    fn from_char(c: char) -> Result<Self, MathParseErrors> {
        match c {
            '*' | '×' | '·'       => Ok(Multiplication),
            '/' | '∕' | '⁄' | '÷' => Ok(Division),
            '+'                   => Ok(Addition),
            '-' | '−'             => Ok(Subtraction),
            '%'                   => Ok(Reminder),
            '⟌'                   => Ok(IntegerDivision),
            '|'                   => Ok(BitwiseOr),
            '&'                   => Ok(BitwiseAnd),
            '^'                   => Ok(BitwiseXor),
            '≪'                   => Ok(ShiftLeft),
            '≫'                   => Ok(ShiftRight),
            '<'                   => Err(BadOperatorHint('<', "<<")),
            '>'                   => Err(BadOperatorHint('>', ">>")),
            x                     => Err(MathParseInternalBug(format!("{x} is not a valid operator."))),
        }
    }
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Multiplication  => write!(f, "*"),
            Division        => write!(f, "/"),
            IntegerDivision => write!(f, "//"),
            Reminder        => write!(f, "%"),
            Addition        => write!(f, "+"),
            Subtraction     => write!(f, "-"),
            ShiftLeft       => write!(f, "<<"),
            ShiftRight      => write!(f, ">>"),
            BitwiseAnd      => write!(f, "&"),
            BitwiseOr       => write!(f, "|"),
            BitwiseXor      => write!(f, "⊕"), // Not ^ in order not to mistake it for exponentiation.
        }
    }
}

/* ----------------------------------- RPN ---------------------------------- */

/// Elements that make a list of RPN instruction extracted from a math
/// expression.
#[derive(Debug, PartialEq, Clone)]
pub enum RPN {
    Name(String),
    Unary(UnaryOp),
    Binary(BinaryOp),
}

impl fmt::Display for RPN {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RPN::Name(x)   => write!(f, "{x}"),
            RPN::Unary(x)  => write!(f, "{x}"),
            RPN::Binary(x) => write!(f, "{x}"),
        }
    }
}

impl MathParse {
    /// Parse a math expression into list of instructions in Reverse Polish
    /// notation (postfix notation).
    ///
    /// Example:
    /// ```
    /// use math_parse::RPN::*;
    /// use math_parse::UnaryOp::*;
    /// use math_parse::BinaryOp::*;
    /// use math_parse::MathParse;
    ///
    /// assert_eq!(
    ///     MathParse::parse("3-4+(-5)").unwrap().to_rpn(),
    ///     Ok(vec![Name("3".to_string()), Name("4".to_string()), Binary(Subtraction), Name("5".to_string()), Unary(Minus), Binary(Addition)]));
    /// ```
    pub fn to_rpn(&self) -> Result<Vec<RPN>, MathParseErrors> {
        Ok(self.internal.clone())
    }
}

/// Shows a representation of an expression formatted into RPN.
///
/// Example:
/// ```
/// use math_parse::*;
/// let rpn = MathParse::parse("3+1*2").unwrap().to_rpn().unwrap();
/// assert_eq!(
///     rpn_slice_to_string(&rpn),
///     "3 1 2 * +".to_string());
/// ```
pub fn rpn_slice_to_string(rpn: &[RPN]) -> String {
    let mut ret = String::new();
    if rpn.len() == 0 {
        return ret;
    }
    ret.push_str(&format!("{}", rpn[0]));
    for i in 1..rpn.len() {
        ret.push_str(&format!(" {}", rpn[i]));
    }
    ret
}

/* ------------------------------ Tree notation ----------------------------- */

/// Parsed element showed in a tree in infix notation. 
#[derive(Debug, PartialEq, Clone)]
pub enum Tree {
    Name(String),
    Unary(UnaryOp, Box<Tree>),
    Binary(BinaryOp, Box<Tree>, Box<Tree>),
}


impl MathParse {
    /// Parse a math expression into list of instructions as a Tree (infix
    /// notation).
    ///
    /// Example:
    /// ```
    /// use math_parse::Tree::*;
    /// use math_parse::UnaryOp::*;
    /// use math_parse::BinaryOp::*;
    /// use math_parse::MathParse;
    ///
    /// assert_eq!(
    ///     MathParse::parse("3*4+(-5)").unwrap().to_tree(),
    ///     Ok(Binary(Addition,
    ///         Box::new(Binary(Multiplication,
    ///             Box::new(Name("3".to_string())),
    ///             Box::new(Name("4".to_string())))),
    ///         Box::new(Unary(Minus,
    ///             Box::new(Name("5".to_string())))))));
    /// ```
    pub fn to_tree(&self) -> Result<Tree, MathParseErrors> {
        tree::parse_to_tree(&self.internal)
    }
}

impl fmt::Display for Tree {
    /// Show a tree as an infix expression.
    ///
    /// Example:
    /// ```
    /// use math_parse::*;
    /// assert_eq!(
    ///     format!("{}", MathParse::parse("(2+3)*2/5").unwrap().to_tree().unwrap()),
    ///     "(((2 + 3) * 2) / 5)".to_string());
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        enum TreeFmt {
            S(String),
            T(Tree),
        } use TreeFmt::*;
        use Tree::*;

        let mut to_format = vec![T(self.clone())];
        while to_format.len() != 0 {
            match to_format.pop().unwrap() {
                T(Name(s)) => {
                    write!(f, "{s}")?;
                },
                T(Unary(op, next)) => {
                    to_format.push(T(*next));
                    write!(f, "{op}")?;
                },
                T(Binary(op, next_1, next_2)) => {
                    write!(f, "(")?;
                    to_format.push(S(")".to_string()));
                    to_format.push(T(*next_2));
                    to_format.push(S(format!(" {op} ")));
                    to_format.push(T(*next_1));
                },
                S(s) => {
                    write!(f, "{s}")?;
                }
            }
        }
        Ok(())
    }
}

/* --------------------------------- Testing -------------------------------- */

#[cfg(test)]
fn name_r(s: &str) -> RPN {
    RPN::Name(s.to_string())
}
#[cfg(test)]
fn name_p(s: &str) -> tokenize::MathValue {
    tokenize::MathValue::Name(s)
}
#[cfg(test)]
fn name_t(s: &str) -> Tree {
    Tree::Name(s.to_string())
}

#[cfg(test)]
fn math_solve_int(expression: &str) -> Result<i64, MathParseErrors> {
    MathParse::parse(expression)?.solve_int(None)
}
#[cfg(test)]
fn math_solve_float(expression: &str) -> Result<f64, MathParseErrors> {
    MathParse::parse(expression)?.solve_float(None)
}
#[cfg(test)]
fn compute(expression: &str, variable_map: Option<&HashMap<String, String>>) -> Result<solve::Number, MathParseErrors> {
    MathParse::parse(expression)?.solve_number(variable_map)
}
#[cfg(test)]
fn parse_rpn(expression: &str) -> Result<Vec<RPN>, MathParseErrors> {
    MathParse::parse(expression)?.to_rpn()
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
        match res {
            Number::Float(res) => {
                assert_eq!(res, output);
            },
            Number::Int(res) => {
                assert_eq!(res as f64, output);
            },
        }
    }
    
    compute_int("((3+3)·b+8)*(a-1)", ((3+3)*b+8)*(a-1));
    compute_int("0", 0);
    compute_int("-a + b − c", -a + b - c);
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
    assert_eq!(math_solve_int("3.0+3.0"), Ok(6));
    assert_eq!(math_solve_int("3.2+3.0"), Err(ReturnFloatExpectedInt(6.2)));

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
fn test_to_rpn() {
    use RPN::*;
    use UnaryOp::*;
    use BinaryOp::*;
    assert_eq!(parse_rpn("8/2").unwrap(), vec![name_r("8"), name_r("2"), Binary(Division)]);
    assert_eq!(parse_rpn("-3+4").unwrap(), vec![name_r("3"), Unary(Minus), name_r("4"), Binary(Addition)]);
}

#[test]
fn test_parse_rpn() {
    fn solve_rpn(expression: &str) -> Result<i64, MathParseErrors> {
        MathParse::parse_rpn(expression)?.solve_int(None)
    }

    assert_eq!(solve_rpn("3 4 + 2 *"), Ok(14));
    assert_eq!(solve_rpn("3 4 2 + *"), Ok(18));
    assert_eq!(solve_rpn("3 (4 + 3) 2 + *"), Err(InvalidRPNOperator('(')));
    assert_eq!(solve_rpn("3 2 + *"), Err(UnbalancedStack));
}

#[test]
fn test_misc_errors() {
    match MathParse::parse("3 3 +") {
        Err(EmptyLine) => {/* Expected */},
        Ok(_) => {panic!("Should not have been solved.");},
        Err(x) => {panic!("Should not have been {x:?}");},
    }

    assert_eq!(compute("1 - (1*3)", None), Ok(Number::Int(-2)));
    assert_eq!(compute("1+-1", None), Ok(Number::Int(0)));
    assert_eq!(compute("1 + - 1", None), Ok(Number::Int(0)));
}

#[test]
fn test_readme_example() {
    let num1: i64 = MathParse::parse("(1+2)*3").unwrap().solve_int(None).unwrap();
    assert_eq!(num1, 9); // Prints 9

    let num2: f64 = MathParse::parse("5/8+6").unwrap().solve_float(None).unwrap();
    assert_eq!(num2, 6.625); // Prints 6.625

    let parsed = MathParse::parse("(2+3)*2/5").unwrap().to_tree().unwrap();
    assert_eq!(
        format!("{parsed}").as_str(),
        "(((2 + 3) * 2) / 5)".to_string());
}
