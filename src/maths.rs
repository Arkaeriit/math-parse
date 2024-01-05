use std::collections::HashMap;
use crate::MathParseErrors;
use crate::MathParseErrors::*;
use crate::utils::*;
use crate::parse::*;
use crate::parse::MathValue::*;

/* ---------------------------------- Maths --------------------------------- */


/// Take a line of MathValue and replace each value found with a number.
fn read_numbers(line: &mut [MathValue]) -> Result<(), MathParseErrors> {
    for i in 0..line.len() {
        if let Name(name) = &line[i] {
            line[i] = Value(number_from_string(name)?);
        }
    }
    Ok(())
}

/// Reads all Names from a line of math and transform any name being a key in
/// the map to it's value.
/// If map is None, nothing is done.
fn read_named_variables(line: &mut [MathValue], map: Option<&HashMap<String, String>>) -> Result<(), MathParseErrors> {
    let map = if let Some(m) = map {
        m
    } else {
        return Ok(());
    };

    for i in 0..line.len() {
        if let Name(name) = &line[i] {
            if let Some(new_name) = map.get(&name.to_string()) {
                let num = math_compute(&new_name, None)?;
                let _ = std::mem::replace(&mut line[i], Value(num));
            }
        }
    }
    Ok(())
}

/// Reads a line of math that contains only values, operations, and parenthesis
/// and returns a computed result.
fn math_final_compute(line: &[MathValue]) -> Result<Number, MathParseErrors> {

    /// Performs the computation and error checking needed to solve
    /// binary operations.
    fn compute_operation(op: char, value_1: Number, value_2: Number) -> Result<Number, MathParseErrors> {
        match op {
            '*' | '×' | '·'       => Ok(value_1 * value_2),
            '/' | '∕' | '⁄' | '÷' => Ok((value_1 / value_2)?),
            '+'                   => Ok(value_1 + value_2),
            '-' | '−'             => Ok(value_1 - value_2),
            '%'                   => Ok((value_1 % value_2)?),
            '⟌'                   => Ok(value_1.integer_div(value_2)?),
            '|'                   => Ok((value_1 | value_2)?),
            '&'                   => Ok((value_1 & value_2)?),
            '^'                   => Ok((value_1 ^ value_2)?),
            '≪'                   => Ok((value_1 << value_2)?),
            '≫'                   => Ok((value_1 >> value_2)?),
            '<'                   => Err(BadOperatorHint('<', "<<")),
            '>'                   => Err(BadOperatorHint('>', ">>")),
            x                     => Err(MathParseInternalBug(format!("{x} is not a valid operator."))),
        }
    }

    /// Performs the computation and error checking needed to solve
    /// unary operations.
    fn compute_unary(op: char, value: Number) -> Result<Number, MathParseErrors> {
        match op {
            '+' => Ok(value),
            '-' => Ok(Int(-1) * value),
            '!' => Ok((!value)?),
            x => Err(MathParseInternalBug(format!("{x} is not a valid unary operator."))),
        }
    }


    /// This function does most of the work in the computation. The line is a
    /// tree, but we can't do a naive recursive traversal as we want to process
    /// input of any size without stack overflow. We thus have to do an
    /// iterative approach but we use home managed stacks to keep the same logic
    /// as a recursive approach.
    fn math_compute_base(line: &[MathValue]) -> Result<Number, MathParseErrors> {
        let mut operation_stack = Vec::<ProcessedValues>::new();
        let mut value_stack = Vec::<Number>::new();
        let mut current_index = 0;

        for _ in 0..line.len() {
            match &line[current_index] {
                ParenOpen(offset) => {current_index = add_index_offset(current_index, *offset)?;},
                UnaryOperation(op, offset) => {
                    operation_stack.push(Unary(*op));
                    current_index = add_index_offset(current_index, *offset)?;
                },
                Operation(op, offset_1, offset_2) => {
                    operation_stack.push(BinaryLeft(*op, add_index_offset(current_index, *offset_1)?));
                    current_index = add_index_offset(current_index, *offset_2)?;
                },
                Value(number) => {
                    current_index = processe_stacks_of_numbers(*number, &mut value_stack, &mut operation_stack)?;
                    if current_index == !0 {
                        return if let Some(number) = value_stack.pop() {
                            Ok(number)
                        } else {
                            Err(MathParseInternalBug("If the stack reading function indicated the end of stacks, it should have pushed a number of the value stack.".to_string()))
                        };
                    }
                }
                TrailingError => {return Err(TrailingOperator);},
                x => {return Err(MathParseInternalBug(format!("{x:?} should not have been handled by math_compute_base. It should have been replaced earlier.")));},
            }
        }
        Err(MathParseInternalBug("Should not have left the compute_index loop".to_string()))
    }

    /// One if the stacks used is to keep track of operations encountered. We
    /// store the operation from the input line of the operation and the index
    /// of one argument for the branching in binary operations.
    enum ProcessedValues {
        Unary(char),
        BinaryLeft(char, usize),
        BinaryRight(char),
    }
    use ProcessedValues::*;

    /// Once we hit a number, we want to go back in our stacks and apply the
    /// operations until we empty the stacks or we go to a binary operator that
    /// need branching.
    fn processe_stacks_of_numbers(num: Number, value_stack: &mut Vec<Number>, operation_stack: &mut Vec<ProcessedValues>) -> Result<usize, MathParseErrors> {
        let mut number = num;

        loop { // Not infinite as we know that the stacks are not infinite

            let processed_index = if let Some(processed_index) = operation_stack.pop() {
                processed_index
            } else {
                value_stack.push(number);
                return Ok(!0); // This special value is used to indicate that there is no computations left to do
            };

            match processed_index {
                Unary(op) => {
                    number = compute_unary(op, number)?;
                },
                BinaryRight(op) => {
                    let other_number = if let Some(other_number) = value_stack.pop() {
                        other_number
                    } else {
                        return Err(MathParseInternalBug("In a binary right, there should have been a value placed on the stack.".to_string()));
                    };
                    number = compute_operation(op, number, other_number)?;
                },
                BinaryLeft(op, line_offset) => {
                    operation_stack.push(BinaryRight(op));
                    value_stack.push(number);
                    return Ok(line_offset);
                },
            }
        }
    }

    math_compute_base(line)
}

