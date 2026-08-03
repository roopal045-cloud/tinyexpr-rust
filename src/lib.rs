use std::cell::Cell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;
pub type VarSlot = Rc<Cell<f64>>;
pub struct FuncDef {
    arity: usize,
    pure: bool,
    call: Box<dyn Fn(&[f64]) -> f64>,
}
impl FuncDef {
    pub fn native0(pure: bool, f: fn() -> f64) -> Self {
        FuncDef {
            arity: 0,
            pure,
         call: Box::new(move |_args| f()),
        }
    }
    pub fn native1(pure: bool, f: fn(f64) -> f64) -> Self {
        FuncDef {
            arity: 1,
            pure,
            call: Box::new(move |args| f(args[0])),
        }
    }
   pub fn native2(pure: bool, f: fn(f64, f64) -> f64) -> Self {
        FuncDef {
            arity: 2,
            pure,
            call: Box::new(move |args| f(args[0], args[1])),
        }
    }
pub fn closure(arity: usize, pure: bool, f: impl Fn(&[f64]) -> f64 + 'static) -> Self {
        FuncDef {
            arity,
            pure,
            call: Box::new(f),
        }
    }
    fn call(&self, args: &[f64]) -> f64 {
        (self.call)(args)
    }
}
pub struct Context {
    variables: HashMap<String, VarSlot>,
    functions: HashMap<String, Rc<FuncDef>>,
}
impl Context {
    pub fn new() -> Self {
        Self::with_builtins(false)
    }
    pub fn with_natural_log() -> Self {
        Self::with_builtins(true)
    }

    fn with_builtins(nat_log: bool) -> Self {
        let mut functions = HashMap::new();
        for (name, def) in builtin_table(nat_log) {
            functions.insert(name.to_string(), def);
        }
        Context {
            variables: HashMap::new(),
            functions,
        }
    }
    pub fn var(&mut self, name: &str) -> VarSlot {
        let slot = Rc::new(Cell::new(0.0));
        self.variables.insert(name.to_string(), slot.clone());
        slot
    }
    pub fn bind_var(&mut self, name: &str, slot: VarSlot) {
        self.variables.insert(name.to_string(), slot);
    }
    pub fn func0(&mut self, name: &str, f: fn() -> f64) {
        self.functions
            .insert(name.to_string(), Rc::new(FuncDef::native0(true, f)));
    }
    pub fn func1(&mut self, name: &str, f: fn(f64) -> f64) {
        self.functions
            .insert(name.to_string(), Rc::new(FuncDef::native1(true, f)));
    }
    pub fn func2(&mut self, name: &str, f: fn(f64, f64) -> f64) {
        self.functions
            .insert(name.to_string(), Rc::new(FuncDef::native2(true, f)));
    }
    pub fn closure(&mut self, name: &str, arity: usize, pure: bool, f: impl Fn(&[f64]) -> f64 + 'static) {
        self.functions
            .insert(name.to_string(), Rc::new(FuncDef::closure(arity, pure, f)));
    }
}
impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}
pub struct Expr(Node);

impl Expr {
    pub fn eval(&self) -> f64 {
        self.0.eval()
    }
}
impl fmt::Debug for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn write_node(f: &mut fmt::Formatter<'_>, n: &Node, depth: usize) -> fmt::Result {
            let pad = "  ".repeat(depth);
            match n {
                Node::Const(v) => writeln!(f, "{pad}{v}"),
                Node::Var(slot) => writeln!(f, "{pad}var(={})", slot.get()),
                Node::Neg(a) => {
                    writeln!(f, "{pad}neg")?;
                    write_node(f, a, depth + 1)
                }
                Node::Add(a, b) => write_bin(f, "add", a, b, depth),
                Node::Sub(a, b) => write_bin(f, "sub", a, b, depth),
                Node::Mul(a, b) => write_bin(f, "mul", a, b, depth),
                Node::Div(a, b) => write_bin(f, "div", a, b, depth),
                Node::Mod(a, b) => write_bin(f, "mod", a, b, depth),
                Node::Pow(a, b) => write_bin(f, "pow", a, b, depth),
                Node::Comma(a, b) => write_bin(f, "comma", a, b, depth),
                Node::Call(_, args) => {
                    writeln!(f, "{pad}call/{}", args.len())?;
              for a in args {
                        write_node(f, a, depth + 1)?;
                 }
                    Ok(())
                }
            }
        }
        fn write_bin(
            f: &mut fmt::Formatter<'_>,
            name: &str,
            a: &Node,
            b: &Node,
            depth: usize,
        ) -> fmt::Result {
            writeln!(f, "{}{}", "  ".repeat(depth), name)?;
            write_node(f, a, depth + 1)?;
            write_node(f, b, depth + 1)
        }
        write_node(f, &self.0, 0)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseError {
    pub position: usize,
}
impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "parse error at byte offset {}", self.position)
    }
}
impl std::error::Error for ParseError {}
pub fn compile(expression: &str, ctx: &Context) -> Result<Expr, ParseError> {
    let parser = Parser::new(expression, ctx);
    parser.parse().map(Expr)
}
pub fn interp(expression: &str) -> Result<f64, ParseError> {
    let ctx = Context::new();
    compile(expression, &ctx).map(|e| e.eval())
}
