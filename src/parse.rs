use crate::MathParseErrors;
use crate::MathParseErrors::*;
use crate::utils::*;

const MATH_CHARS: [char; 23] = ['+', '-', '−', '*', '×', '·', '/', '∕', '⁄', '÷', '(', ')', '%', '⟌', '!', '~', '^', '&', '|', '≪', '<', '>', '≫'];

#[derive(Debug, PartialEq)]
pub enum MathValue<'a> {
    // Values used in parsing
    /// A slice of the input string. As only a single string is used, the single
    /// lifetime for the reference is well suited.
    Name(&'a str), 
    /// A character from the MATH_CHAR list.
    Operator(char),

    // Values used in solving
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
                    line[i] = ParenOpen(1); // Here and for the wollowing tokens, the ParenOpen(1) is used as a pointer to the next token, acting as if one of the two tokens of the complex is removed.
                },
                (Some('<'), Some('<')) => {
                    line[i-1] = Operator('≪');
                    line[i] = ParenOpen(1);
                },
                (Some('>'), Some('>')) => {
                    line[i-1] = Operator('≫');
                    line[i] = ParenOpen(1);
                },
                (_, _) => {},
            }
        }
    }

    let mut ret = token_base(s);
    token_complex(&mut ret);
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
fn math_parse_tokens(line: &mut [MathValue]) -> Result<(), MathParseErrors> {

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
                ParenOpen(_) => {
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

        /// Parse the first group in parenthesis that will be closed first.
        /// Return true if a parenthesis group was parsed and false otherwise.
        fn single_paren_parse(line: &mut [MathValue]) -> Result<bool, MathParseErrors> {
            // TODO: Maybe not n²...
            let mut i = 0;
            let mut maybe_paren_open_index = None;
            while i < line.len() {
                if let Operator('(') = line[i] {
                    maybe_paren_open_index = Some(i);
                } else if let Operator(')') = line[i] {
                    let paren_open_index = if let Some(index) = maybe_paren_open_index {
                        index
                    } else {
                        return Err(UnopenedParenthesis);
                    };
                    let size_between_paren = i - paren_open_index - 1;
                    let (before_used, used_slice_and_end) = line.split_at_mut(paren_open_index+1);
                    let (used_slice, after_used) = used_slice_and_end.split_at_mut(size_between_paren);
                    all_but_paren_parse(used_slice)?;
                    let open  = ParenOpen(1);
                    let close = ParenClose(size_between_paren);
                    let _ = std::mem::replace(&mut before_used[paren_open_index], open);
                    let _ = std::mem::replace(&mut after_used[0], close);
                    return Ok(true);
                }
                i += 1;
            }
            if let Some(_) = maybe_paren_open_index {
                Err(UnclosedParenthesis)
            } else {
                Ok(false)
            }
        }

        while single_paren_parse(line)? {}
        Ok(())
    }

    /// Parse everything that is not parenthesis and unary operators.
    fn all_but_paren_parse(line: &mut [MathValue]) -> Result<(), MathParseErrors> {

        /// Use to represent a slice of the line we are working on. Needed as
        /// we will cover multiple slices at the same time.
        #[derive(Clone, Debug)]
        struct IndexRange {
            from: usize,
            to: usize,
        }

        /// Represent each steps we can encounter while parsing the line.
        enum ParseSteps {
            /// We want to solve a range. In it we will find an operator and
            /// split each sides in `BlockSolving` and the middle in
            /// `OperatorReading`.
            BlockSolving(IndexRange),

            /// An operator, with it we only need to move around the operator so
            /// that it ends up on the front.
            OperatorReading{range: IndexRange, index: usize},
        } use ParseSteps::*;

        /// Processes all the tasks in the given stack until it's empty.
        fn solve_tasks(line: &mut [MathValue], tasks_stack: &mut Vec<ParseSteps>) -> Result<(), MathParseErrors> {
            while tasks_stack.len() != 0 {
                match tasks_stack.pop() {
                    Some(OperatorReading{range, index}) => {
                        make_op(line, &range, index)?;
                    },
                    Some(BlockSolving(range)) => {
                        solve_block(line, &range, tasks_stack)?;
                    },
                    None => {
                        return Err(MathParseInternalBug(format!("Error, the stack is empty in solve_tasks.")));
                    },
                }
            }
            Ok(())
        }

        /// To solve a bloc, search for all operators and process them.
        fn solve_block(line: &mut [MathValue], range: &IndexRange, tasks_stack: &mut Vec<ParseSteps>) -> Result<(), MathParseErrors> {
            if parse_op(line, &['|'], range, tasks_stack)? { return Ok(()); }
            if parse_op(line, &['^'], range, tasks_stack)? { return Ok(()); }
            if parse_op(line, &['&'], range, tasks_stack)? { return Ok(()); }
            if parse_op(line, &['≪', '≫', '<', '>'], range, tasks_stack)? { return Ok(()); }
            if parse_op(line, &['+', '-', '−'], range, tasks_stack)? { return Ok(()); }
            parse_op(line, &['/', '∕', '⁄', '÷', '*', '×', '·', '%', '⟌'], range, tasks_stack)?;
            Ok(())
        }

        /// Parse a line of math from left to right, if any operator from the
        /// list if found, calls `make_tasks_from_op` on it.
        /// Handles the special cases of 1 or 2 elements in the line.
        fn parse_op(line: &mut [MathValue], ops: &[char], range: &IndexRange, tasks_stack: &mut Vec<ParseSteps>) -> Result<bool, MathParseErrors> {
            match range.to - range.from {
                0 => Err(EmptyLine),
                1 => match line[range.from] {
                    TrailingError => Err(EmptyLine),
                    _ => Ok(true),
                },
                _ => {
                    let mut index = range.to - 2;
                    while index >= range.from+1 {
                        match line[index] {
                            Operator(c) => {
                                for op in ops {
                                    if c == *op {
                                        make_tasks_from_op(range, index, tasks_stack);
                                        return Ok(true);
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
                    Ok(false)
                },
            }
        }

        /// When an operation index, cut the two parts of the equation that need
        /// solving and add them to the task stack. Also add the operation
        /// management to it.
        fn make_tasks_from_op(range: &IndexRange, operator_index: usize, tasks_stack: &mut Vec<ParseSteps>) {
            let part1 = IndexRange{
                from: range.from,
                to: operator_index,
            };
            let part2 = IndexRange {
                from: operator_index+1,
                to: range.to,
            };
            tasks_stack.push(OperatorReading{range: range.clone(), index: operator_index});
            tasks_stack.push(BlockSolving(part1));
            tasks_stack.push(BlockSolving(part2));
        }

        /// To solve a `OperatorReading` task, replace the operator into an
        /// operation and put it at the front of the range.
        fn make_op(line: &mut [MathValue], range: &IndexRange, operator_index: usize) -> Result<(), MathParseErrors> {
            let op = if let Operator(c) = line[operator_index] {
                c
            } else {
                return Err(MathParseInternalBug(format!("{:?} should not have been used in make_op.", line[operator_index])));
            };
            let operator_offset = u_to_i(operator_index - range.from)?;
            let operation = Operation(op, operator_offset, operator_offset+1);
            let part_1_header = std::mem::replace(&mut line[range.from], operation);
            let part_1_header = match part_1_header {
                ParenOpen(inside_offset) => ParenOpen(inside_offset - operator_offset),
                Operation(c, offset_1, offset_2) => Operation(c, offset_1 - operator_offset, offset_2 - operator_offset),
                UnaryOperation(c, offset) => UnaryOperation(c, offset - operator_offset),
                x => x,
            };
            let _ = std::mem::replace(&mut line[operator_index], part_1_header);
            Ok(())
        }

        let mut tasks_stack = vec![BlockSolving(IndexRange{from:0, to:line.len()})];
        solve_tasks(line, &mut tasks_stack)
    }

    unary_parse(line)?;
    paren_parse(line)?;
    all_but_paren_parse(line)?;
    Ok(())
}

/// Tokenize and then parse a math expression.
pub fn math_parse<'a>(expression: &'a str) -> Result<Vec<MathValue<'a>>, MathParseErrors> {
    let mut tokens = math_token(expression)?;
    math_parse_tokens(&mut tokens)?;
    Ok(tokens)
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
    math_parse_tokens(&mut tokens).unwrap();
    assert_eq!(tokens, vec![Operation('+', 2, 3), Name("88"), UnaryOperation('+', -1), Name("89"), TrailingError]);

    let math_line = "-1*2+-3*4";
    let mut tokens = math_token(math_line).unwrap();
    math_parse_tokens(&mut tokens).unwrap();
    assert_eq!(tokens, vec![Operation('+', 4, 5), Name("1"), UnaryOperation('-', -1), Name("2"), Operation('*', -2, -1), Operation('*', 2, 3), Name("3"), UnaryOperation('-', -1), Name("4"), TrailingError]);

    let math_line = "(1+2)*(3+4)";
    let mut tokens = math_token(math_line).unwrap();
    math_parse_tokens(&mut tokens).unwrap();
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

    assert_eq!(math_parse_tokens(&mut math_token("33)").unwrap()), Err(UnopenedParenthesis));
    assert_eq!(math_parse_tokens(&mut math_token("((33)").unwrap()), Err(UnclosedParenthesis));
    assert_eq!(math_parse_tokens(&mut math_token("").unwrap()), Err(EmptyLine));
    assert_eq!(math_parse_tokens(&mut math_token("22+()").unwrap()), Err(EmptyLine));
    assert_eq!(math_parse_tokens(&mut math_token("33+*23").unwrap()), Err(MisplacedOperator('*')));
    assert_eq!(math_parse_tokens(&mut math_token("*2").unwrap()), Err(MisplacedOperator('*')));
    assert_eq!(math_parse_tokens(&mut math_token("2/").unwrap()), Err(EmptyLine));
}


