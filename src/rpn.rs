use crate::MathParseErrors;
use crate::MathParseErrors::*;
use crate::utils::*;
use crate::parse::*;
use crate::parse::MathValue::*;
use crate::RPN;
use crate::RPN::*;

enum RPNSteps {
    Solve(usize),
    OperatorStep(usize),
} use RPNSteps::*;

pub fn parse_rpn<'a>(line: &[MathValue<'a>]) -> Result<Vec<RPN<'a>>, MathParseErrors> {
    let mut rpn_steps = vec![Solve(0)];
    let mut ret = Vec::<RPN>::new();
    while rpn_steps.len() != 0 {
        rpn_run_step(line, &mut rpn_steps, &mut ret)?;
    }
    Ok(ret)
}

fn rpn_run_step<'a>(line: &[MathValue<'a>], rpn_steps: &mut Vec<RPNSteps>, rpn_ret: &mut Vec<RPN<'a>>) -> Result<(), MathParseErrors> {
    match rpn_steps.pop() {
        Some(Solve(index)) => rpn_solve(line, rpn_steps, rpn_ret, index),
        Some(OperatorStep(index)) => rpn_operator(line, rpn_ret, index),
        None => Err(MathParseInternalBug(format!("Error, stack should not have been empty in rpn_solve_step."))),
    }
}

fn rpn_solve<'a>(line: &[MathValue<'a>], rpn_steps: &mut Vec<RPNSteps>, rpn_ret: &mut Vec<RPN<'a>>, index: usize) -> Result<(), MathParseErrors> {
    match &line[index] {
        MathValue::Name(name) => {
            rpn_ret.push(RPN::Name(name));
        },
        Operation(_char, offset_1, offset_2) => {
            rpn_steps.push(OperatorStep(index));
            rpn_steps.push(Solve(add_index_offset(index, *offset_1)?));
            rpn_steps.push(Solve(add_index_offset(index, *offset_2)?));
        },
        UnaryOperation(_char, offset) => {
            rpn_steps.push(OperatorStep(index));
            rpn_steps.push(Solve(add_index_offset(index, *offset)?));
        },
        ParenOpen(offset) => {
            rpn_steps.push(Solve(add_index_offset(index, *offset)?));
        },
        x => {
            return Err(MathParseInternalBug(format!("{x:?} should not have been handled by rpn_solve. It should have been replaced earlier.")));
        },
    }
    Ok(())
}

fn rpn_operator<'a>(line: &[MathValue<'a>], rpn_ret: &mut Vec<RPN<'a>>, index: usize) -> Result<(), MathParseErrors> {
    let to_push = read_operator(line, index)?;
    rpn_ret.push(to_push);
    Ok(())
}

fn read_operator<'a>(line: &[MathValue<'a>], index: usize) -> Result<RPN<'a>, MathParseErrors> {
    match &line[index] {
        Operation(x, _offset_1, _offset_2) => match x {
            '*' | '×' | '·'       => Ok(Multiplication),
            '/' | '∕' | '⁄' | '÷' => Ok(Division),
            '+'                   => Ok(Addition),
            '-' | '−'             => Ok(Subtraction),
            '%'                   => Ok(Reminder),
            '⟌'                   => Ok(IntegerDivision),
            '|'                   => Ok(BitwiseOr),
            '&'                   => Ok(BitwiseAnd),
            '^'                   => Ok(BitwiseXor),
            '≪'                   => Ok(ShiftLeft),
            '≫'                   => Ok(ShiftRight),
            '<'                   => Err(BadOperatorHint('<', "<<")),
            '>'                   => Err(BadOperatorHint('>', ">>")),
            x                     => Err(MathParseInternalBug(format!("{x} is not a valid operator."))),
        },
        UnaryOperation(x, _offset) => match x {
            '!' => Ok(UnaryNot),
            '-' => Ok(UnaryMinus),
            '+' => Ok(UnaryPlus),
            x   => Err(MathParseInternalBug(format!("{x} is not a valid unary operator."))),
        },
        x => Err(MathParseInternalBug(format!("{x:?} should not have been handled by rpn_operator."))),
    }
}

/* ---------------------------------- Utils --------------------------------- */

/// Takes an index and an offset and return the resulting index
fn add_index_offset(index: usize, offset: isize) -> Result<usize, MathParseErrors> {
    let index_i = u_to_i(index)?;
    i_to_u(index_i + offset)
}


