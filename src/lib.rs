const MATH_CHARS: [char; 6] = ['+', '-', '*', '/', '(', ')'];

enum MathValue {
    Value(usize), // TODO: float?
    SubExpression(Box<MathOp>),
}

enum MathOp {
    Add(MathValue, MathValue),
    Sub(MathValue, MathValue),
    Mult(MathValue, MathValue),
    Div(MathValue, MathValue),
}

/// Represent the kind of text that makes up a math expression.
#[derive(Debug, PartialEq)]
enum MathText {
    Name(String), // With a bit of work, I could use &str or the like, but as math constants are probably not going to be used very often, performance is not a great concern.
    Op(char),
}
use MathText::*;

/// Tokenise a line of math expression into a vector of `MathText`.
fn math_token(s: &str) -> Vec<MathText> {
    let mut ret = Vec::<MathText>::new();
    let mut new_name = String::new(); // Word that we are writing

    for c in s.chars() {
        if is_in(c, &MATH_CHARS) {
            if new_name.as_str() != "" { // We were writing a work
                ret.push(Name(new_name.clone()));
                new_name.clear();
            }
            ret.push(Op(c));
        } else {
            new_name.push(c);
        }
    }

    if new_name.as_str() != "" { // We were writing a work
        ret.push(Name(new_name.clone()));
    }
    ret
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
    assert_eq!(math_token(math_line), vec![Op('+'), Name("4".to_string()), Op('/'), Name("88".to_string()), Op('*'), Name("toto".to_string())]);
}


