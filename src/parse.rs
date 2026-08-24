use pest::{Parser, pratt_parser::{Assoc, Op, PrattParser}};
use pest_derive::Parser;


#[derive(Debug, Clone)]
pub enum Expr {
    Bad,
    Float(f32),
    Name(String),
    Slate(SlateExpr),
    Bin(BinExpr),
    
    Neg(Box<Expr>),
    Factorial(Box<Expr>),
    Circle(CircleExpr),
    Define(DefineExpr),
    Assign(AssignExpr),
}

#[derive(Debug, Clone)]
pub struct SlateExpr {
    pub inner: Box<Expr>,
    pub args: Vec<(String, Expr)>,
}

#[derive(Debug, Clone)]
pub struct BinExpr {
    pub op: BinOp,
    pub left: Box<Expr>,
    pub right: Box<Expr>,
}

#[derive(Debug, Clone, Copy)]
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


#[derive(Debug, Clone)]
pub struct CircleExpr {
    pub x: Box<Expr>,
    pub y: Box<Expr>,
    pub r: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct DefineExpr {
    pub name: String,
    pub val: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct AssignExpr {
    pub name: String,
    pub val: Box<Expr>,
}

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct LeParser;

lazy_static::lazy_static! {
    static ref PRATT_PARSER: PrattParser<Rule> = {
        use Rule::*;
        
        PrattParser::new()
            .op(Op::infix(eq, Assoc::Left)
                | Op::infix(ne, Assoc::Left)
                | Op::infix(lt, Assoc::Left)
                | Op::infix(le, Assoc::Left)
                | Op::infix(gt, Assoc::Left)
                | Op::infix(ge, Assoc::Left))
            .op(Op::infix(arrow, Assoc::Left))
            .op(Op::infix(add, Assoc::Left) | Op::infix(sub, Assoc::Left))
            .op(Op::infix(mul, Assoc::Left) | Op::infix(div, Assoc::Left))
            .op(Op::infix(pow, Assoc::Left))
            .op(Op::prefix(neg))
            .op(Op::postfix(factorial) | Op::postfix(slate))
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
            Rule::factorial => Expr::Factorial(Box::new(left)),
            Rule::slate => {
                let mut args = Vec::new();
                for slatom in op.into_inner() {
                    let mut inner = slatom.into_inner();
                    let name = inner.next().unwrap().to_string();
                    let val = parse_expr(inner.next().unwrap().into_inner());
                    args.push((name, val));
                }
                
                Expr::Slate(SlateExpr { inner: Box::new(left), args })
            }
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
                    // haxy (maybe I'll impl references...)
                    if let Expr::Name(name) = left {
                        return Expr::Define(DefineExpr { name, val: Box::new(right), })
                    }
                    // todo calls
                    // else if let Expr::Call(call) = left {
                    //     return Expr::Define(DefineExpr { name: format!("{:?}", call.callee), val: Box::new(right), })
                    // }

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