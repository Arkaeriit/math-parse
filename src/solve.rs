use std::collections::HashMap;
use crate::MathParseErrors;
use crate::MathParseErrors::*;
use crate::RPN;
use crate::RPN::*;

/* ---------------------------------- Maths --------------------------------- */

/// Reads a Names and transform any name being a key in the map to it's value.
/// If map is None, nothing is done.
fn read_name(name: &str, map: Option<&HashMap<String, String>>) -> Result<Number, MathParseErrors> {
    let map = if let Some(m) = map {
        m
    } else {
        return number_from_string(name);
    };

    if let Some(new_name) = map.get(&name.to_string()) {
        let num = crate::compute(&new_name, None)?;
        Ok(num)
    } else {
        number_from_string(name)
    }
}

/// Pop one number from the stack.
fn pop_one(number_stack: &mut Vec<Number>) -> Result<Number, MathParseErrors> {
    if let Some(num) = number_stack.pop() {
        Ok(num)
    } else {
        Err(MathParseInternalBug(format!("Error, unable to pop one, no element on the stack.")))
    }
}

/// Pop two numbers from the stack.
fn pop_two(number_stack: &mut Vec<Number>) -> Result<(Number, Number), MathParseErrors> {
    let num_1 = if let Some(n) = number_stack.pop() {
        n
    } else {
        return Err(MathParseInternalBug(format!("Error, unable to pop two, no element on the stack.")));
    };
    let num_2 = if let Some(n) = number_stack.pop() {
        n
    } else {
        return Err(MathParseInternalBug(format!("Error, unable to pop two, only one element on the stack.")));
    };
    Ok((num_1, num_2))
}

enum Unary {
    Not,
    Minus,
    Plus,
}

/// Execute the given unary operation on the top element of the stack.
fn compute_unary(number_stack: &mut Vec<Number>, op: Unary) -> Result<(), MathParseErrors> {
    let num = pop_one(number_stack)?;
    let computed = match op {
        Unary::Not => (!num)?,
        Unary::Minus => Int(-1) * num,
        Unary::Plus => num,
    };
    number_stack.push(computed);
    Ok(())
}

enum Binary {
    Mult,
    Div,
    IDiv,
    Rem,
    Add,
    Sub,
    SLe,
    SRi,
    And,
    Or,
    Xor,
}

/// Execute the given binary operation on the top two elements of the stack.
fn compute_binary(number_stack: &mut Vec<Number>, op: Binary) -> Result<(), MathParseErrors> {
    let (num_1, num_2) = pop_two(number_stack)?;
    let computed = match op {
        Binary::Mult => num_1 * num_2,
        Binary::Div  => (num_1 / num_2)?,
        Binary::IDiv => num_1.integer_div(num_2)?,
        Binary::Rem  => (num_1 % num_2)?,
        Binary::Add  => num_1 + num_2,
        Binary::Sub  => num_1 - num_2,
        Binary::SLe  => (num_1 << num_2)?,
        Binary::SRi  => (num_1 >> num_2)?,
        Binary::And  => (num_1 & num_2)?,
        Binary::Or   => (num_1 | num_2)?,
        Binary::Xor  => (num_1 ^ num_2)?,
    };
    number_stack.push(computed);
    Ok(())
}

