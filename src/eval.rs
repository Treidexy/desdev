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
    pub val: f32,
}

#[derive(Debug, Clone)]
pub struct AssignEval {
    pub name: String,
    pub val: f32,
}


#[derive(Debug, Clone)]
pub struct FunctionEval {
    pub inner: Expr, // assumed to be simplified?
    pub params: HashSet<String>,
}

fn binevalf(op: BinOp, left: f32, right: f32) -> f32 {
    match op {
        BinOp::Add => left + right,
        BinOp::Sub => left - right,
        BinOp::Mul => left * right,
        BinOp::Div => left / right,
        BinOp::Pow => left.powf(right),
        BinOp::Eq => todo!(),
        BinOp::Ne => todo!(),
        BinOp::Lt => todo!(),
        BinOp::Le => todo!(),
        BinOp::Gt => todo!(),
        BinOp::Ge => todo!(),
        BinOp::Arrow => todo!(),
    }
}

// should expr be own vez ref?
fn evalf(expr: &Expr, args: &HashMap<String, Eval>) -> Result<f32, FunctionEval> {
    match expr {
        Expr::Bad | Expr::Circle(_) | Expr::Define(_) | Expr::Assign(_) => unreachable!(),
        &Expr::Float(f) => Ok(f),
        Expr::Name(name) => if let Some(val) = args.get(name) {
            let Eval::Float(val) = val else { unreachable!() };
            Ok(*val)
        } else {
            Err(FunctionEval {
                inner: Expr::Name(name.clone()),
                params: HashSet::from([name.clone()]),
            })
        },
        Expr::Call(_) => todo!(),
        Expr::Bin(BinExpr { op, left, right }) => {
            let left = evalf(left, args);
            let right = evalf(right, args);
            let (left, right, params) = match (left, right) {
                (Ok(left), Ok(right)) => return Ok(binevalf(*op, left, right)),
                (Ok(left), Err(right)) => (Expr::Float(left), right.inner, right.params),
                (Err(left), Ok(right)) => (left.inner, Expr::Float(right), left.params),
                (Err(left), Err(right)) => (left.inner, right.inner, left.params.union(&right.params).collect::<_>()),
            };

            Err(FunctionEval {
                inner: Expr::Bin(BinExpr {
                    op: *op,
                    left: Box::new(left),
                    right: Box::new(right),
                }),
                params,
            })
        },
        Expr::Neg(e) => evalf(e, args).map(|f: f32| -f),
        Expr::Factorial(_) => todo!(),
    }
}

fn eval(expr: &Expr, args: &HashMap<String, Eval>) -> Eval {
    match expr {
        Expr::Neg(_) | Expr::Float(_) | Expr::Factorial(_) | Expr::Bin(_) | Expr::Call(_) => match evalf(expr, args) {
            Ok(f) => Eval::Float(f),
            Err(e) => Eval::Function(e),
        },
        Expr::Name(name) => if let Some(val) = args.get(name) {
            val.clone()
        } else {
            Eval::Function(FunctionEval {
                inner: Expr::Name(name.clone()),
                params: HashSet::from([name.clone()]),
            })
        },

        Expr::Bad => todo!(),
        Expr::Circle(CircleExpr { x, y, r }) => {
            let x = evalf(x, args);
            let y = evalf(y, args);
            let r = evalf(r, args);

            match (x, y, r) {
                (Ok(x), Ok(y), Ok(r)) => return Eval::Circle(CircleEval { x, y, r, }),
                (Ok(_), Ok(_), Err(_)) => todo!(),
                (Ok(_), Err(_), Ok(_)) => todo!(),
                (Ok(_), Err(_), Err(_)) => todo!(),
                (Err(_), Ok(_), Ok(_)) => todo!(),
                (Err(_), Ok(_), Err(_)) => todo!(),
                (Err(_), Err(_), Ok(_)) => todo!(),
                (Err(_), Err(_), Err(_)) => todo!(),
            }
        }
        Expr::Define(DefineExpr { name, val }) => {
            let val = eval(val, args);
            Eval::Define(DefineEval { name: name.clone(), val })
        }
        Expr::Assign(AssignExpr { name, val }) => {
            let val = eval(val, args);
            Eval::Assign(AssignEval { name: name.clone(), val })
        }
    }
}