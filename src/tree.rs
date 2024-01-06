use crate::Tree;
use crate::Tree::*;
use crate::RPN;
use crate::MathParseErrors;
use crate::MathParseErrors::*;
use crate::utils::*;
use crate::BinaryOp;
use crate::UnaryOp;
use crate::rpn_stack_manipulation::*;

fn compute_unary(num: Tree, op: UnaryOp) -> Result<Tree, MathParseErrors> {
    let boxed = Box::new(num);
    Ok(Unary(op, boxed))
}

fn compute_binary<'a>(num_1: Tree<'a>, num_2: Tree<'a>, op: BinaryOp) -> Result<Tree<'a>, MathParseErrors> {
    let boxed_1 = Box::new(num_1);
    let boxed_2 = Box::new(num_2);
    Ok(Binary(op, boxed_1, boxed_2))
}

/// Reads a line of math that contains only values, operations, and parenthesis
/// and returns a computed result.
pub fn parse_to_tree<'a>(rpn_actions: &[RPN<'a>]) -> Result<Tree<'a>, MathParseErrors> {
    let mut compute_name = | name: &'a str | -> Result<Tree<'a>, MathParseErrors> {
        Ok(Name(name))
    };

    exec_rpn(rpn_actions, &mut Box::new(compute_name), &compute_unary, &compute_binary)
}

/* --------------------------------- Testing -------------------------------- */

#[test]
fn test_parse_to_tree() {
    use crate::BinaryOp::*;
    use crate::UnaryOp::*;

    let rpn = [RPN::Name("toto"), RPN::Name("tata"), RPN::Unary(Plus), RPN::Name("titi"), RPN::Binary(Subtraction), RPN::Binary(Multiplication)];
    assert_eq!(parse_to_tree(&rpn).unwrap(),
        Binary(Multiplication,
            Box::new(Name("toto")),
            Box::new(Binary(Subtraction,
                    Box::new(Unary(Plus,
                            Box::new(Name("tata")))),
                    Box::new(Name("titi"))))));
}

