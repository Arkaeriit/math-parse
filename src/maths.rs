use std::collections::HashMap;
use crate::MathParseErrors;
use crate::MathParseErrors::*;

/* ---------------------------------- Maths --------------------------------- */

const MATH_CHARS: [char; 8] = ['+', '-', '*', '/', '(', ')', '%', '⟌'];

#[derive(Debug, PartialEq)]
enum MathValue<'a> {
    TrailingError, // Placed at the last element of a line, used as a canary to catch misbehaving unary operators
    NoMath, // Used to remove a math value that will need to be garbage collected.

    // Values used in parsing
    Name(&'a str), 
    Operator(char),

    // Values used in solving
    Value(Number),
    Operation(char, isize, isize),
    UnaryOperation(char, isize),
    ParenOpen(isize), // Value stored is the offset between the parenthesis' index and the index of what is inside
    ParenClose(usize),
}
use MathValue::*;

/// Tokenise a line of math expression into a vector of `MathValue`.
fn math_token<'a>(s: &'a str) -> Vec<MathValue<'a>> {

    /// Reads name and operators in a line of math.
    fn token_base<'a>(s: &'a str) -> Vec<MathValue<'a>> {
        let mut ret = Vec::<MathValue>::new();
        let mut new_name_index = !0; // Word that we are writing, !0 indicate we were not writing anything.
        let mut current_index = 0;

        for c in s.chars() {
            println!("c = {}, ci = {}, nni = {}", c, current_index, new_name_index);
            if is_in(c, &MATH_CHARS) {
                if new_name_index != !0 { // We were writing a work
                    ret.push(Name(&s[new_name_index..current_index]));
                    new_name_index = !0;
                }
                ret.push(Operator(c));
            } else if new_name_index == !0 {
                new_name_index = current_index;
            }
            current_index += 1;
        }

        if new_name_index != !0 { // We were writing a work
            ret.push(Name(&s[new_name_index..]));
        }
        ret.push(TrailingError);
        ret
    }

    /// Combine complex math symbols such as // to make operators
    fn token_complex(line: &mut [MathValue]) {
        for i in 1..line.len() {
            let previous_op = if let Operator(c) = line[i-1] {
                Some(c)
            } else {
                None
            };
            let current_op = if let Operator(c) = line[i] {
                Some(c)
            } else {
                None
            };
            match (previous_op, current_op) {
                (Some('/'), Some('/')) => {
                    line[i-1] = Operator('⟌');
                    line[i] = NoMath;
                },
                (_, _) => {},
            }

        }
    }

    /// Removes all the NoMath elements from a slice.
    /// Returns a slice which only contains useful elements.
    fn token_garbage_collect(line: &mut Vec<MathValue>) {
        let mut tmp = Vec::<MathValue>::new();
        for _ in 0..line.len() {
            let from = line.pop().expect("Should not happen as we do it as much as there is elements in tmp.");
            if from != NoMath {
                tmp.push(from);
            }
        }
        // tmp is reversed so we want to reverse it back
        for _ in 0..tmp.len() {
            line.push(tmp.pop().expect("Should not happen as we do it as much as there is elements in tmp."));
        }
    }

    let mut ret = token_base(s);
    token_complex(&mut ret);
    token_garbage_collect(&mut ret);
    ret
}