/// Does all the computation from a string with a line of math to the final
/// resulting number.
pub fn math_compute(s: &str, map: Option<&HashMap<String, String>>) -> Result<Number, MathParseErrors> {
    let mut tokens = math_token(s)?;
    math_parse(&mut tokens)?;
    read_named_variables(&mut tokens, map)?;
    read_numbers(&mut tokens)?;
    Ok(math_final_compute(&tokens)?)
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
    let converted = if s.len() > 3 && &s[0..2] == "0x" {
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

/// Takes an index and an offset and return the resulting index
fn add_index_offset(index: usize, offset: isize) -> Result<usize, MathParseErrors> {
    let index_i = u_to_i(index)?;
    i_to_u(index_i + offset)
}

/// Convert a float to an integer
fn f_to_i(f: f64) -> Result<i64, MathParseErrors> {
    const INTEGRAL_LIMIT: f64 = 9007199254740992.0;
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
    let math_line = "100*0x10-2.5";
    let mut tokens = math_token(math_line).unwrap();
    math_parse(&mut tokens).unwrap();
    read_numbers(&mut tokens).unwrap();
    assert_eq!(tokens, vec![Operation('-', 3, 4), Value(Int(100)), Value(Int(0x10)), Operation('*', -2, -1), Value(Float(2.5)), TrailingError]);

    let math_line = "toto-0x10";
    let mut tokens = math_token(math_line).unwrap();
    math_parse(&mut tokens).unwrap();
    assert_eq!(read_numbers(&mut tokens), Err(InvalidNumber("toto".to_string())));
}

#[test]
fn test_read_named_variables() {
    let variables = HashMap::from([
        ("direct_1".to_string(), "1.0".to_string()),
        ("indirect_3".to_string(), "2".to_string()),
        ("indirect_2".to_string(), "indirect_3".to_string()),
        ("indirect_1".to_string(), "indirect_2".to_string()),
    ]);
    let mut tokens = vec![Name("3"), Name("direct_1")];
    read_named_variables(&mut tokens, Some(&variables)).unwrap();
    assert_eq!(tokens, vec![Name("3"), Value(Float(1.0))]);
    let mut tokens = vec![Name("3"), Name("indirect_1"), Name("direct_1")];
    assert_eq!(read_named_variables(&mut tokens, Some(&variables)), Err(InvalidNumber("indirect_2".to_string())));
}

#[test]
fn test_math_final_compute() {
    let mut tokens = math_token("(3-5)*4").unwrap();
    math_parse(&mut tokens).unwrap();
    read_numbers(&mut tokens).unwrap();
    let computation = math_final_compute(&tokens).unwrap();
    if let Int(computation) = computation {
        assert_eq!(computation, (3-5)*4);
    } else {
        panic!("Expected int.");
    }

    let mut tokens = math_token("3++").unwrap();
    math_parse(&mut tokens).unwrap();
    read_numbers(&mut tokens).unwrap();
    let computation = math_final_compute(&tokens);
    assert_eq!(computation, Err(TrailingOperator));
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
        let res = math_compute(input, Some(&variables)).unwrap();
        if let Int(res) = res {
            assert_eq!(res, output);
        } else {
            panic!("Expected integer instead of float.");
        }
    };

    fn compute_float (input: &str, output: f64) {
        let res = math_compute(input, None).unwrap();
        if let Float(res) = res {
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

