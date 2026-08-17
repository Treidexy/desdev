use std::{collections::{HashMap, HashSet}, intrinsics::unreachable};

use crate::parse::{BinExpr, BinOp, Expr};


#[derive(Debug)]
pub enum Eval {
    Float(f32),
    Circle(CircleEval),
    Define(DefineEval),
    Assign(AssignEval),
    Functio(FunctionEval),
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


#[derive(Debug)]
pub struct FunctionEval {
    pub inner: Expr,
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
fn evalf(expr: &Expr, ctx: &HashMap<String, f32>) -> Result<f32, FunctionEval> {
    match expr {
        Expr::Bad => unreachable!(),
        &Expr::Float(f) => Ok(f),
        Expr::Name(name) => if let Some(&val) = ctx.get(name) {
            Ok(val)
        } else {
            Err(FunctionEval {
                inner: Expr::Name(name.clone()),
                params: HashSet::from([name.clone()]),
            })
        },
        Expr::Call(_) => todo!(),
        Expr::Bin(BinExpr { op, left, right }) => {
            let left = evalf(left, ctx);
            let right = evalf(right, ctx);
            match (left, right) {
                (Ok(left), Ok(right)) => return Ok(binevalf(*op, left, right)),
                (Ok(left), Err(right)) => todo!(),
                (Err(left), Ok(right)) => todo!(),
                (Err(left), Err(right)) => {
                    FunctionEval {
                        inner: Expr::Bin(BinExpr {
                            op: *op,
                            left: Box::new(left.inner),
                            right: Box::new(right.inner),
                        }),
                        params: left.params.union(&right.params).collect::<_>(),
                    };
                },
            };

            todo!();
        },
        Expr::Neg(e) => self.evalf(e).map(|f: f32| -f),
        Expr::Factorial(_) => None,
        Expr::Circle(_) => None,
        Expr::Define(_) => None,
        Expr::Assign(_) => None,
    }
}

fn eval(expr: &Expr) -> Option<Eval> {
    match expr {
        Expr::Neg(_) | Expr::Float(_) | Expr::Factorial(_) | Expr::Bin(_) | Expr::Call(_) => self.evalf(expr).map(Eval::Float),

        Expr::Bad => None,
        Expr::Name(_) => None,
        Expr::Circle(CircleExpr { x, y, r }) => {
            let x = self.evalf(x)?;
            let y = self.evalf(y)?;
            let r = self.evalf(r)?;
            Some(Eval::Circle(CircleEval { x, y, r }))
        }
        Expr::Define(DefineExpr { name, val }) => {
            let val = self.evalf(val)?;
            Some(Eval::Define(DefineEval { name: name.clone(), val }))
        }
        Expr::Assign(AssignExpr { name, val }) => {
            let val = self.evalf(val)?;
            Some(Eval::Assign(AssignEval { name: name.clone(), val }))
        }
    }
}