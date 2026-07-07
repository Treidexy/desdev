use eframe::wgpu::naga::MathFunction::Exp;
use log::error;
use pest::{Parser, pratt_parser::{Assoc, Op, PrattParser}};
use pest_derive::Parser;

#[derive(Debug)]
pub enum Eval {
    Float(f32),
    Circle(CircleEval),
    Assign(AssignEval),
}

#[derive(Debug)]
pub enum Expr {
    Bad,
    Float(f32),
    Name(String),
    Call(CallExpr),
    Bin(BinExpr),
    
    Neg(Box<Expr>),
    Factorial(Box<Expr>),
    Circle(CircleExpr),
    Assign(AssignExpr),
}

#[derive(Debug)]
pub struct CallExpr {
    pub callee: Box<Expr>,
    pub args: Vec<Expr>, // todo: named args
}

#[derive(Debug)]
pub struct BinExpr {
    pub op: BinOp,
    pub left: Box<Expr>,
    pub right: Box<Expr>,
}

#[derive(Debug)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,

    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,

    Arrow,
}


#[derive(Debug)]
pub struct CircleExpr {
    pub x: Box<Expr>,
    pub y: Box<Expr>,
    pub r: Box<Expr>,
}

#[derive(Debug, Clone, Copy)]
pub struct CircleEval {
    pub x: f32,
    pub y: f32,
    pub r: f32,
}

#[derive(Debug)]
pub struct AssignExpr {
    pub name: String,
    pub val: Box<Expr>,
}

#[derive(Debug)]
pub struct AssignEval {
    pub name: String,
    pub val: f32,
}


#[derive(Parser)]
#[grammar = "grammar.pest"]
struct LeParser;

lazy_static::lazy_static! {
    static ref PRATT_PARSER: PrattParser<Rule> = {
        use Rule::*;
        
        // Each `.op()` call increases the precedence.
        // Comparators have the lowest precedence, function calls have the highest.
        PrattParser::new()
            .op(Op::infix(arrow, Assoc::Left))
            .op(Op::infix(eq, Assoc::Left)
                | Op::infix(ne, Assoc::Left)
                | Op::infix(lt, Assoc::Left)
                | Op::infix(le, Assoc::Left)
                | Op::infix(gt, Assoc::Left)
                | Op::infix(ge, Assoc::Left))
            .op(Op::infix(add, Assoc::Left) | Op::infix(sub, Assoc::Left))
            .op(Op::infix(mul, Assoc::Left) | Op::infix(div, Assoc::Left))
            .op(Op::infix(pow, Assoc::Left))
            .op(Op::prefix(neg))
            .op(Op::postfix(call) | Op::postfix(factorial))
    };
}

pub fn evalf(expr: &Expr) -> Option<f32> {
    match expr {
        Expr::Bad => None,
        &Expr::Float(f) => Some(f),
        Expr::Name(_) => None,
        Expr::Call(_) => None,
        Expr::Bin(BinExpr { op, left, right }) => match op {
            BinOp::Add => Some(evalf(left)? + evalf(right)?),
            BinOp::Sub => Some(evalf(left)? - evalf(right)?),
            BinOp::Mul => Some(evalf(left)? * evalf(right)?),
            BinOp::Div => Some(evalf(left)? / evalf(right)?),
            BinOp::Pow => Some(evalf(left)?.powf(evalf(right)?)),
            BinOp::Eq => None,
            BinOp::Ne => None,
            BinOp::Lt => None,
            BinOp::Le => None,
            BinOp::Gt => None,
            BinOp::Ge => None,
            BinOp::Arrow => None,
        },
        Expr::Neg(e) => evalf(e).map(|f: f32| -f),
        Expr::Factorial(_) => None,
        Expr::Circle(_) => None,
        Expr::Assign(AssignExpr { name, val }) => evalf(val),
    }
}

pub fn eval(expr: &Expr) -> Option<Eval> {
    dbg!(expr);

    match expr {
        Expr::Neg(_) | Expr::Float(_) | Expr::Factorial(_) | Expr::Bin(_) | Expr::Call(_) => evalf(expr).map(Eval::Float),

        Expr::Bad => None,
        Expr::Name(_) => None,
        Expr::Circle(CircleExpr { x, y, r }) => {
            let x = evalf(x)?;
            let y = evalf(y)?;
            let r = evalf(r)?;
            Some(Eval::Circle(CircleEval { x, y, r }))
        }
        Expr::Assign(AssignExpr { name, val }) => {
            let val = evalf(val)?;
            Some(Eval::Assign(AssignEval { name: name.clone(), val }))
        }
    }
}

fn parse_expr(pairs: pest::iterators::Pairs<Rule>) -> Expr {
    PRATT_PARSER
        .map_primary(|primary| match primary.as_rule() {
            Rule::circle => {
                let mut args= primary.into_inner();
                let Some(x) = args.next() else {
                    return Expr::Bad;
                };
                let Some(y) = args.next() else {
                    return Expr::Bad;
                };
                let Some(r) = args.next() else {
                    return Expr::Bad;
                };
                let x = Box::new(parse_expr(x.into_inner()));
                let y = Box::new(parse_expr(y.into_inner()));
                let r = Box::new(parse_expr(r.into_inner()));
                Expr::Circle(CircleExpr { x, y, r })
            },
            Rule::number => Expr::Float(primary.as_str().parse().unwrap()),
            Rule::name => Expr::Name(primary.as_str().to_string()),
            // If it's parentheses, we evaluate the inner expression
            Rule::expr => parse_expr(primary.into_inner()),
            rule => unreachable!("Expected atom, found {:?}", rule),
        })
        .map_prefix(|op, right| match op.as_rule() {
            Rule::neg => Expr::Neg(Box::new(right)),
            _ => unreachable!(),
        })
        .map_postfix(|left, op| match op.as_rule() {
            Rule::call => {
                let args = op
                    .into_inner()
                    .map(|arg| parse_expr(arg.into_inner()))
                    .collect();
                Expr::Call(CallExpr { callee: Box::new(left), args })
            }
            Rule::factorial => Expr::Factorial(Box::new(left)), // <-- Handle factorial here
            _ => unreachable!(),
        })
        .map_infix(|left, op, right| {
            let op = match op.as_rule() {
                Rule::add => BinOp::Add,
                Rule::sub => BinOp::Sub,
                Rule::mul => BinOp::Mul,
                Rule::div => BinOp::Div,
                Rule::pow => BinOp::Pow,

                Rule::eq  => {
                    // bc pest is a fucking imbecile
                    if let Expr::Name(name) = left {
                        return Expr::Assign(AssignExpr { name, val: Box::new(right), })
                    }

                    BinOp::Eq
                },
                Rule::ne => BinOp::Ne,
                Rule::lt  => BinOp::Lt,
                Rule::le => BinOp::Le,
                Rule::gt  => BinOp::Gt,
                Rule::ge => BinOp::Ge,

                Rule::arrow => BinOp::Arrow,
                _ => unreachable!(),
            };
            Expr::Bin(BinExpr { op, left: Box::new(left), right: Box::new(right) })
        })
        .parse(pairs) // Execute the pratt parse
}

pub fn parse(input: &str) -> Result<Expr, pest::error::Error<Rule>> {
    let mut pairs = LeParser::parse(Rule::line, input)?;
    let pairs = pairs.next().unwrap().into_inner();
    let ast = parse_expr(pairs);
    Ok(ast)
}