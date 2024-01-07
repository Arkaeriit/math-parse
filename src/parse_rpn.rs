use crate::tokenize::MathValue::*;
use crate::tokenize::math_token;
use crate::tokenize::MathValue;
use crate::MathParseErrors::*;
use crate::MathParseErrors;
use crate::BinaryOp;
use crate::RPN::*;
use crate::RPN;

pub fn parse_rpn(expression: &str) -> Result<Vec<RPN>, MathParseErrors> {
    let tokens = math_token(expression);
    let rpn = rpn_parse_tokens(&tokens)?;
    check_rpn_valid(&rpn)?;
    Ok(rpn)
}

fn rpn_parse_tokens(tokens: &[MathValue]) -> Result<Vec<RPN>, MathParseErrors> {
    let mut ret = Vec::new();
    for token in tokens {
        let parsed_values = match token {
            MathValue::Name(x) => parse_rpn_name(*x),
            Operator(x) => parse_rpn_operator(*x)?,
            TrailingError => vec![],
            ParenOpen(1) => vec![], // This one can be put there when parsing complex tokens
            x => {
                return Err(MathParseInternalBug(format!("{x:?} should not have been on math_parse_rpn.")));
            },
        };
        for value in parsed_values {
            ret.push(value);
        }
    }
    Ok(ret)
}

fn parse_rpn_operator(c: char) -> Result<Vec<RPN>, MathParseErrors> {
    let op = match BinaryOp::from_char(c) {
        Ok(x)                        => Ok(x),
        Err(MathParseInternalBug(_)) => Err(InvalidRPNOperator(c)),
        x                            => x
    };
    match op {
        Ok(x)  => Ok(vec![Binary(x)]),
        Err(x) => Err(x),
    }
}

fn parse_rpn_name(names: &str) -> Vec<RPN> {
    split_words(names).iter().map(|x| RPN::Name(x.clone())).collect::<Vec<RPN>>()
}

fn check_rpn_valid(rpn: &[RPN]) -> Result<(), MathParseErrors> {
    fn map_to_one(_: &str) -> Option<String> {
        Some("1".to_string())
    }

    match crate::solve::math_solve(rpn, &map_to_one) {
        Ok(_) => Ok(()),
        Err(x) => Err(x),
    }
}

/* ---------------------------------- Utils --------------------------------- */

/// From a str, return an iterator to each words.
fn split_words(s: &str) -> Vec<String> {
    let uniform_whitespace = s.replace(&['\t', ' ', '\n', '\r', ' '][..], " ");
    uniform_whitespace.split(" ")
        .filter(|x| *x != "")
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
}

/* --------------------------------- Testing -------------------------------- */

#[cfg(test)]
use crate::name_r;

#[test]
fn test_parse_rpn() {
    use BinaryOp::*;

    assert_eq!(
        parse_rpn("88
            99	+"),
        Ok(vec![name_r("88"), name_r("99"), Binary(Addition)]));

    assert_eq!(
        parse_rpn("88 77 + 33 11 + *"),
        Ok(vec![name_r("88"), name_r("77"), Binary(Addition), name_r("33"), name_r("11"), Binary(Addition), Binary(Multiplication)]));

    assert_eq!(
        parse_rpn("(88 77 +)"),
        Err(InvalidRPNOperator('(')));

    assert_eq!(
        parse_rpn("8 7 + +"),
        Err(UnbalancedStack));

    assert_eq!(
        parse_rpn("+"),
        Err(UnbalancedStack));

    assert_eq!(
        parse_rpn("8 7"),
        Err(UnbalancedStack));

    assert_eq!(
        parse_rpn("6 1 >>"),
        Ok(vec![name_r("6"), name_r("1"), Binary(ShiftRight)]));

    assert_eq!(
        parse_rpn("6 1 >"),
        Err(BadOperatorHint('>', ">>")));
}