/// Parse a line of `MathValue` and make it into a tree of operations.
/// The root of the tree will be kept as the first element of the vector.
// Here are some example of rearangements:
//                               
// | 0 | + | 1 |                 
//                               
//       |                       
//       v                       
//                               
//   /---v                       
// | + | 0 | 1 |                 
//   ·-------^                   
//                               
//                               
//                               
// | 2 | * | 4 | + | 3 |         
//                               
//           |                   
//           v                   
//                               
//   /--v /--v                   
// | + | * | 2 | 4 | 3 |         
//   |   ·-------^   ^           
//   ·---------------/           
//                               
// | ( | 2 | + | 4 | ) | * | 3 | 
//                               
//               |               
//               v               
//   /-------------------·       
//   |   /---v           v       
// | * | + | 2 | 4 | ) | ( | 3 | 
//   |  ^ ·------^   |   |   |   
//   |  ·---------------./   |   
//   ·-----------------------/   
//                               
fn math_parse(line: &mut [MathValue]) -> Result<(), MathParseErrors> {

    /// Parse unary operators this must be done before any other steps of the
    /// parsing as the next steps will move around the elements used to
    /// determine which operators are unary.
    fn unary_parse(line: &mut [MathValue]) -> Result<(), MathParseErrors> {
        let mut previous_operator = true;
        for i in 0..line.len() {
            match &line[i] {
                Operator('+') => {
                    if previous_operator {
                        let _ = std::mem::replace(&mut line[i], UnaryOperation('+', 1));
                    }
                    previous_operator = true;
                },
                Operator('-') => {
                    if previous_operator {
                        let _ = std::mem::replace(&mut line[i], UnaryOperation('-', 1));
                    }
                    previous_operator = true;
                },
                Operator('(') => {
                    previous_operator = true;
                },
                Operator(')') => {
                    previous_operator = false;
                },
                Operator(x) => {
                    if previous_operator {
                        return Err(MisplacedOperator(*x));
                    }
                    previous_operator = true;
                },
                Name(_) => {
                    previous_operator = false;
                },
                TrailingError => {},
                x => {
                    return Err(MathParseInternalBug(format!("{x:?} should not have been present in unary_parse.")));
                },
            }
        }
        Ok(())
    }

    /// Transform content in parenthesis into a root element.
    fn paren_parse(line: &mut [MathValue]) -> Result<(), MathParseErrors> {
        let mut paren_open_index = 0;
        let mut paren_depth = 0;
        for i in 0..line.len() {
            if paren_depth == 0 {
                if let Operator('(') = line[i] {
                    paren_open_index = i;
                    paren_depth = 1;
                } else if let Operator(')') = line[i] {
                    return Err(UnopenedParenthesis);
                }
            } else {
                if let Operator('(') = line[i] {
                    paren_depth += 1;
                } else if let Operator(')') = line[i] {
                    paren_depth -= 1;
                    if paren_depth == 0 { // We finally closed the parenthesis
                        let size_between_paren = i - paren_open_index - 1;
                        let (before_used, used_slice_and_end) = line.split_at_mut(paren_open_index+1);
                        let (used_slice, after_used) = used_slice_and_end.split_at_mut(size_between_paren);
                        math_parse(used_slice)?;
                        let open  = ParenOpen(1);
                        let close = ParenClose(size_between_paren);
                        let _ = std::mem::replace(&mut before_used[paren_open_index], open);
                        let _ = std::mem::replace(&mut after_used[0], close);
                    }
                }
            }
        }

        if paren_depth != 0 {
            Err(UnclosedParenthesis)
        } else {
            Ok(())
        }
    }

    /// Convert two slices and a symbol into a `MathOp`
    /// Return an user error if needed.
    fn make_op(line: &mut [MathValue], operator_index: usize) -> Result<(), MathParseErrors> {
        let (part1, part2_and_op) = line.split_at_mut(operator_index);
        let operator_offset = u_to_i(operator_index)?;
        let (op, part2) = part2_and_op.split_at_mut(1);
        all_but_paren_parse(part1)?;
        all_but_paren_parse(part2)?;
        let op = if let Operator(c) = op[0] {
            c
        } else {
            return Err(MathParseInternalBug(format!("{:?} should not have been used in make_op.", op[0])));
        };
        let operation = Operation(op, operator_offset, operator_offset+1);
        let part_1_header = std::mem::replace(&mut line[0], operation);
        let part_1_header = match part_1_header {
            ParenOpen(inside_offset) => ParenOpen(inside_offset - operator_offset),
            Operation(c, offset_1, offset_2) => Operation(c, offset_1 - operator_offset, offset_2 - operator_offset),
            UnaryOperation(c, offset) => UnaryOperation(c, offset - operator_offset),
            x => x,
        };
        let _ = std::mem::replace(&mut line[operator_index], part_1_header);
        Ok(())
    }

    /// Parse a line of math from left to right, if any operator from the list
    /// if found, makes a `MathOp` out of it.
    /// Handles the special cases of 1 or 2 elements in the line.
    fn parse_op(line: &mut [MathValue], ops: &[char]) -> Result<(), MathParseErrors> {
        match line.len() {
            0 => Err(EmptyLine),
            1 => match line[0] {
                TrailingError => Err(EmptyLine),
                _ => Ok(()),
            },
            _ => {
                let mut index = line.len() - 2;
                while index >= 1 {
                    match line[index] {
                        Operator(c) => {
                            for op in ops {
                                if c == *op {
                                    make_op(line, index)?;
                                    return Ok(());
                                }
                            }
                            index -= 1;
                        },
                        ParenClose(size) => {
                            index -= size;
                        },
                        _ => {
                            index -= 1;
                        }
                    }
                }
                Ok(())
            },
        }
    }

    /// Parse everything except for parenthesis, which are already parsed
    /// recursively, and unary, which are parsed in a single pass.
    fn all_but_paren_parse(line: &mut [MathValue]) -> Result<(), MathParseErrors> {
        parse_op(line, &['+', '-'])?;
        parse_op(line, &['/', '*', '%', '⟌'])?;
        Ok(())
    }

    println!("Parse {:?}", line);
    unary_parse(line)?;
    paren_parse(line)?;
    println!("after paren {:?}", line);
    all_but_paren_parse(line)?;


    Ok(())
}

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
/// the map to it's value. Keeps trying with the result and then, try to make
/// it into a number.
/// If map is None, nothing is done.
fn read_named_variables(line: &mut [MathValue], map: Option<&HashMap<String, String>>) -> Result<(), MathParseErrors> {
    let map = if let Some(m) = map {
        m
    } else {
        return Ok(());
    };

    for i in 0..line.len() {
        if let Name(name) = &line[i] {
            if let Some(mut new_name) = map.get(&name.to_string()) {
                while let Some(newer_name) = map.get(new_name) {
                    new_name = newer_name;
                }
                let num = math_compute(&new_name, Some(map))?;
                let _ = std::mem::replace(&mut line[i], Value(num));
            }
        }
    }
    Ok(())
}

