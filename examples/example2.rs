use tinyexpr::Context;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Usage: example2 \"expression\"");
        return;
    }
    let expression = &args[1];
    println!("Evaluating:\n\t{expression}");

    let mut ctx = Context::new();
    let x = ctx.var("x");
    let y = ctx.var("y");

    match tinyexpr::compile(expression, &ctx) {
        Ok(expr) => {
            x.set(3.0);
            y.set(4.0);
            let r = expr.eval();
            println!("Result:\n\t{r}");
        }
        Err(e) => {
            println!("\t{}^\nError near here", " ".repeat(e.position));
        }
    }
}
