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

fn compute_binary(num_1: Tree, num_2: Tree, op: BinaryOp) -> Result<Tree, MathParseErrors> {
    let boxed_1 = Box::new(num_1);
    let boxed_2 = Box::new(num_2);
    Ok(Binary(op, boxed_1, boxed_2))
}

/// Reads a line of math that contains only values, operations, and parenthesis
/// and returns a computed result.
pub fn parse_to_tree(rpn_actions: &[RPN]) -> Result<Tree, MathParseErrors> {
    let mut compute_name = | name: &str | -> Result<Tree, MathParseErrors> {
        Ok(Name(name.to_string()))
    };

    exec_rpn(rpn_actions, &mut Box::new(compute_name), &compute_unary, &compute_binary)
}

/* --------------------------------- Testing -------------------------------- */

#[test]
fn test_parse_to_tree() {
    use crate::BinaryOp::*;
    use crate::UnaryOp::*;
    use crate::name_t;
    use crate::name_r;

    let rpn = [name_r("toto"), name_r("tata"), RPN::Unary(Plus), name_r("titi"), RPN::Binary(Subtraction), RPN::Binary(Multiplication)];
    assert_eq!(parse_to_tree(&rpn).unwrap(),
        Binary(Multiplication,
            Box::new(name_t("toto")),
            Box::new(Binary(Subtraction,
                    Box::new(Unary(Plus,
                            Box::new(name_t("tata")))),
                    Box::new(name_t("titi"))))));
}

