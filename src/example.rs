extern crate math_parse;

/* ---------------------------------- main ---------------------------------- */

fn main() {
    let args = concat_cli_arg();

    match math_parse::math_solve_int(&args, None) {
        Ok(i) => {
            println!("{i}");
            std::process::exit(0);
        },
        Err(math_parse::MathParseErrors::ReturnFloatExpectedInt(_)) => {
            // We will proceed to the next match statement that handles floats.
        },
        Err(x) => {
            println!("{x}");
            std::process::exit(1);
        },
    }

    match math_parse::math_solve_float(&args, None) {
        Ok(f) => {
            println!("{f}");
            std::process::exit(0);
        },
        Err(x) => {
            println!("{x}");
            std::process::exit(1);
        },
    }
}

/* ---------------------------- Helper functions ---------------------------- */

/// Reads the command line arguments and concatenate them on a single string.
fn concat_cli_arg() -> String {
    let mut ret = String::new();
    let mut first_arg = true;
    for arg in std::env::args() {
        if first_arg {
            first_arg = false;
        } else {
            ret.push_str(&arg);
        }
    }
    ret
}

