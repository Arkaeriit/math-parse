use crate::MathParseErrors;
use crate::MathParseErrors::*;
use crate::BinaryOp;
use crate::UnaryOp;
use crate::RPN;
use crate::RPN::*;


/// Pop one number from the stack.
pub fn pop_one<T>(number_stack: &mut Vec<T>) -> Result<T, MathParseErrors> {
    if let Some(num) = number_stack.pop() {
        Ok(num)
    } else {
        Err(UnbalancedStack)
    }
}

/// Pop two numbers from the stack.
pub fn pop_two<T>(number_stack: &mut Vec<T>) -> Result<(T, T), MathParseErrors> {
    let num_1 = if let Some(n) = number_stack.pop() {
        n
    } else {
        return Err(UnbalancedStack);
    };
    let num_2 = if let Some(n) = number_stack.pop() {
        n
    } else {
        return Err(UnbalancedStack);
    };
    Ok((num_2, num_1))
}

type UnaryExecFn<T> = dyn Fn(T, UnaryOp) -> Result<T, MathParseErrors>;
/// Execute the given unary operation on the top element of the stack.
pub fn execute_unary<T>(number_stack: &mut Vec<T>, op: UnaryOp, exec: &UnaryExecFn<T>) -> Result<(), MathParseErrors> {
    let num = pop_one(number_stack)?;
    let computed = exec(num, op)?;
    number_stack.push(computed);
    Ok(())
}

type BinaryExecFn<T> = dyn Fn(T, T, BinaryOp) -> Result<T, MathParseErrors>;
/// Execute the given binary operation on the top two elements of the stack.
pub fn execute_binary<T>(number_stack: &mut Vec<T>, op: BinaryOp, exec: &BinaryExecFn<T>) -> Result<(), MathParseErrors> {
    let (num_1, num_2) = pop_two(number_stack)?;
    let computed = exec(num_1, num_2, op)?;
    number_stack.push(computed);
    Ok(())
}

type NameExecFc<'a, T> = dyn Fn(&str) -> Result<T, MathParseErrors> + 'a;
/// Execute a single RPN action and update the stack of numbers accordingly.
fn exec_rpn_one_action<T>(number_stack: &mut Vec<T>, action: &RPN,
   compute_name: &NameExecFc<T>, compute_unary: &UnaryExecFn<T>, compute_binary: &BinaryExecFn<T>) -> Result<(), MathParseErrors> {
    match action {
        Name(x) => {
            number_stack.push(compute_name(&x)?);
            Ok(())
        },
        Unary(op) => execute_unary(number_stack, *op, compute_unary),
        Binary(op) => execute_binary(number_stack, *op, compute_binary),
    }
}

/// Execute all RPN actions and return the single element left in the stack.
pub fn exec_rpn<T>(rpn_actions: &[RPN], compute_name: &NameExecFc<T>, compute_unary: &UnaryExecFn<T>, compute_binary: &BinaryExecFn<T>) -> Result<T, MathParseErrors> {
    let mut number_stack = Vec::<T>::new();

    for action in rpn_actions {
        exec_rpn_one_action(&mut number_stack, action, compute_name, compute_unary, compute_binary)?;
    }

    if number_stack.len() == 1 {
        pop_one(&mut number_stack)
    } else {
        Err(UnbalancedStack)
    }
}

/* --------------------------------- Testing -------------------------------- */

#[test]
fn test_pop() {
    let mut vec = vec!['a', 'b', 'c', 'd', 'e'];
    assert_eq!(pop_two(&mut vec), Ok(('d', 'e')));
    assert_eq!(pop_two(&mut vec), Ok(('b', 'c')));
    assert_eq!(pop_two(&mut vec), Err(UnbalancedStack));
    vec.push('a');
    assert_eq!(pop_one(&mut vec), Ok('a'));
    assert_eq!(pop_two(&mut vec), Err(UnbalancedStack));
    assert_eq!(pop_one(&mut vec), Err(UnbalancedStack));
}

