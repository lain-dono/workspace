use super::{BinaryOp, Error, Expr, Func, Spanned, UnaryOp, Value};
use std::collections::HashMap;

pub struct Interpreter<'a> {
    pub funcs: &'a HashMap<String, Func>,
    pub stack: Vec<(String, Value)>,
}

impl<'a> Interpreter<'a> {
    pub fn new(funcs: &'a HashMap<String, Func>) -> Self {
        Self {
            funcs,
            stack: Vec::new(),
        }
    }

    pub fn eval(&mut self, expr: &Spanned<Expr>) -> Result<Value, Error> {
        Ok(match &expr.0 {
            Expr::Error => unreachable!(), // Error expressions only get created by parser errors, so cannot exist in a valid AST
            Expr::Value(val) => val.clone(),
            Expr::List(items) => Value::List(
                items
                    .iter()
                    .map(|item| self.eval(item))
                    .collect::<Result<_, _>>()?,
            ),
            Expr::Local(name) => self
                .stack
                .iter()
                .rev()
                .find(|(l, _)| l == name)
                .map(|(_, v)| v.clone())
                .or_else(|| {
                    Some(Value::Func(name.clone())).filter(|_| self.funcs.contains_key(name))
                })
                .ok_or_else(|| Error {
                    span: expr.1.clone(),
                    msg: format!("No such variable '{}' in scope", name),
                })?,
            Expr::Let(local, val, body) => {
                let val = self.eval(val)?;
                self.stack.push((local.clone(), val));
                let res = self.eval(body)?;
                self.stack.pop();
                res
            }
            Expr::Then(a, b) => {
                self.eval(a)?;
                self.eval(b)?
            }

            Expr::Unary(UnaryOp::NEG, op) => Value::Num(-self.eval(op)?.number(&op.1)?),
            Expr::Unary(_, op) => todo!(),

            Expr::Binary(lhs, op, rhs) => {
                let (l_value, r_value) = (self.eval(lhs)?, self.eval(rhs)?);
                let (l_span, r_span) = (&lhs.1, &rhs.1);

                match *op {
                    BinaryOp::ADD => Value::Num(l_value.number(l_span)? + r_value.number(r_span)?),
                    BinaryOp::SUB => Value::Num(l_value.number(l_span)? - r_value.number(r_span)?),
                    BinaryOp::MUL => Value::Num(l_value.number(l_span)? * r_value.number(r_span)?),
                    BinaryOp::QUO => Value::Num(l_value.number(l_span)? / r_value.number(r_span)?),
                    BinaryOp::REM => Value::Num(l_value.number(l_span)? % r_value.number(r_span)?),

                    BinaryOp::EQ => Value::Bool(l_value == r_value),
                    BinaryOp::NE => Value::Bool(l_value != r_value),

                    BinaryOp::LE => Value::Bool(l_value <= r_value),
                    BinaryOp::LT => Value::Bool(l_value < r_value),
                    BinaryOp::GE => Value::Bool(l_value >= r_value),
                    BinaryOp::GT => Value::Bool(l_value > r_value),

                    BinaryOp::LAND => {
                        Value::Bool(l_value.boolean(l_span)? && r_value.boolean(r_span)?)
                    }
                    BinaryOp::LIOR => {
                        Value::Bool(l_value.boolean(l_span)? || r_value.boolean(r_span)?)
                    }
                }
            }

            Expr::Call(func, args) => {
                let f = self.eval(func)?;
                match f {
                    Value::Func(name) => {
                        let f = &self.funcs[&name];
                        let stack = if f.args.len() != args.len() {
                            return Err(Error {
                            span: expr.1.clone(),
                            msg: format!("'{}' called with wrong number of arguments (expected {}, found {})", name, f.args.len(), args.len()),
                        });
                        } else {
                            f.args
                                .iter()
                                .zip(args.iter())
                                .map(|(name, arg)| Ok((name.clone(), self.eval(arg)?)))
                                .collect::<Result<_, _>>()?
                        };
                        let mut inner = Self {
                            funcs: self.funcs,
                            stack,
                        };
                        inner.eval(&f.body)?
                    }
                    f => {
                        return Err(Error {
                            span: func.1.clone(),
                            msg: format!("'{:?}' is not callable", f),
                        })
                    }
                }
            }
            Expr::If(cond, a, b) => {
                let c = self.eval(cond)?;
                match c {
                    Value::Bool(true) => self.eval(a)?,
                    Value::Bool(false) => self.eval(b)?,
                    c => {
                        return Err(Error {
                            span: cond.1.clone(),
                            msg: format!("Conditions must be booleans, found '{:?}'", c),
                        })
                    }
                }
            }
            Expr::Print(a) => {
                let val = self.eval(a)?;
                println!("{}", val);
                val
            }
        })
    }
}