/// Reads a line of math that contains only values, operations, and parenthesis
/// and returns a computed result.
fn math_final_compute(line: &[MathValue]) -> Result<Number, MathParseErrors> {

    fn math_compute_index(line: &[MathValue], index: usize) -> Result<Number, MathParseErrors> {
        match &line[index] {
            Value(number) => Ok(*number),
            ParenOpen(offset) => {
                let target = add_index_offset(index, *offset)?;
                Ok(math_compute_index(line, target)?)
            },
            Operation(op, offset_1, offset_2) => {
                let target = add_index_offset(index, *offset_1)?;
                let value_1 = math_compute_index(line, target)?;
                let target = add_index_offset(index, *offset_2)?;
                let value_2 = math_compute_index(line, target)?;
                match op {
                    '*' => Ok(value_1 * value_2),
                    '/' => Ok(value_1 / value_2),
                    '+' => Ok(value_1 + value_2),
                    '-' => Ok(value_1 - value_2),
                    '%' => Ok(value_1 % value_2),
                    '⟌' => Ok(value_1.integer_div(value_2)?),
                    x => Err(MathParseInternalBug(format!("{x} is not a valid operator."))),
                }
            },
            UnaryOperation(op, offset) => {
                let target = add_index_offset(index, *offset)?;
                let value = math_compute_index(line, target)?;
                match op {
                    '+' => Ok(value),
                    '-' => Ok(Int(-1) * value),
                    x => Err(MathParseInternalBug(format!("{x} is not a valid unary operator."))),
                }
            },
            TrailingError => Err(TrailingOperator),
            x => Err(MathParseInternalBug(format!("{x:?} should not have been handled by math_compute_index. It should have been replaced earlier."))),
        }
    }

    math_compute_index(line, 0)
}

pub fn math_compute(s: &str, map: Option<&HashMap<String, String>>) -> Result<Number, MathParseErrors> {
    let mut tokens = math_token(s);
    math_parse(&mut tokens)?;
    read_named_variables(&mut tokens, map)?;
    read_numbers(&mut tokens)?;
    Ok(math_final_compute(&tokens)?)
}

/* --------------------------------- Numbers -------------------------------- */

use std::ops::*;

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
    type Output = Self;
    
    fn div(self, other: Self) -> Self {
        match (self, other) {
            (Int(s),   Int(o))   => Float(i_to_f(s) / i_to_f(o)),
            (Float(s), Int(o))   => Float(s / i_to_f(o)),
            (Int(s),   Float(o)) => Float(i_to_f(s) / o),
            (Float(s), Float(o)) => Float(s / o),
        }
    }
}

impl Rem for Number {
    type Output = Self;
    
    fn rem(self, other: Self) -> Self {
        match (self, other) {
            (Int(s),   Int(o))   => Int(s % o),
            (Float(s), Int(o))   => Float(s % i_to_f(o)),
            (Int(s),   Float(o)) => Float(i_to_f(s) % o),
            (Float(s), Float(o)) => Float(s % o),
        }
    }
}

