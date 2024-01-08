extern crate math_parse;

/* ---------------------------------- main ---------------------------------- */

fn main() {
    let args = concat_cli_arg();

    let parsed = math_parse::MathParse::parse(&args);
    let solved = match parsed {
        Ok(x)  => x.solve_auto(None),
        Err(x) => Err(x),
    };

    match solved {
        Ok(Ok(i)) => {
            println!("{i}");
            std::process::exit(0);
        },
        Ok(Err(f)) => {
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

