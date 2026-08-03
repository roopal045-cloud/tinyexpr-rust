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
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum InfixOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Mod,
}
#[derive(Clone)]
enum Token {
    Number(f64),
    Variable(VarSlot),
    Function(Rc<FuncDef>),
    Infix(InfixOp),
    Open,
    Close,
    Sep,
    End,
    Error(usize),
}
struct Lexer<'a> {
    src: &'a str,
    pos: usize,
    ctx: &'a Context,
}
impl<'a> Lexer<'a> {
    fn new(src: &'a str, ctx: &'a Context) -> Self {
        Lexer { src, pos: 0, ctx }
    }
    fn pos(&self) -> usize {
        self.pos
    }
    fn rest(&self) -> &'a str {
        &self.src[self.pos..]
    }
    fn peek_byte(&self) -> Option<u8> {
        self.rest().as_bytes().first().copied()
    }
    fn next(&mut self) -> Token {
        loop {
            let Some(b0) = self.peek_byte() else {
                return Token::End;
            };
            if b0.is_ascii_digit() || b0 == b'.' {
                return self.read_number();
            }
            if b0.is_ascii_alphabetic() {
                return self.read_identifier();
            }
            let start = self.pos;
            self.pos += 1;
            match b0 {
                b'+' => return Token::Infix(InfixOp::Add),
                b'-' => return Token::Infix(InfixOp::Sub),
                b'*' => return Token::Infix(InfixOp::Mul),
                b'/' => return Token::Infix(InfixOp::Div),
                b'^' => return Token::Infix(InfixOp::Pow),
                b'%' => return Token::Infix(InfixOp::Mod),
                b'(' => return Token::Open,
                b')' => return Token::Close,
                b',' => return Token::Sep,
                b' ' | b'\t' | b'\n' | b'\r' => continue,
                _ => return Token::Error(start),
            }
        }
    }
    fn read_number(&mut self) -> Token {
        let bytes = self.rest().as_bytes();
        let mut i = 0;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if i < bytes.len() && bytes[i] == b'.' {
            i += 1;
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                i += 1;
            }
        }
        if i < bytes.len() && (bytes[i] == b'e' || bytes[i] == b'E') {
            let mut j = i + 1;
            if j < bytes.len() && (bytes[j] == b'+' || bytes[j] == b'-') {
                j += 1;
            }
            if j < bytes.len() && bytes[j].is_ascii_digit() {
                while j < bytes.len() && bytes[j].is_ascii_digit() {
                    j += 1;
                }
                i = j;
            }
        }
        let text = &self.rest()[..i];
        let value: f64 = text.parse().unwrap_or(f64::NAN);
        self.pos += i;
        Token::Number(value)
    }
    fn read_identifier(&mut self) -> Token {
        let start = self.pos;
        let bytes = self.rest().as_bytes();
        let mut i = 0;
        while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
            i += 1;
        }
        let name = &self.rest()[..i];
        self.pos += i;
        if let Some(slot) = self.ctx.variables.get(name) {
            return Token::Variable(slot.clone());
        }
        if let Some(func) = self.ctx.functions.get(name) {
            return Token::Function(func.clone());
        }
        Token::Error(start)
    }
}
#[derive(Clone)]
enum Node {
    Const(f64),
    Var(VarSlot),
    Neg(Box<Node>),
    Add(Box<Node>, Box<Node>),
    Sub(Box<Node>, Box<Node>),
    Mul(Box<Node>, Box<Node>),
    Div(Box<Node>, Box<Node>),
    Mod(Box<Node>, Box<Node>),
    Pow(Box<Node>, Box<Node>),
    Comma(Box<Node>, Box<Node>),
    Call(Rc<FuncDef>, Vec<Node>),
}
impl Node {
    fn eval(&self) -> f64 {
        match self {
            Node::Const(v) => *v,
            Node::Var(slot) => slot.get(),
            Node::Neg(a) => -a.eval(),
            Node::Add(a, b) => a.eval() + b.eval(),
            Node::Sub(a, b) => a.eval() - b.eval(),
            Node::Mul(a, b) => a.eval() * b.eval(),
            Node::Div(a, b) => a.eval() / b.eval(),
            Node::Mod(a, b) => a.eval() % b.eval(),
            Node::Pow(a, b) => a.eval().powf(b.eval()),
            Node::Comma(a, b) => {
                let _ = a.eval();
                b.eval()
            }
            Node::Call(f, args) => {
                let vals: Vec<f64> = args.iter().map(Node::eval).collect();
                f.call(&vals)
            }
        }
    }
}
fn optimize(n: Node) -> Node {
    fn as_const(n: &Node) -> Option<f64> {
        match n {
            Node::Const(v) => Some(*v),
            _ => None,
        }
    }
    match n {
        Node::Const(_) | Node::Var(_) => n,
        Node::Neg(a) => {
            let a = optimize(*a);
            match as_const(&a) {
                Some(av) => Node::Const(-av),
                None => Node::Neg(Box::new(a)),
            }
        }
        Node::Add(a, b) => fold2(optimize(*a), optimize(*b), Node::Add, |x, y| x + y),
        Node::Sub(a, b) => fold2(optimize(*a), optimize(*b), Node::Sub, |x, y| x - y),
        Node::Mul(a, b) => fold2(optimize(*a), optimize(*b), Node::Mul, |x, y| x * y),
        Node::Div(a, b) => fold2(optimize(*a), optimize(*b), Node::Div, |x, y| x / y),
        Node::Mod(a, b) => fold2(optimize(*a), optimize(*b), Node::Mod, |x, y| x % y),
        Node::Pow(a, b) => fold2(optimize(*a), optimize(*b), Node::Pow, f64::powf),
        Node::Comma(a, b) => fold2(optimize(*a), optimize(*b), Node::Comma, |_, y| y),
        Node::Call(f, args) => {
            let args: Vec<Node> = args.into_iter().map(optimize).collect();
            if f.pure {
                let mut vals = Vec::with_capacity(args.len());
                let mut all_const = true;
                for a in &args {
                    match as_const(a) {
                        Some(v) => vals.push(v),
                        None => {
                            all_const = false;
                            break;
                        }
                    }
                }
                if all_const {
                    return Node::Const(f.call(&vals));
                }
            }
            Node::Call(f, args)
        }
    }
}
fn fold2(
    a: Node,
    b: Node,
    make: impl FnOnce(Box<Node>, Box<Node>) -> Node,
    apply: impl FnOnce(f64, f64) -> f64,
) -> Node {
    if let (Node::Const(av), Node::Const(bv)) = (&a, &b) {
        Node::Const(apply(*av, *bv))
    } else {
        make(Box::new(a), Box::new(b))
    }
}
struct Parser<'a> {
    lexer: Lexer<'a>,
    cur: Token,
    cur_pos: usize,
}
impl<'a> Parser<'a> {
    fn new(src: &'a str, ctx: &'a Context) -> Self {
        let mut lexer = Lexer::new(src, ctx);
        let cur_pos = lexer.pos();
        let cur = lexer.next();
        Parser { lexer, cur, cur_pos }
    }
 fn advance(&mut self) {
        self.cur_pos = self.lexer.pos();
        self.cur = self.lexer.next();
    }
    fn err(&self) -> ParseError {
        ParseError {
            position: self.cur_pos,
        }
    }
    fn parse(mut self) -> Result<Node, ParseError> {
        let root = self.list()?;
        if !matches!(self.cur, Token::End) {
            return Err(self.err());
        }
        Ok(optimize(root))
    }
    fn list(&mut self) -> Result<Node, ParseError> {
        let mut ret = self.expr()?;
        while matches!(self.cur, Token::Sep) {
            self.advance();
            let rhs = self.expr()?;
            ret = Node::Comma(Box::new(ret), Box::new(rhs));
        }
        Ok(ret)
    }
    fn expr(&mut self) -> Result<Node, ParseError> {
        let mut ret = self.term()?;
        loop {
            match self.cur {
                Token::Infix(InfixOp::Add) => {
                    self.advance();
                    let rhs = self.term()?;
                    ret = Node::Add(Box::new(ret), Box::new(rhs));
                }
                Token::Infix(InfixOp::Sub) => {
                    self.advance();
                    let rhs = self.term()?;
                    ret = Node::Sub(Box::new(ret), Box::new(rhs));
                }
                _ => break,
            }
        }
        Ok(ret)
    }   fn term(&mut self) -> Result<Node, ParseError> {
        let mut ret = self.factor()?;
        loop {
            match self.cur {
                Token::Infix(InfixOp::Mul) => {
                    self.advance();
                    let rhs = self.factor()?;
                    ret = Node::Mul(Box::new(ret), Box::new(rhs));
                }
                Token::Infix(InfixOp::Div) => {
                    self.advance();
                    let rhs = self.factor()?;
                    ret = Node::Div(Box::new(ret), Box::new(rhs));
                }
                Token::Infix(InfixOp::Mod) => {
                    self.advance();
                    let rhs = self.factor()?;
                    ret = Node::Mod(Box::new(ret), Box::new(rhs));
                }
                _ => break,
            }
        }
        Ok(ret)
    }
  fn factor(&mut self) -> Result<Node, ParseError> {
        let mut ret = self.power()?;
        while matches!(self.cur, Token::Infix(InfixOp::Pow)) {
            self.advance();
            let rhs = self.power()?;
            ret = Node::Pow(Box::new(ret), Box::new(rhs));
        }
        Ok(ret)
    } fn power(&mut self) -> Result<Node, ParseError> {
        let mut sign = 1i32;
        loop {
            match self.cur {
                Token::Infix(InfixOp::Add) => self.advance(),
                Token::Infix(InfixOp::Sub) => {
                    sign = -sign;
                    self.advance();
                }
                _ => break,
            }
        }
        let b = self.base()?;
        Ok(if sign == -1 { Node::Neg(Box::new(b)) } else { b })
    }
    fn base(&mut self) -> Result<Node, ParseError> {
        match self.cur.clone() {
            Token::Number(v) => {
                self.advance();
                Ok(Node::Const(v))
            }
            Token::Variable(slot) => {
                self.advance();
                Ok(Node::Var(slot))
            }
            Token::Function(f) => {
                let arity = f.arity;
                self.advance();
                match arity {
                    0 => {
                        if matches!(self.cur, Token::Open) {
                            self.advance();
                            if !matches!(self.cur, Token::Close) {
                                return Err(self.err());
                            }
                            self.advance();
                        }
                        Ok(Node::Call(f, Vec::new()))
                    }
                    1 => {
                        let arg = self.power()?;
                        Ok(Node::Call(f, vec![arg]))
                    }
                    n => {
                        if !matches!(self.cur, Token::Open) {
                            return Err(self.err());
                        }
                        self.advance();
                        let mut args = Vec::with_capacity(n);
                        for i in 0..n {
                            let a = self.expr()?;
                            args.push(a);
                            if matches!(self.cur, Token::Sep) {
                                self.advance();
                            } else if i != n - 1 {
                                break;
                            }
                        }
                        if !matches!(self.cur, Token::Close) || args.len() != n {
                            return Err(self.err());
                        }
                        self.advance();
                        Ok(Node::Call(f, args))
                    }
                }
            }
            Token::Open => {
                self.advance();
                let inner = self.list()?;
                if !matches!(self.cur, Token::Close) {
                    return Err(self.err());
                }
                self.advance();
                Ok(inner)
            }
            Token::Error(pos) => Err(ParseError { position: pos }),
            _ => Err(self.err()),
        }
    }
}
fn fac(a: f64) -> f64 {
    if a < 0.0 {
        return f64::NAN;
    }
    if a > u32::MAX as f64 {
        return f64::INFINITY;
    }
    let ua = a as u32;
    let mut result: u64 = 1;
    for i in 1..=(ua as u64) {
        if i > u64::MAX / result {
            return f64::INFINITY;
        }
        result *= i;
    }
    result as f64
}
fn ncr(n: f64, r: f64) -> f64 {
    if n < 0.0 || r < 0.0 || n < r {
        return f64::NAN;
    }
    if n > u32::MAX as f64 || r > u32::MAX as f64 {
        return f64::INFINITY;
    }
    let un = n as u64;
    let mut ur = r as u64;
    if ur > un / 2 {
        ur = un - ur;
    }
    let mut result: u64 = 1;
    let mut i: u64 = 1;
    while i <= ur {
        if result > u64::MAX / (un - ur + i) {
            return f64::INFINITY;
        }
        result *= un - ur + i;
        result /= i;
        i += 1;
    }
    result as f64
}
fn npr(n: f64, r: f64) -> f64 {
    ncr(n, r) * fac(r)
}
fn pi() -> f64 {
    std::f64::consts::PI
}
fn e() -> f64 {
    std::f64::consts::E
}
fn builtin_table(nat_log: bool) -> Vec<(&'static str, Rc<FuncDef>)> {
    macro_rules! f0 {
        ($f:expr) => {
            Rc::new(FuncDef::native0(true, $f))
        };
    }
    macro_rules! f1 {
        ($f:expr) => {
            Rc::new(FuncDef::native1(true, $f))
        };
    }
    macro_rules! f2 {
        ($f:expr) => {
            Rc::new(FuncDef::native2(true, $f))
        };
    }
    vec![
        ("abs", f1!(f64::abs)),
        ("acos", f1!(f64::acos)),
        ("asin", f1!(f64::asin)),
        ("atan", f1!(f64::atan)),
        ("atan2", f2!(f64::atan2)),
        ("ceil", f1!(f64::ceil)),
        ("cos", f1!(f64::cos)),
        ("cosh", f1!(f64::cosh)),
        ("e", f0!(e)),
        ("exp", f1!(f64::exp)),
        ("fac", f1!(fac)),
        ("floor", f1!(f64::floor)),
        ("ln", f1!(f64::ln)),
        (
            "log",
            if nat_log {
                f1!(f64::ln)
            } else {
                f1!(f64::log10)
            },
        ),
        ("log10", f1!(f64::log10)),
        ("ncr", f2!(ncr)),
        ("npr", f2!(npr)),
        ("pi", f0!(pi)),
        ("pow", f2!(f64::powf)),
        ("sin", f1!(f64::sin)),
        ("sinh", f1!(f64::sinh)),
        ("sqrt", f1!(f64::sqrt)),
        ("tan", f1!(f64::tan)),
        ("tanh", f1!(f64::tanh)),
    ]
}
