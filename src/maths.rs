use std::collections::HashMap;
use crate::MathParseErrors;
use crate::MathParseErrors::*;

/* ---------------------------------- Maths --------------------------------- */

const MATH_CHARS: [char; 23] = ['+', '-', '−', '*', '×', '·', '/', '∕', '⁄', '÷', '(', ')', '%', '⟌', '!', '~', '^', '&', '|', '≪', '<', '>', '≫'];

#[derive(Debug, PartialEq)]
enum MathValue<'a> {
    // Values used in parsing
    /// A slice of the input string. As only a single string is used, the single
    /// lifetime for the reference is well suited.
    Name(&'a str), 
    /// A character from the MATH_CHAR list.
    Operator(char),

    // Values used in solving
    /// A solved value.
    Value(Number),
    /// A math operation. The character is the operator used and the two `isize`
    /// are the offsets from the Operation to its two members.
    Operation(char, isize, isize),
    /// An unary operator. The character is the operator used and the `isize` is
    /// the offset from the `UnaryOperation` to the value the operator is used
    /// on.
    UnaryOperation(char, isize),
    /// The start of a group in parenthesis. The `isize` if the offset between
    /// the `ParenOpen` and the start of the inside of the parenthesis.
    ParenOpen(isize),
    /// The end of a group in parenthesis. The `usize` is the number of elements
    /// inside of the group needed to fly back to the beginning if the
    /// parenthesis group during parsing.
    ParenClose(usize),

    // Special values
    /// Placed at the last element of a line, used as a canary to catch
    /// misbehaving unary operators.
    TrailingError,
    /// Used to remove a math value that will need to be garbage collected.
    NoMath,
}
use MathValue::*;

/// Tokenise a line of math expression into a vector of `MathValue`.
fn math_token<'a>(s: &'a str) -> Result<Vec<MathValue<'a>>, MathParseErrors> {

    /// Reads name and operators in a line of math.
    fn token_base<'a>(s: &'a str) -> Vec<MathValue<'a>> {
        let mut ret = Vec::<MathValue>::new();
        let mut new_name_index = !0; // Word that we are writing, !0 indicate we were not writing anything.
        let mut current_index = 0;

        for c in s.chars() {
            if is_in(c, &MATH_CHARS) {
                if new_name_index != !0 { // We were writing a work
                    ret.push(Name(&s[new_name_index..current_index]));
                    new_name_index = !0;
                }
                ret.push(Operator(c));
            } else if new_name_index == !0 {
                new_name_index = current_index;
            }
            current_index += c.len_utf8();
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
                (Some('<'), Some('<')) => {
                    line[i-1] = Operator('≪');
                    line[i] = NoMath;
                },
                (Some('>'), Some('>')) => {
                    line[i-1] = Operator('≫');
                    line[i] = NoMath;
                },
                (_, _) => {},
            }
        }
    }

    /// Removes all the NoMath elements from a slice.
    /// Returns a slice which only contains useful elements.
    fn token_garbage_collect(line: &mut Vec<MathValue>) -> Result<(), MathParseErrors> {
        let mut tmp = Vec::<MathValue>::new();
        for _ in 0..line.len() {
            if let Some(from) = line.pop() {
                if from != NoMath {
                    tmp.push(from);
                }
            } else {
                return Err(MathParseInternalBug("Should not happen as we do it as much as there is elements in line.".to_string()));
            }
        }
        // tmp is reversed so we want to reverse it back
        for _ in 0..tmp.len() {
            if let Some(to) = tmp.pop() {
                line.push(to);
            } else {
                return Err(MathParseInternalBug("Should not happen as we do it as much as there is elements in tmp.".to_string()));
            }
        }
        Ok(())
    }

    let mut ret = token_base(s);
    token_complex(&mut ret);
    token_garbage_collect(&mut ret)?;
    Ok(ret)
}

