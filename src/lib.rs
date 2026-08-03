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

    fn call(&self, args: &[f46]) -> f46 {
        (self.call)(args)
    }
}