/// Execute a single RPN action and update the stack of numbers accordingly.
fn exec_rpn_action(number_stack: &mut Vec<Number>, action: &RPN, map: Option<&HashMap<String, String>>) -> Result<(), MathParseErrors> {
    match action {
        RPN::Name(x) => {
            number_stack.push(read_name(x, map)?);
            Ok(())
        },
        UnaryNot => compute_unary(number_stack, Unary::Not),
        UnaryMinus => compute_unary(number_stack, Unary::Minus),
        UnaryPlus => compute_unary(number_stack, Unary::Plus),
        Multiplication => compute_binary(number_stack, Binary::Mult),
        Division => compute_binary(number_stack, Binary::Div),
        IntegerDivision => compute_binary(number_stack, Binary::IDiv),
        Reminder => compute_binary(number_stack, Binary::Rem),
        Addition => compute_binary(number_stack, Binary::Add),
        Subtraction => compute_binary(number_stack, Binary::Sub),
        ShiftLeft => compute_binary(number_stack, Binary::SLe),
        ShiftRight => compute_binary(number_stack, Binary::SRi),
        BitwiseAnd => compute_binary(number_stack, Binary::And),
        BitwiseOr => compute_binary(number_stack, Binary::Or),
        BitwiseXor => compute_binary(number_stack, Binary::Xor),
    }
}

/// Reads a line of math that contains only values, operations, and parenthesis
/// and returns a computed result.
pub fn math_solve(rpn_actions: &[RPN], map: Option<&HashMap<String, String>>) -> Result<Number, MathParseErrors> {

    let mut number_stack = Vec::<Number>::new();

    for action in rpn_actions {
        exec_rpn_action(&mut number_stack, action, map)?;
    }

    pop_one(&mut number_stack)
}

/* --------------------------------- Numbers -------------------------------- */

use std::ops::*;

/// A type representing the numbers understood by math-parse. Math operation can
/// be formed with numbers of different types and the time of the result will
/// be chosen in the most sensible way.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Number {
    Int(i64),
    Float(f64),
}
use Number::*;

impl Add for Number {
    type Output = Self;
    
    fn add(self, other: Self) -> Self {
        match (self, other) {
            (Int(s),   Int(o))   => Int(s + o),
            (Float(s), Int(o))   => Float(s + i_to_f(o)),
            (Int(s),   Float(o)) => Float(i_to_f(s) + o),
            (Float(s), Float(o)) => Float(s + o),
        }
    }
}

impl Sub for Number {
    type Output = Self;
    
    fn sub(self, other: Self) -> Self {
        match (self, other) {
            (Int(s),   Int(o))   => Int(s - o),
            (Float(s), Int(o))   => Float(s - i_to_f(o)),
            (Int(s),   Float(o)) => Float(i_to_f(s) - o),
            (Float(s), Float(o)) => Float(s - o),
        }
    }
}

impl Mul for Number {
    type Output = Self;
    
    fn mul(self, other: Self) -> Self {
        match (self, other) {
            (Int(s),   Int(o))   => Int(s * o),
            (Float(s), Int(o))   => Float(s * i_to_f(o)),
            (Int(s),   Float(o)) => Float(i_to_f(s) * o),
            (Float(s), Float(o)) => Float(s * o),
        }
    }
}

impl Div for Number {
    type Output = Result<Number, MathParseErrors>;
    
    fn div(self, other: Self) -> Result<Self, MathParseErrors> {
        other.err_on_zero()?;
        match (self, other) {
            (Int(s),   Int(o))   => Ok(Float(i_to_f(s) / i_to_f(o))),
            (Float(s), Int(o))   => Ok(Float(s / i_to_f(o))),
            (Int(s),   Float(o)) => Ok(Float(i_to_f(s) / o)),
            (Float(s), Float(o)) => Ok(Float(s / o)),
        }
    }
}

impl Rem for Number {
    type Output = Result<Number, MathParseErrors>;
    
    fn rem(self, other: Self) -> Result<Self, MathParseErrors> {
        other.err_on_zero()?;
        match (self, other) {
            (Int(s),   Int(o))   => Ok(Int(s % o)),
            (Float(s), Int(o))   => Ok(Float(s % i_to_f(o))),
            (Int(s),   Float(o)) => Ok(Float(i_to_f(s) % o)),
            (Float(s), Float(o)) => Ok(Float(s % o)),
        }
    }
}

impl Not for Number {
    type Output = Result<Number, MathParseErrors>;
    
