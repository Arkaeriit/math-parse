const MATH_CHARS: [char; 6] = ['+', '-', '*', '/', '(', ')'];

#[derive(Debug, PartialEq)]
enum MathValue {
    // Values used in parsing
    Name(String),  // With a bit of work, I could use &str or the like, but as math constants are probably not going to be used very often, performance is not a great concern.
    Operator(char),
    NoMath,        // Value that have been removed, it should be garbage-collected.

    // Values used in solving
    Value(usize),  // TODO: float?
    Operation(MathOp),
    SubExpression(usize),
}
use MathValue::*;

#[derive(Debug, PartialEq)]
enum MathOp {
    Add(usize, usize),
    Sub(usize, usize),
    Mul(usize, usize),
    Div(usize, usize),
}
use MathOp::*;

/// Tokenise a line of math expression into a vector of `MathValue`.
fn math_token(s: &str) -> Vec<MathValue> {
    let mut ret = Vec::<MathValue>::new();
    let mut new_name = String::new(); // Word that we are writing

    for c in s.chars() {
        if is_in(c, &MATH_CHARS) {
            if new_name.as_str() != "" { // We were writing a work
                ret.push(Name(new_name.clone()));
                new_name.clear();
            }
            ret.push(Operator(c));
        } else {
            new_name.push(c);
        }
    }

    if new_name.as_str() != "" { // We were writing a work
        ret.push(Name(new_name.clone()));
    }
    ret
}



/// Parse a line of `MathValue` and make it into a tree of operations.
/// The root of the tree will be kept as the first element of the vector.
/// If there is an error, return an error message made for the user.
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
//                               
//   /---v   /---v               
// | * | ( | + | 2 | 4 | ) | 3 | 
//   |   |   ·-------^   |   ^   
//   |   ·---------------/   |   
//   ·-----------------------/   
//                               
fn math_parse(line: &mut [MathValue]) -> Result<(), String> {

    /// Transform content in parenthesis into a root element.
    fn paren_parse(line: &mut [MathValue]) -> Result<(), String> {
        let mut paren_open_index = 0;
        let mut paren_depth = 0;
        for i in 0..line.len() {
            if paren_depth == 0 {
                if let Operator('(') = line[i] {
                    paren_open_index = i;
                    paren_depth = 1;
                } else if let Operator(')') = line[i] {
                    return Err(format!("Error, closing a parenthesis with no matching open ones.\n"));
                }
            } else {
                if let Operator('(') = line[i] {
                    paren_depth += 1;
                } else if let Operator(')') = line[i] {
                    paren_depth -= 1;
                    if paren_depth == 0 { // We finally closed the parenthesis
                        let mut used_slice = &mut line[paren_open_index+1..i];
                        math_parse(used_slice);
                        let replacement = SubExpression(i);
                        let _ = std::mem::replace(&mut line[paren_open_index], NoMath);
                        let _ = std::mem::replace(&mut line[i], replacement);
                    }
                }
            }
        }

        if paren_depth != 0 {
            Err("Error, opening parenthesis with no matching closed one.\n".to_string())
        } else {
            Ok(())
        }
    }

    /// Removes all the NoMath eGlements from a slice.
    /// Returns a slice which only contains useful elements.
    fn garbage_collect_math(line: &mut [MathValue]) -> &mut [MathValue] {
        let mut tmp = Vec::<MathValue>::new();
        for i in 0..line.len() {
            //let from = &line[line.len() - 1 - i];
            let from = std::mem::replace(&mut line[line.len() - 1 - i], NoMath);
            if from != NoMath {
                tmp.push(from);
            }
        }
        // tmp is reversed so we want to reverse it back
        println!("tmp={:?}", tmp);
        let mut last_i = 0;
        for i in 0..tmp.len() {
            line[i] = tmp.pop().expect("Should not happen as we do it as much as there is elements in tmp.");
            last_i = i;
        }
        &mut line[0..last_i+1]
    }

    /// Convert two slices and a symbol into a `MathOp`
    /// Return an user error if needed.
    fn make_op(line: &mut [MathValue], operator_index: usize) -> Result<(), String> {
        let (part1, mut part2_and_op) = line.split_at_mut(operator_index);
        let (op, part2) = part2_and_op.split_at_mut(1);
        let val1 = math_parse(part1)?;
        let val2 = math_parse(part2)?;
        let op = if let Operator(c) = op[0] {
            c
        } else {
            panic!("Wrong use of make_op!");
        };
        let operation = match op {
            '+' => Add(operator_index, operator_index+1),
            '-' => Sub(operator_index, operator_index+1),
            '*' => Mul(operator_index, operator_index+1),
            '/' => Div(operator_index, operator_index+1),
            _ => {return Err(format!("Error, {} is not a valid math operator.", op))},
        };
        let part_1_header = std::mem::replace(&mut line[0], Operation(operation));
        let _ = std::mem::replace(&mut line[operator_index], part_1_header);
        Ok(())
    }

    /// Parse a line of math from left to right, if any operator from the list
    /// if found, makes a `MathOp` out of it.
    /// Handles the special cases of 1 or 2 elements in the line.
    /// Returns None if none of the operators are found.
    fn parse_op(line: &mut [MathValue], ops: &[char]) -> Result<(), String> {
        match line.len() {
            0 => Err(format!("Error, empty line of math.\n")),
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
                        SubExpression(size) => {
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
    let cleaned = garbage_collect_math(line);
    println!("cleaned {:?}", cleaned);

    //TODO: unary operators
    parse_op(cleaned, &['+', '-'])?;
    parse_op(cleaned, &['/', '*'])?;

    Ok(())
}



/* ---------------------------------- Utils --------------------------------- */

/// Return true if a string got any chars from the `MATH_CHARS`.
fn contains_math_chars(s: &str) -> bool {
    s.contains(MATH_CHARS)
}

/// Return true if the element is in the slice
fn is_in<T: Eq>(a: T, set: &[T]) -> bool {
    for elem in set {
        if a == *elem {
            return true;
        }
    }
    false
}

/* --------------------------------- Testing -------------------------------- */

#[test]
fn test_math_token() {
    let math_line = "+4/88*toto";
    assert_eq!(math_token(math_line), vec![Operator('+'), Name("4".to_string()), Operator('/'), Name("88".to_string()), Operator('*'), Name("toto".to_string())]);
}

#[test]
fn test_math_parse() {
    /*
    let math_line = "88+89";
    let mut tokens = math_token(math_line);
    math_parse(&mut tokens).unwrap();
    assert_eq!(tokens[0], MV(SubExpression(Box::new(Add(TmpName("88".to_string()), TmpName("89".to_string()))))));
    let math_line = "1*2+3*4";
    let mut tokens = math_token(math_line);
    math_parse(&mut tokens).unwrap();
    assert_eq!(tokens[0], MV(SubExpression(Box::new(Add(
                    SubExpression(Box::new(Mul(TmpName("1".to_string()), TmpName("2".to_string())))),
                    SubExpression(Box::new(Mul(TmpName("3".to_string()), TmpName("4".to_string()))))

                    )))));
    let math_line = "(1+2)*(3+4)";
    let mut tokens = math_token(math_line);
    let parsed = math_parse(&mut tokens).unwrap();
    assert_eq!(parsed, MV(SubExpression(Box::new(Mul(
                    SubExpression(Box::new(Add(TmpName("1".to_string()), TmpName("2".to_string())))),
                    SubExpression(Box::new(Add(TmpName("3".to_string()), TmpName("4".to_string()))))

                    )))));
                    */
}