/// Parse a line of `MathValue` and make it into a tree of operations.
/// The root of the tree will be kept as the first element of the vector.
// Here are some example of rearrangement:
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
                Operator('-') | Operator('−') => {
                    if previous_operator {
                        let _ = std::mem::replace(&mut line[i], UnaryOperation('-', 1));
                    }
                    previous_operator = true;
                },
                Operator('!') | Operator('~') => {
                    if previous_operator {
                        let _ = std::mem::replace(&mut line[i], UnaryOperation('!', 1));
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
        parse_op(line, &['|'])?;
        parse_op(line, &['^'])?;
        parse_op(line, &['&'])?;
        parse_op(line, &['≪', '≫'])?;
        parse_op(line, &['+', '-', '−'])?;
        parse_op(line, &['/', '∕', '⁄', '÷', '*', '×', '·', '%', '⟌'])?;
        Ok(())
    }

    unary_parse(line)?;
    paren_parse(line)?;
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

    fn math_compute_index(line: &[MathValue], index: usize) -> Result<Number, MathParseErrors> {
        match &line[index] {
            Value(number) => Ok(*number),
            ParenOpen(offset) => {
                let target = add_index_offset(index, *offset)?;
                math_compute_index(line, target)
            },
            Operation(op, offset_1, offset_2) => {
                let target = add_index_offset(index, *offset_1)?;
                let value_1 = math_compute_index(line, target)?;
                let target = add_index_offset(index, *offset_2)?;
                let value_2 = math_compute_index(line, target)?;
                compute_operation(*op, value_1, value_2)
            },
            UnaryOperation(op, offset) => {
                let target = add_index_offset(index, *offset)?;
                let value = math_compute_index(line, target)?;
                compute_unary(*op, value)
            },
            TrailingError => Err(TrailingOperator),
            x => Err(MathParseInternalBug(format!("{x:?} should not have been handled by math_compute_index. It should have been replaced earlier."))),
        }
    }

    math_compute_index(line, 0)
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

/// Return true if the given string contains any math char
pub fn contains_math_char(s: &str) -> bool {
    s.contains(MATH_CHARS)
}

/* --------------------------------- Testing -------------------------------- */

#[test]
fn test_math_token() {
    let math_line = "+4/88*toto";
    assert_eq!(math_token(math_line).unwrap(), vec![Operator('+'), Name("4"), Operator('/'), Name("88"), Operator('*'), Name("toto"), TrailingError]);
}

#[test]
fn test_math_parse() {
    let math_line = "+88+89";
    let mut tokens = math_token(math_line).unwrap();
    math_parse(&mut tokens).unwrap();
    assert_eq!(tokens, vec![Operation('+', 2, 3), Name("88"), UnaryOperation('+', -1), Name("89"), TrailingError]);

    let math_line = "-1*2+-3*4";
    let mut tokens = math_token(math_line).unwrap();
    math_parse(&mut tokens).unwrap();
    assert_eq!(tokens, vec![Operation('+', 4, 5), Name("1"), UnaryOperation('-', -1), Name("2"), Operation('*', -2, -1), Operation('*', 2, 3), Name("3"), UnaryOperation('-', -1), Name("4"), TrailingError]);

    let math_line = "(1+2)*(3+4)";
    let mut tokens = math_token(math_line).unwrap();
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

    assert_eq!(math_parse(&mut math_token("33)").unwrap()), Err(UnopenedParenthesis));
    assert_eq!(math_parse(&mut math_token("((33)").unwrap()), Err(UnclosedParenthesis));
    assert_eq!(math_parse(&mut math_token("").unwrap()), Err(EmptyLine));
    assert_eq!(math_parse(&mut math_token("22+()").unwrap()), Err(EmptyLine));
    assert_eq!(math_parse(&mut math_token("33+*23").unwrap()), Err(MisplacedOperator('*')));
    assert_eq!(math_parse(&mut math_token("*2").unwrap()), Err(MisplacedOperator('*')));
    assert_eq!(math_parse(&mut math_token("2/").unwrap()), Err(EmptyLine));
}

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

