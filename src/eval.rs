use std::collections::{HashMap, HashSet};

use crate::parse::*;


#[derive(Debug, Clone)]
pub enum Eval {
    Float(f32),
    Circle(CircleEval),
    Define(DefineEval),
    Assign(AssignEval),
    Function(FunctionEval),
}

#[derive(Debug, Clone, Copy)]
pub struct CircleEval {
    pub x: f32,
    pub y: f32,
    pub r: f32,
}

#[derive(Debug, Clone)]
pub struct DefineEval {
    pub name: String,
    pub val: Box<Eval>,
}

#[derive(Debug, Clone)]
pub struct AssignEval {
    pub name: String,
    pub val: Box<Eval>,
}


#[derive(Debug, Clone)]
pub struct FunctionEval {
    pub inner: Expr, // assumed to be simplified?
    pub params: HashSet<String>,
}

pub trait Args : Fn(&String) -> Option<Eval> {}
impl<T: Fn(&String) -> Option<Eval>> Args for T {}

fn binevalf(op: BinOp, left: f32, right: f32) -> f32 {
    match op {
        BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge | BinOp::Arrow => unreachable!(),

        BinOp::Add => left + right,
        BinOp::Sub => left - right,
        BinOp::Mul => left * right,
        BinOp::Div => left / right,
        BinOp::Pow => left.powf(right),
    }
}

pub struct Todo;

pub fn eval_slate(SlateExpr { inner, args: args_expr }: &SlateExpr, big_args: &dyn Args) -> Result<Eval, Todo> {
    let mut small_args = HashMap::new();
    for (name, val) in args_expr {
        small_args.insert(name, eval(val, big_args)?);
    }

    let small_args = |name| small_args.get(name).cloned();
    let args = |name| {
        if let Some(val) = small_args(name) {
            return Some(val);
        }
        
        let eval_result = big_args(name)?;
        if let Eval::Function(FunctionEval { inner, params }) = eval_result {
            eval(&inner, &args).ok()
        } else {
            Some(eval_result)
        }
    };

    eval(inner, &args)
}

pub fn eval(expr: &Expr, args: &dyn Args) -> Result<Eval, Todo> {
    let eval = match expr {
        &Expr::Float(f) => Eval::Float(f),
        Expr::Slate(slate) => eval_slate(slate, args)?,
        Expr::Bin(BinExpr { op, left, right }) => {
            let left = eval(left, args)?;
            let right = eval(right, args)?;
            let (left, right, params) = match (left, right) {
                (Eval::Float(left), Eval::Float(right)) => return Ok(Eval::Float(binevalf(*op, left, right))),
                (Eval::Float(left), Eval::Function(right)) => (Expr::Float(left), right.inner, right.params),
                (Eval::Function(left), Eval::Float(right)) => (left.inner, Expr::Float(right), left.params),
                (Eval::Function(left), Eval::Function(right)) => (
                    left.inner,
                    right.inner,
                    left.params.into_iter().chain(right.params).collect(),
                ),
                _ => return Err(Todo),
            };

            Eval::Function(FunctionEval {
                inner: Expr::Bin(BinExpr {
                    op: *op,
                    left: Box::new(left),
                    right: Box::new(right),
                }),
                params,
            })
        },
        Expr::Neg(e) => match eval(e, args)? {
            Eval::Float(f) => Eval::Float(-f),
            Eval::Function(f) => Eval::Function(FunctionEval {
                inner: Expr::Neg(Box::new(f.inner)),
                params: f.params,
            }),
            _ => return Err(Todo),
        },
        Expr::Factorial(_) => return Err(Todo),
        Expr::Name(name) =>
            if let Some(val) = args(name) {
                val
            } else {
                Eval::Function(FunctionEval {
                    inner: Expr::Name(name.clone()),
                    params: HashSet::from([name.clone()]),
                })
            },

        Expr::Bad => return Err(Todo),
        Expr::Circle(CircleExpr { x, y, r }) => {
            let x = eval(x, args)?;
            let y = eval(y, args)?;
            let r = eval(r, args)?;

            match (x, y, r) {
                (Eval::Float(x), Eval::Float(y), Eval::Float(r)) => Eval::Circle(CircleEval { x, y, r, }),
                _ => return Err(Todo),
            }
        }
        Expr::Define(DefineExpr { name, val }) => {
            let val = eval(val, args)?;
            Eval::Define(DefineEval {
                name: name.clone(),
                val: Box::new(val)
            })
        }
        Expr::Assign(AssignExpr { name, val }) => {
            let val = eval(val, args)?;
            Eval::Assign(AssignEval {
                name: name.clone(),
                val: Box::new(val)
            })
        }
    };

    Ok(eval)
}

pub fn stab<A: Args>(function: FunctionEval, args: &A) -> Result<Eval, Todo> {
    eval(&function.inner, args)
}