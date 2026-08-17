use std::collections::HashSet;

use crate::parse::*;


#[derive(Debug, Clone)]
pub enum Eval {
    Float(f32),
    Circle(CircleEval),
    Define(DefineEval),
    Assign(AssignEval),
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
    pub val: Box<Function>,
}

#[derive(Debug, Clone)]
pub struct AssignEval {
    pub name: String,
    pub val: Box<Function>,
}


#[derive(Debug, Clone)]
pub struct Function {
    pub inner: Expr, // assumed to be simplified?
    pub params: HashSet<String>,
}

pub trait Args {
    fn get(&self, name: &String) -> Option<Function>;
}

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

pub fn tighten(expr: &Expr) -> Result<Function, Todo> {
    let function: Function = match expr {
        &Expr::Float(f) => Eval::Float(f),
        Expr::Call(_) => return Err(Todo),
        Expr::Bin(BinExpr { op, left, right }) => {
            let left = tighten(left)?;
            let right = tighten(right)?;
            if left.params.is_empty() && right.params.is_empty() {
                let (Expr::Float(left), Expr::Float(right)) = (left.inner, right.inner) else { return Err(Todo); };
                Function {
                    inner: Expr::Float(binevalf(*op, left, right)),
                    params: HashSet::new()
                }
            } else {
                Function {
                    inner: Expr::Bin(BinExpr {
                        op: *op,
                        left: Box::new(left),
                        right: Box::new(right),
                    }),
                    params: left.params.into_iter().chain(right.params).collect(),
                }
            }
        },
        Expr::Neg(e) => {
            let e = match tighten(e)?;
            if e.params.is_empty() {
                let Expr::Float(e) = e.inner else { return Err(Todo); };
                Function {
                    inner: Expr::Float(-e),
                    params: HashSet::new(),
                }
            } else {
                Function {
                    inner: Expr::Neg(e),
                    params: e.params,
                }
            }
        },
        Expr::Factorial(_) => return Err(Todo),
        Expr::Name(name) =>
            if let Some(val) = args.get(name) {
                val
            } else {
                Function {
                    inner: Expr::Name(name.clone()),
                    params: HashSet::from([name.clone()]),
                }
            },

        Expr::Bad => return Err(Todo),
        Expr::Circle(CircleExpr { x, y, r }) => {
            let x = tighten(x, args)?;
            let y = tighten(y, args)?;
            let r = tighten(r, args)?;

            match (x, y, r) {
                (Eval::Float(x), Eval::Float(y), Eval::Float(r)) => Eval::Circle(CircleEval { x, y, r, }),
                _ => return Err(Todo),
            }
        }
        Expr::Define(DefineExpr { name, val }) => {
            let val = tighten(val, args)?;
            Eval::Define(DefineEval {
                name: name.clone(),
                val: Box::new(val)
            })
        }
        Expr::Assign(AssignExpr { name, val }) => {
            let val = tighten(val, args)?;
            Eval::Assign(AssignEval {
                name: name.clone(),
                val: Box::new(val)
            })
        }
    };

    Ok(function)
}

pub fn eval<A: Args>(function: Function, args: &A) -> Result<Eval, Todo> {
    todo!()
}