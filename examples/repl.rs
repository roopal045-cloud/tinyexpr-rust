use std::io::{self, BufRead, Write};
use tinyexpr::Context;

fn main() {
    let ctx = Context::new();
    let stdin = io::stdin();
    print!("> ");
    io::stdout().flush().ok();
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        if line.is_empty() {
            print!("> ");
            io::stdout().flush().ok();
            continue;
        }
        match tinyexpr::compile(&line, &ctx) {
            Ok(expr) => println!("{}", expr.eval()),
            Err(e) => {
                println!("{}^", " ".repeat(e.position));
                println!("Error near here");
            }
        }
        print!("> ");
        io::stdout().flush().ok();
    }
}