impl Number {
    fn integer_div(self, other: Self) -> Result<Self, MathParseErrors> {
        let s = match self {
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

/// Return true if the element is in the slice
fn is_in<T: Eq>(a: T, set: &[T]) -> bool {
    for elem in set {
        if a == *elem {
            return true;
        }
    }
    false
}

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

/// Takes a isize that should be positive and makes it a usize
fn i_to_u(i: isize) -> Result<usize, MathParseErrors> {
    if let Ok(u) = TryInto::<usize>::try_into(i) {
        Ok(u)
    } else {
        Err(MathParseInternalBug(format!("{i} should have been positive.")))
    }
}

/// Takes a usize and try to make it into a isize
fn u_to_i(u: usize) -> Result<isize, MathParseErrors> {
    if let Ok(i) = TryInto::<isize>::try_into(u) {
        Ok(i)
    } else {
        return Err(MathParseInternalBug(format!("{u} should be made as isize.")));
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
fn test_math_token() {
    let math_line = "+4/88*toto";
    assert_eq!(math_token(math_line), vec![Operator('+'), Name("4"), Operator('/'), Name("88"), Operator('*'), Name("toto"), TrailingError]);
}

#[test]
fn test_math_parse() {
    let math_line = "+88+89";
    let mut tokens = math_token(math_line);
    math_parse(&mut tokens).unwrap();
    assert_eq!(tokens, vec![Operation('+', 2, 3), Name("88"), UnaryOperation('+', -1), Name("89"), TrailingError]);

    let math_line = "-1*2+-3*4";
    let mut tokens = math_token(math_line);
    math_parse(&mut tokens).unwrap();
    assert_eq!(tokens, vec![Operation('+', 4, 5), Name("1"), UnaryOperation('-', -1), Name("2"), Operation('*', -2, -1), Operation('*', 2, 3), Name("3"), UnaryOperation('-', -1), Name("4"), TrailingError]);

    let math_line = "(1+2)*(3+4)";
    let mut tokens = math_token(math_line);
    math_parse(&mut tokens).unwrap();
    assert_eq!(tokens, vec![
               Operation('*', 5, 6),
               Operation('+', 1, 2),
               Name("1"),
               Name("2"),
               ParenClose(3),
               ParenOpen(-4),
               ParenOpen(1),
               Operation('+', 1, 2),
               Name("3"),
               Name("4"),
               ParenClose(3),
               TrailingError]);

    assert_eq!(math_parse(&mut math_token("33)")), Err(UnopenedParenthesis));
    assert_eq!(math_parse(&mut math_token("((33)")), Err(UnclosedParenthesis));
    assert_eq!(math_parse(&mut math_token("")), Err(EmptyLine));
    assert_eq!(math_parse(&mut math_token("22+()")), Err(EmptyLine));
    assert_eq!(math_parse(&mut math_token("33+*23")), Err(MisplacedOperator('*')));
    assert_eq!(math_parse(&mut math_token("*2")), Err(MisplacedOperator('*')));
    assert_eq!(math_parse(&mut math_token("2/")), Err(EmptyLine));
}

#[test]
fn test_reading_numbers() {
    let math_line = "100*0x10-2.5";
    let mut tokens = math_token(math_line);
    math_parse(&mut tokens).unwrap();
    read_numbers(&mut tokens).unwrap();
    assert_eq!(tokens, vec![Operation('-', 3, 4), Value(Int(100)), Value(Int(0x10)), Operation('*', -2, -1), Value(Float(2.5)), TrailingError]);

    let math_line = "toto-0x10";
    let mut tokens = math_token(math_line);
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
    let mut tokens = vec![Name("3"), Name("indirect_1"), Name("direct_1")];
    read_named_variables(&mut tokens, Some(&variables)).unwrap();
    assert_eq!(tokens, vec![Name("3"), Value(Int(2)), Value(Float(1.0))]);
}

#[test]
fn test_math_final_compute() {
    let mut tokens = math_token("(3-5)*4");
    math_parse(&mut tokens).unwrap();
    read_numbers(&mut tokens).unwrap();
    let computation = math_final_compute(&tokens).unwrap();
    if let Int(computation) = computation {
        assert_eq!(computation, (3-5)*4);
    } else {
        panic!("Expected int.");
    }

    let mut tokens = math_token("3++");
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
    
    compute_int("((3+3)*b+8)*(a-1)", ((3+3)*b+8)*(a-1));
    compute_int("0", 0);
    compute_int("-a+b-c", -a+b-c);
    compute_int("---+++-a", ----a);
    compute_int("3%8+99", 3%8+99);
    compute_int("10.0//3.0", 10/3);

    compute_float("4*9/4", 4.0*9.0/4.0);
    compute_float("4*9/4.0", 4.0*9.0/4.0);
    compute_float("4.0*9/4", 4.0*9.0/4.0);
    compute_float("4.0*9.0/4", 4.0*9.0/4.0);
    compute_float("4*9.0/4", 4.0*9.0/4.0);
    compute_float("4*9.0/4.0", 4.0*9.0/4.0);
    compute_float("4.0+9-4", 4.0+9.0-4.0);
    compute_float("4+9-4.0", 4.0+9.0-4.0);
    compute_float("4.0+9-4", 4.0+9.0-4.0);
    compute_float("4.0+9.0-4", 4.0+9.0-4.0);
    compute_float("4+9.0-4", 4.0+9.0-4.0);
    compute_float("4+9.0-4.0", 4.0+9.0-4.0);

}

