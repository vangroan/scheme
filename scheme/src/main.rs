use std::io::{self, Write};

use scheme_engine;

fn main() {
    run_repl()
}

fn run_repl() {
    let mut buf = String::new();
    let stdin = io::stdin();
    let mut count = 0;

    loop {
        count += 1;
        buf.clear();
        print!("{count} > ");
        let _ = io::stdout().flush();
        stdin.read_line(&mut buf).expect("read stdin");

        match scheme_engine::parse(buf.as_str()) {
            Ok(expr) => {
                println!("{:?}", expr);
            }
            Err(err) => {
                eprintln!("error: {err}");
            }
        }
    }
}