    fn not(self) -> Result<Number, MathParseErrors> {
        match self {
            Int(s) => Ok(Int(!s)),
            Float(s) => Err(BinaryOpOnFloat(s, '!')),
        }
    }
}

impl BitXor for Number {
    type Output = Result<Number, MathParseErrors>;
    
    fn bitxor(self, other: Self) -> Result<Number, MathParseErrors> {
        self.err_on_float('^')?;
        other.err_on_float('^')?;
        match (self, other) {
            (Int(s),   Int(o))   => Ok(Int(s ^ o)),
            _                    => Err(MathParseInternalBug("Invalid type check on binxor.".to_string())),
        }
    }
}

impl BitAnd for Number {
    type Output = Result<Number, MathParseErrors>;
    
    fn bitand(self, other: Self) -> Result<Number, MathParseErrors> {
        self.err_on_float('&')?;
        other.err_on_float('&')?;
        match (self, other) {
            (Int(s),   Int(o))   => Ok(Int(s & o)),
            _                    => Err(MathParseInternalBug("Invalid type check on binand.".to_string())),
        }
    }
}

impl BitOr for Number {
    type Output = Result<Number, MathParseErrors>;
    
    fn bitor(self, other: Self) -> Result<Number, MathParseErrors> {
        self.err_on_float('|')?;
        other.err_on_float('|')?;
        match (self, other) {
            (Int(s),   Int(o))   => Ok(Int(s | o)),
            _                    => Err(MathParseInternalBug("Invalid type check on binor.".to_string())),
        }
    }
}

impl Shl for Number {
    type Output = Result<Number, MathParseErrors>;
    
    fn shl(self, other: Self) -> Result<Number, MathParseErrors> {
        self.err_on_float('≪')?;
        other.err_on_float('≪')?;
        other.err_on_negative()?;
        match (self, other) {
            (Int(s),   Int(o))   => Ok(Int(s << o)),
            _                    => Err(MathParseInternalBug("Invalid type check on Shl.".to_string())),
        }
    }
}

impl Shr for Number {
    type Output = Result<Number, MathParseErrors>;
    
    fn shr(self, other: Self) -> Result<Number, MathParseErrors> {
        self.err_on_float('≫')?;
        other.err_on_float('≫')?;
        other.err_on_negative()?;
        match (self, other) {
            (Int(s),   Int(o))   => Ok(Int(s >> o)),
            _                    => Err(MathParseInternalBug("Invalid type check on Shr.".to_string())),
        }
    }
}

impl Number {
    /// Return an error related to the given operator if the number is a float.
    fn err_on_float(self, op: char) -> Result<(), MathParseErrors> {
        if let Float(f) = self {
            Err(BinaryOpOnFloat(f, op))
        } else {
            Ok(())
        }
    }

    /// Return true if the number is equal to 0.
    fn is_zero(self) -> bool {
        match self {
            Int(i) => i == 0,
            Float(f) => f == 0.0,
        }
    }

    /// Return an error if the given number is 0.
    fn err_on_zero(self) -> Result<(), MathParseErrors> {
        if self.is_zero() {
            Err(UnexpectedZero)
        } else {
            Ok(())
        }
    }

    /// Return true if the number is less than 0.
    fn is_negative(self) -> bool {
        match self {
            Int(i) => i < 0,
            Float(f) => f < 0.0,
        }
    }

    /// Return an error if the given number is negative.
    fn err_on_negative(self) -> Result<(), MathParseErrors> {
        if self.is_negative() {
            Err(UnexpectedNegative)
        } else {
            Ok(())
        }
    }
    fn integer_div(self, other: Self) -> Result<Self, MathParseErrors> {
        other.err_on_zero()?;
        let s = self - (self % other)?;
        let s = match s {
            Int(s) => s,
            Float(s) => f_to_i(s)?,
        };
        let o = match other {
            Int(o) => o,
            Float(o) => f_to_i(o)?,
        };
        Ok(Int(s / o))
    }
}

