use std::collections::HashMap;
use crate::MathParseErrors;
use crate::MathParseErrors::*;

/* ---------------------------------- Maths --------------------------------- */

const MATH_CHARS: [char; 6] = ['+', '-', '*', '/', '(', ')'];

#[derive(Debug, PartialEq)]
enum MathValue<'a> {
    // Values used in parsing
    Name(&'a str), 
    Operator(char),

    // Values used in solving
    Value(i64),  // TODO: float?
    Operation(char, isize, isize),
    ParenOpen(isize), // Value stored is the offset between the parenthesis' index and the index of what is inside
    ParenClose(usize),
}
use MathValue::*;

/// Tokenise a line of math expression into a vector of `MathValue`.
fn math_token<'a>(s: &'a str) -> Vec<MathValue<'a>> {
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
        math_parse(part1)?;
        math_parse(part2)?;
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
            1 => Ok(()),
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

    println!("Parse {:?}", line);
    paren_parse(line)?;
    println!("after paren {:?}", line);

    //TODO: unary operators
    parse_op(line, &['+', '-'])?;
    parse_op(line, &['/', '*'])?;

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
                let num: i64 = math_compute(&new_name, Some(map))?;
                let _ = std::mem::replace(&mut line[i], Value(num));
            }
        }
    }
    Ok(())
}

/// Reads a line of math that contains only values, operations, and parenthesis
/// and returns a computed result.
fn math_final_compute(line: &[MathValue]) -> Result<i64, MathParseErrors> {

    fn math_compute_index(line: &[MathValue], index: usize) -> Result<i64, MathParseErrors> {
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
                    x => Err(MathParseInternalBug(format!("{x} is not a valid operator."))),
                }
            },
            x => Err(MathParseInternalBug(format!("{x:?} should not have been handled by math_compute_index. It should have been replaced earlier."))),
        }
    }

    math_compute_index(line, 0)
}

pub fn math_compute(s: &str, map: Option<&HashMap<String, String>>) -> Result<i64, MathParseErrors> {
    let mut tokens = math_token(s);
    math_parse(&mut tokens)?;
    read_named_variables(&mut tokens, map)?;
    read_numbers(&mut tokens)?;
    Ok(math_final_compute(&tokens)?)
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
fn number_from_string(s: &str) -> Result<i64, MathParseErrors> {
    let converted = if s.len() > 3 && &s[0..2] == "0x" {
        i64::from_str_radix(&s[2..], 16)
    } else {
        i64::from_str_radix(&s, 10)
    };
    if let Ok(num) = converted {
        Ok(num)
    } else {
        Err(InvalidNumber(s.to_string()))
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

/* --------------------------------- Testing -------------------------------- */

#[test]
fn test_math_token() {
    let math_line = "+4/88*toto";
    assert_eq!(math_token(math_line), vec![Operator('+'), Name("4"), Operator('/'), Name("88"), Operator('*'), Name("toto")]);
}

#[test]
fn test_math_parse() {
    let math_line = "88+89";
    let mut tokens = math_token(math_line);
    math_parse(&mut tokens).unwrap();
    assert_eq!(tokens, vec![Operation('+', 1, 2), Name("88"), Name("89")]);

    let math_line = "1*2+3*4";
    let mut tokens = math_token(math_line);
    math_parse(&mut tokens).unwrap();
    assert_eq!(tokens, vec![Operation('+', 3, 4), Name("1"), Name("2"), Operation('*', -2, -1), Operation('*', 1, 2), Name("3"), Name("4")]);

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
               ParenClose(3)]);

    assert_eq!(math_parse(&mut math_token("33)")), Err(UnopenedParenthesis));
    assert_eq!(math_parse(&mut math_token("((33)")), Err(UnclosedParenthesis));
    assert_eq!(math_parse(&mut math_token("")), Err(EmptyLine));
}

#[test]
fn test_reading_numbers() {
    let math_line = "100*0x10-2";
    let mut tokens = math_token(math_line);
    math_parse(&mut tokens).unwrap();
    read_numbers(&mut tokens).unwrap();
    assert_eq!(tokens, vec![Operation('-', 3, 4), Value(100), Value(0x10), Operation('*', -2, -1), Value(2)]);

    let math_line = "toto-0x10";
    let mut tokens = math_token(math_line);
    math_parse(&mut tokens).unwrap();
    assert_eq!(read_numbers(&mut tokens), Err(InvalidNumber("toto".to_string())));
}

#[test]
fn test_read_named_variables() {
    let variables = HashMap::from([
        ("direct_1".to_string(), "1".to_string()),
        ("indirect_3".to_string(), "2".to_string()),
        ("indirect_2".to_string(), "indirect_3".to_string()),
        ("indirect_1".to_string(), "indirect_2".to_string()),
    ]);
    let mut tokens = vec![Name("3"), Name("indirect_1"), Name("direct_1")];
    read_named_variables(&mut tokens, Some(&variables)).unwrap();
    assert_eq!(tokens, vec![Name("3"), Value(2), Value(1)]);
}

#[test]
fn test_math_final_compute() {
    let mut tokens = math_token("(3-5)*4");
    math_parse(&mut tokens).unwrap();
    read_numbers(&mut tokens).unwrap();
    let computation = math_final_compute(&tokens).unwrap();
    assert_eq!(computation, (3-5)*4);
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
    
    let compute = |input: &str, output: i64| {
        let res = math_compute(input, Some(&variables)).unwrap();
        assert_eq!(res, output);
    };

    compute("((3+3)*b+8)/(a-1)", ((3+3)*b+8)/(a-1));
    compute("0", 0);
    compute("a+b-c", a+b-c);
}

