use crate::Tree;
use crate::Tree::*;
use crate::RPN;
use crate::MathParseErrors;
use crate::MathParseErrors::*;
use crate::utils::*;
use crate::BinaryOp;
use crate::UnaryOp;

/// Pop one number from the stack.
fn pop_one<'a>(number_stack: &mut Vec<Tree<'a>>) -> Result<Tree<'a>, MathParseErrors> {
    if let Some(num) = number_stack.pop() {
        Ok(num)
    } else {
        Err(MathParseInternalBug(format!("Error, unable to pop one, no element on the stack.")))
    }
}

/// Pop two numbers from the stack.
fn pop_two<'a>(number_stack: &mut Vec<Tree<'a>>) -> Result<(Tree<'a>, Tree<'a>), MathParseErrors> {
    let num_1 = if let Some(n) = number_stack.pop() {
        n
    } else {
        return Err(MathParseInternalBug(format!("Error, unable to pop two, no element on the stack.")));
    };
    let num_2 = if let Some(n) = number_stack.pop() {
        n
    } else {
        return Err(MathParseInternalBug(format!("Error, unable to pop two, only one element on the stack.")));
    };
    Ok((num_2, num_1))
}

/// Execute the given unary operation on the top element of the stack.
fn compute_unary(number_stack: &mut Vec<Tree>, op: UnaryOp) -> Result<(), MathParseErrors> {
    let num = pop_one(number_stack)?;
    let boxed = Box::new(num);
    let computed = Unary(op, boxed);
    number_stack.push(computed);
    Ok(())
}

/// Execute the given binary operation on the top two elements of the stack.
fn compute_binary(number_stack: &mut Vec<Tree>, op: BinaryOp) -> Result<(), MathParseErrors> {
    let (num_1, num_2) = pop_two(number_stack)?;
    let boxed_1 = Box::new(num_1);
    let boxed_2 = Box::new(num_2);
    let computed = Binary(op, boxed_1, boxed_2);
    number_stack.push(computed);
    Ok(())
}

/// Execute a single RPN action and update the stack of numbers accordingly.
fn exec_rpn_action<'a>(number_stack: &mut Vec<Tree<'a>>, action: &RPN<'a>) -> Result<(), MathParseErrors> {
    match action {
        RPN::Name(x) => {
            number_stack.push(Name(x));
            Ok(())
        },
        RPN::Unary(op) => compute_unary(number_stack, *op),
        RPN::Binary(op) => compute_binary(number_stack, *op),
    }
}

/// Reads a line of math that contains only values, operations, and parenthesis
/// and returns a computed result.
pub fn parse_to_tree<'a>(rpn_actions: &[RPN<'a>]) -> Result<Tree<'a>, MathParseErrors> {

    let mut number_stack = Vec::<Tree>::new();

    for action in rpn_actions {
        exec_rpn_action(&mut number_stack, action)?;
    }

    pop_one(&mut number_stack)
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

