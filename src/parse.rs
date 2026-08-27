use pest::{Parser, pratt_parser::{Assoc, Op, PrattParser}};
use pest_derive::Parser;


#[derive(Debug, Clone)]
pub enum Expr {
    Bad,
    Question,
    Float(f32),
    Name(String),
    Slate(SlateExpr),
    Arith(ArithExpr),
    Logic(LogicExpr),
    
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
pub struct LogicExpr {
    pub glue: Vec<LogicOp>,
    pub carts: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub enum LogicOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Debug, Clone)]
pub struct ArithExpr {
    pub op: ArithOp,
    pub left: Box<Expr>,
    pub right: Box<Expr>,
}

#[derive(Debug, Clone, Copy)]
pub enum ArithOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
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
            // .op(Op::infix(eq, Assoc::Left)
            //     | Op::infix(ne, Assoc::Left)
            //     | Op::infix(lt, Assoc::Left)
            //     | Op::infix(le, Assoc::Left)
            //     | Op::infix(gt, Assoc::Left)
            //     | Op::infix(ge, Assoc::Left))
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
            Rule::question => Expr::Question,
            // If it's parentheses, we evaluate the inner expression
            Rule::expr => parse_expr(primary.into_inner()),
            
            Rule::train => {
                let mut glue = Vec::new();
                let mut carts = Vec::new();

                let mut args = primary.into_inner();
                let Some(first) = args.next() else {
                    return Expr::Bad;
                };
                let first = parse_expr(first.into_inner());
                carts.push(first);
                while !args.is_empty() {
                    let Some(op) = args.next() else {
                        return Expr::Bad;
                    };
                    let op = match op.as_rule() {
                        Rule::eq  => {
                            // haxy (maybe I'll impl references...)
                            // if let Expr::Name(name) = left {
                            //     return Expr::Define(DefineExpr { name, val: Box::new(right), })
                            // }
                            // todo calls
                            // else if let Expr::Call(call) = left {
                            //     return Expr::Define(DefineExpr { name: format!("{:?}", call.callee), val: Box::new(right), })
                            // }

                            LogicOp::Eq
                        },
                        Rule::ne => LogicOp::Ne,
                        Rule::lt  => LogicOp::Lt,
                        Rule::le => LogicOp::Le,
                        Rule::gt  => LogicOp::Gt,
                        Rule::ge => LogicOp::Ge,
                        _ => return Expr::Bad,
                    };
                    glue.push(op);

                    let Some(cart) = args.next() else {
                        return Expr::Bad;
                    };
                    let cart = parse_expr(cart.into_inner());
                    carts.push(cart);
                }

                Expr::Logic(LogicExpr { glue, carts, })
            }
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
                Rule::add => ArithOp::Add,
                Rule::sub => ArithOp::Sub,
                Rule::mul => ArithOp::Mul,
                Rule::div => ArithOp::Div,
                Rule::pow => ArithOp::Pow,

                Rule::arrow => {
                    // haxy (maybe I'll impl references...)
                    if let Expr::Name(name) = left {
                        return Expr::Assign(AssignExpr { name, val: Box::new(right), })
                    }

                    return Expr::Bad;
                },
                _ => unreachable!(),
            };

            Expr::Arith(ArithExpr { op, left: Box::new(left), right: Box::new(right) })
        })
        .parse(pairs) // Execute the pratt parse
}

pub fn parse(input: &str) -> Result<Expr, pest::error::Error<Rule>> {
    let mut pairs = LeParser::parse(Rule::line, input)?;
    let pairs = pairs.next().unwrap().into_inner();
    let ast = parse_expr(pairs);
    Ok(ast)
}