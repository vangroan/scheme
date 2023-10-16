use std::io::{self, Write};

use scheme_engine;

fn main() {
    run_repl()
}

fn run_repl() {
    let mut buf = String::new();
    let stdin = io::stdin();
    let mut count = 0;

    // Console environment.
    let env = scheme_engine::new_env().expect("failed creating new core environment");

    loop {
        count += 1;
        buf.clear();
        print!("{count} > ");
        let _ = io::stdout().flush();
        stdin.read_line(&mut buf).expect("read stdin");

        match scheme_engine::parse(buf.as_str()) {
            Ok(expr) => {
                println!("parse:\n\t{:#?}", expr);

                match scheme_engine::compile(env.clone(), &expr) {
                    Ok(closure) => {
                        println!("bytecode:");
                        for (index, op) in
                            closure.borrow().procedure().bytecode().iter().enumerate()
                        {
                            println!("  {index:>6} : {op:?}");
                        }

                        // Run closure in VM
                        match scheme_engine::eval(closure) {
                            Ok(value) => {
                                println!("{:?}", value);
                            }
                            Err(err) => {
                                eprintln!("error: {err}");
                            }
                        }
                    }
                    Err(err) => {
                        eprintln!("error: {err}");
                    }
                }
            }
            Err(err) => {
                eprintln!("error: {err}");
            }
        }
    }
}
