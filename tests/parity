use tinyexpr::{compile, interp, Context};
fn approx(a: f64, b: f64) {
    let ok = (a.is_nan() && b.is_nan()) || a == b || (a - b).abs() < 1e-9;
    assert!(ok, "{a} != {b}");
}
#[test]
fn constants_and_arithmetic() {
    approx(interp("1").unwrap(), 1.0);
    approx(interp("1 ").unwrap(), 1.0);
    approx(interp("(1)").unwrap(), 1.0);
    approx(interp("pi").unwrap(), std::f64::consts::PI);
    approx(interp("atan(1)*4 - pi").unwrap(), 0.0);
    approx(interp("e").unwrap(), std::f64::consts::E);
    approx(interp("2+1").unwrap(), 3.0);
    approx(interp("(((2+(1))))").unwrap(), 3.0);
    approx(interp("3+2").unwrap(), 5.0);
    approx(interp("3+2-4").unwrap(), 1.0);
    approx(interp("3+2*4").unwrap(), 11.0);
    approx(interp("3.0+2*4").unwrap(), 11.0);
    approx(interp("3.0+2.0*4.0").unwrap(), 11.0);
    approx(interp("3.0/2.0*4.0").unwrap(), 6.0);
    approx(interp("2^2").unwrap(), 4.0);
    approx(interp("10 % 3").unwrap(), 1.0);
}
#[test]
fn unary_and_pow_precedence() {
    approx(interp("-2^2").unwrap(), 4.0);
    approx(interp("2^-2").unwrap(), 0.25);
    approx(interp("-2^-2").unwrap(), 0.25);
    approx(interp("2^2^3").unwrap(), 64.0);
    approx(interp("-2").unwrap(), -2.0);
    approx(interp("--2").unwrap(), 2.0);
    approx(interp("---2").unwrap(), -2.0);
    approx(interp("-(2)").unwrap(), -2.0);
    approx(interp("-(-2)").unwrap(), 2.0);
}
#[test]
fn functions_no_paren_unary() {
    approx(interp("sqrt 100").unwrap(), 10.0);
    approx(interp("sqrt(100)").unwrap(), 10.0);
    approx(interp("sqrt (100)").unwrap(), 10.0);
    approx(interp("sqrt(100) + 1").unwrap(), 11.0);
    approx(interp("sqrt 100 + 1").unwrap(), 11.0);
    approx(interp("-sqrt(100)").unwrap(), -10.0);
}
#[test]
fn multi_arg_functions() {
    approx(interp("atan2(1,1)").unwrap(), std::f64::consts::FRAC_PI_4);
    approx(interp("pow(2,10)").unwrap(), 1024.0);
    approx(interp("ncr(6,2)").unwrap(), 15.0);
    approx(interp("npr(6,2)").unwrap(), 30.0);
    approx(interp("fac(5)").unwrap(), 120.0);
    approx(interp("log10(1000)").unwrap(), 3.0);
    approx(interp("log(1000)").unwrap(), 3.0);
}
#[test]
fn natural_log_variant() {
    let ctx = Context::with_natural_log();
    let r = compile("log(e)", &ctx).unwrap().eval();
    approx(r, 1.0);
}
#[test]
fn zero_arity_optional_parens() {
    approx(interp("pi()").unwrap(), std::f64::consts::PI);
    approx(interp("pi").unwrap(), std::f64::consts::PI);
}
#[test]
fn comma_list() {
    approx(interp("1,2").unwrap(), 2.0);
    approx(interp("1,2,3").unwrap(), 3.0);
    approx(interp("(1,2),3").unwrap(), 3.0);
}
#[test]
fn variables_are_live_bound() {
    let mut ctx = Context::new();
    let x = ctx.var("x");
    let y = ctx.var("y");
    let expr = compile("sqrt(x^2+y^2)", &ctx).unwrap();
    x.set(3.0);
    y.set(4.0);
    approx(expr.eval(), 5.0);
    x.set(6.0);
    y.set(8.0);
    approx(expr.eval(), 10.0);
}
#[test]
fn user_variable_shadows_builtin() {
    let mut ctx = Context::new();
    let e = ctx.var("e");
    e.set(123.0);
    let expr = compile("e", &ctx).unwrap();
    approx(expr.eval(), 123.0);
}
#[test]
fn user_closures() {
    let mut ctx = Context::new();
    ctx.closure("double", 1, true, |args| args[0] * 2.0);
    ctx.closure("sum3", 3, true, |args| args[0] + args[1] + args[2]);
    approx(compile("double(21)", &ctx).unwrap().eval(), 42.0);
    approx(compile("sum3(1,2,3)", &ctx).unwrap().eval(), 6.0);
}
#[test]
fn impure_closure_not_folded_but_still_called() {
    use std::cell::Cell;
    use std::rc::Rc;
    let calls = Rc::new(Cell::new(0));
    let calls2 = calls.clone();
    let mut ctx = Context::new();
    ctx.closure("counter", 0, false, move |_| {
        calls2.set(calls2.get() + 1);
        calls2.get() as f64
    });
    let expr = compile("counter()", &ctx).unwrap();
    approx(expr.eval(), 1.0);
    approx(expr.eval(), 2.0);
    approx(expr.eval(), 3.0);
}
#[test]
fn errors_report_a_position() {
    assert!(interp("").is_err());
    assert!(interp("1+").is_err());
    assert!(interp("1+1x").is_err());
    assert!(interp("(1").is_err());
    assert!(interp("1)").is_err());
    assert!(interp("atan2(1)").is_err());
    assert!(interp("unknown_fn(1)").is_err());
    let err = interp("1+2+").unwrap_err();
    assert_eq!(err.position, 4);
}
#[test]
fn divide_by_zero_is_inf_or_nan_not_a_panic() {
    approx(interp("1/0").unwrap(), f64::INFINITY);
    assert!(interp("0/0").unwrap().is_nan());
}
#[test]
fn nested_parens_and_whitespace() {
    approx(interp("  ( 1 + 2 ) * ( 3 - 1 )  ").unwrap(), 6.0);
}
