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
pub fn math_token<'a>(s: &'a str) -> Vec<MathValue<'a>> {

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
    ret
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

#[cfg(test)]
use crate::name_p;

#[test]
fn test_math_token() {
    let math_line = "+4/88*toto";
    assert_eq!(math_token(math_line), vec![Operator('+'), name_p("4"), Operator('/'), name_p("88"), Operator('*'), name_p("toto"), TrailingError]);
}

