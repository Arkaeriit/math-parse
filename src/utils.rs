use crate::MathParseErrors;
use crate::MathParseErrors::*;

/// Takes a isize that should be positive and makes it a usize
pub fn i_to_u(i: isize) -> Result<usize, MathParseErrors> {
    if let Ok(u) = TryInto::<usize>::try_into(i) {
        Ok(u)
    } else {
        Err(MathParseInternalBug(format!("{i} should have been positive.")))
    }
}

/// Takes a usize and try to make it into a isize
pub fn u_to_i(u: usize) -> Result<isize, MathParseErrors> {
    if let Ok(i) = TryInto::<isize>::try_into(u) {
        Ok(i)
    } else {
        return Err(MathParseInternalBug(format!("{u} should be made as isize.")));
    }
}

