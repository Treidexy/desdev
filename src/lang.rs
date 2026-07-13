use pest::{Parser, pratt_parser::{Assoc, Op, PrattParser}};
use pest_derive::Parser;

#[derive(Debug)]
pub enum Eval {
    Float(f32),
    Circle(CircleEval),
    Define(DefineEval),
    Assign(AssignEval),
}

#[derive(Debug)]
pub enum Expr {
    Bad,
    Float(f32),
    Name(String),
    Call(CallExpr),
    Arith(ArithExpr),
    Logic(LogicExpr),
    
    Neg(Box<Expr>),
    Factorial(Box<Expr>),
    Circle(CircleExpr),
    Define(DefineExpr),
    Assign(AssignExpr),
}

#[derive(Debug)]
pub struct CallExpr {
    pub callee: Box<Expr>,
    pub args: Vec<Expr>, // todo: named args
}

#[derive(Debug)]
pub struct ArithExpr {
    pub op: ArithOp,
    pub left: Box<Expr>,
    pub right: Box<Expr>,
}

#[derive(Debug)]
pub enum ArithOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
}

#[derive(Debug)]
pub struct LogicExpr {
    pub op: LogicOp,
    pub left: Box<Expr>,
    pub right: Box<Expr>,
}

#[derive(Debug)]
pub enum LogicOp {
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
pub struct DefineExpr {
    pub name: String,
    pub val: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct DefineEval {
    pub name: String,
    pub val: f32,
}

#[derive(Debug)]
pub struct AssignExpr {
    pub name: String,
    pub val: Box<Expr>,
}

#[derive(Debug, Clone)]
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
                Rule::add => ArithOp::Add,
                Rule::sub => ArithOp::Sub,
                Rule::mul => ArithOp::Mul,
                Rule::div => ArithOp::Div,
                Rule::pow => ArithOp::Pow,

                Rule::eq  => {
                    // haxy (maybe I'll impl references...)
                    if let Expr::Name(name) = left {
                        return Expr::Define(DefineExpr { name, val: Box::new(right), })
                    }

                    BinOp::Eq
                },
                Rule::ne => BinOp::Ne,
                Rule::lt  => BinOp::Lt,
                Rule::le => BinOp::Le,
                Rule::gt  => BinOp::Gt,
                Rule::ge => BinOp::Ge,

                Rule::arrow => {
                    // haxy (maybe I'll impl references...)
                    if let Expr::Name(name) = left {
                        return Expr::Assign(AssignExpr { name, val: Box::new(right), })
                    }

                    BinOp::Arrow
                },
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