/* ---------------------------------- Utils --------------------------------- */

/// Takes a string and try to return a number for it.
fn number_from_string(s: &str) -> Result<Number, MathParseErrors> {
    let converted = if s.len() >= 3 && &s[0..2] == "0x" {
        i64::from_str_radix(&s[2..], 16)
    } else {
        i64::from_str_radix(&s, 10)
    };
    if let Ok(num) = converted {
        Ok(Int(num))
    } else {
        if let Ok(num) = s.parse::<f64>() {
            Ok(Float(num))
        } else {
            Err(InvalidNumber(s.to_string()))
        }
    }
}

/// Convert a float to an integer
const INTEGRAL_LIMIT: f64 = 9007199254740992.0;
fn f_to_i(f: f64) -> Result<i64, MathParseErrors> {
    if f.is_nan() {
        return Err(IntConversion(f));
    }
    let f = f.round();

    if f > INTEGRAL_LIMIT {
        Err(IntConversion(f))
    } else if f < -1.0 * INTEGRAL_LIMIT {
        Err(IntConversion(f))
    } else {
        Ok(f as i64)
    }
}

/// Convert an integer to a float
fn i_to_f(i: i64) -> f64 {
    i as f64
}

/* --------------------------------- Testing -------------------------------- */

#[test]
fn test_reading_numbers() {
    assert_eq!(number_from_string("100"),  Ok(Int(100)));
    assert_eq!(number_from_string("0"),    Ok(Int(0)));
    assert_eq!(number_from_string("0x10"), Ok(Int(0x10)));
    assert_eq!(number_from_string("2.5"),  Ok(Float(2.5)));
    assert_eq!(number_from_string("toto"), Err(InvalidNumber("toto".to_string())));
}

#[test]
fn test_read_named_variables() {
    let variables = HashMap::from([
        ("direct_1".to_string(), "1.0".to_string()),
        ("indirect_3".to_string(), "2".to_string()),
        ("indirect_2".to_string(), "indirect_3".to_string()),
        ("indirect_1".to_string(), "indirect_2".to_string()),
    ]);
    assert_eq!(read_name("3",          Some(&variables)), Ok(Int(3)));
    assert_eq!(read_name("direct_1",   Some(&variables)), Ok(Float(1.0)));
    assert_eq!(read_name("indirect_1", Some(&variables)), Err(InvalidNumber("indirect_2".to_string())));
    assert_eq!(read_name("direct_1",   None),             Err(InvalidNumber("direct_1".to_string())));
}

#[test]
fn test_math_compute() {
    let rpn_actions = [RPN::Name("4"), RPN::Name("5"), RPN::Name("3"), Subtraction, Multiplication];
    let computation = math_solve(&rpn_actions, None).unwrap();
    if let Int(computation) = computation {
        assert_eq!(computation, (3-5)*4);
    } else {
        panic!("Expected int.");
    }
}

#[test]
fn test_errors() {
    assert_eq!(Int(10) / Int(0), Err(UnexpectedZero));
    assert_eq!(Int(10) >> Int(-1), Err(UnexpectedNegative));
    assert_eq!(!Float(1.3), Err(BinaryOpOnFloat(1.3, '!')));
    assert_eq!(Float(-5.5).is_negative(), true);
    assert_eq!(Float(5.5).is_negative(), false);

    let big_float = (INTEGRAL_LIMIT as f64) * 5.0;
    assert_eq!(Float(big_float).integer_div(Int(10)), Err(IntConversion(big_float)));
    assert_eq!(Float(-1.0 * big_float).integer_div(Int(10)), Err(IntConversion(big_float * -1.0)));
    match Float(f64::NAN).integer_div(Int(10)) {
        Err(IntConversion(x)) => {
            assert_eq!(x.is_nan(), true);
        },
        x => {
            panic!("Didn't expected {x:?}");
        },
    }
}
