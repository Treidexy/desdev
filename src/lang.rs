use pest::{Parser, pratt_parser::{Assoc, Op, PrattParser}};
use pest_derive::Parser;

#[derive(Debug)]
pub enum Eval {
    Float(f32),
    Circle(CircleEval),
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


#[derive(Parser)]
#[grammar = "grammar.pest"]
struct LeParser;

lazy_static::lazy_static! {
    static ref PRATT_PARSER: PrattParser<Rule> = {
        use Rule::*;
        
        // Each `.op()` call increases the precedence.
        // Comparators have the lowest precedence, function calls have the highest.
        PrattParser::new()
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
        Expr::Bin(_) => None,
        Expr::Neg(e) => evalf(e).map(|f: f32| -f),
        Expr::Factorial(_) => None,
        Expr::Circle(_) => None,
    }
}

pub fn eval(expr: &Expr) -> Option<Eval> {
    match expr {
        Expr::Bad => None,
        &Expr::Float(f) => Some(Eval::Float(f)),
        Expr::Name(_) => None, // placeholder
        Expr::Call(_) => None, // placeholder
        Expr::Bin(_) => None, // placeholder
        Expr::Neg(e) => evalf(e).map(Eval::Float),
        Expr::Factorial(_) => None,
        Expr::Circle(CircleExpr { x, y, r }) => {
            let x = evalf(x)?;
            let y = evalf(y)?;
            let r = evalf(r)?;
            Some(Eval::Circle(CircleEval { x, y, r }))
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

                Rule::eq  => BinOp::Eq,
                Rule::ne => BinOp::Ne,
                Rule::lt  => BinOp::Lt,
                Rule::le => BinOp::Le,
                Rule::gt  => BinOp::Gt,
                Rule::ge => BinOp::Ge,
                _ => unreachable!(),
            };
            Expr::Bin(BinExpr { op, left: Box::new(left), right: Box::new(right) })
        })
        .parse(pairs) // Execute the pratt parse
}

pub fn parse(input: &str) -> Result<Expr, pest::error::Error<Rule>> {
    let pairs = LeParser::parse(Rule::circle, input)?;
    let ast = parse_expr(pairs);
    Ok(ast)
}