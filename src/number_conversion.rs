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

/// Convert a float to an integer
pub const INTEGRAL_LIMIT: f64 = 9007199254740992.0;
pub fn f_to_i(f: f64) -> Result<i64, MathParseErrors> {
    if f.is_nan() {
        return Err(IntConversion(f));
    }
    let f = f.round();

    if f > INTEGRAL_LIMIT {
        Err(IntConversion(f))
    } else if f < -1.0 * INTEGRAL_LIMIT {
        Err(IntConversion(f))
    } else {
        Ok(f as i64)
    }
}

/// Convert a float to an integer only if it stays the same.
pub fn f_to_i_strict(f: f64) -> Result<i64, MathParseErrors> {
    let convert_loose = f_to_i(f)?;
    let convert_back = i_to_f(convert_loose);
    if convert_back == f {
        Ok(convert_loose)
    } else {
        Err(ReturnFloatExpectedInt(f))
    }
}

/// Convert an integer to a float
pub fn i_to_f(i: i64) -> f64 {
    i as f64
}

