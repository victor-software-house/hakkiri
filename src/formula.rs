//! Formulas: the expression syntax of Tengo that a `metric` rule uses.
//!
//! A formula reads numbers, variables, parentheses, `+`, `-`, `*`, `/`, `%`,
//! comparisons, `&&`, `||`, `!`, the functions of Tengo's `math` module, and
//! `round`. An integer divides as an integer, as in Tengo, and a number with
//! a point is a float.

use std::f64::consts::{E, GOLDEN_RATIO, LN_2, LN_10, PI, SQRT_2};
use std::{error, fmt};

/// What a formula gives.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Value {
    /// A whole number.
    Int(i64),
    /// A number with a fraction.
    Float(f64),
    /// A truth value.
    Bool(bool),
}

impl Value {
    /// The number as a float, or `None` for a truth value.
    #[must_use]
    pub fn number(self) -> Option<f64> {
        match self {
            Self::Int(whole) => Some(int_to_float(whole)),
            Self::Float(number) => Some(number),
            Self::Bool(_) => None,
        }
    }

    /// Whether the value counts as false: `false`, 0 and not-a-number.
    fn falsy(self) -> bool {
        match self {
            Self::Int(whole) => whole == 0,
            Self::Float(number) => number.is_nan(),
            Self::Bool(truth) => !truth,
        }
    }
}

#[expect(
    clippy::as_conversions,
    clippy::cast_precision_loss,
    reason = "A whole number joins a float as Tengo does, rounded past 2^53."
)]
const fn int_to_float(whole: i64) -> f64 {
    whole as f64
}

/// Why a formula did not parse.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FormulaError {
    /// What is wrong, naming the byte where it was found.
    pub message: String,
}

impl fmt::Display for FormulaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl error::Error for FormulaError {}

/// Why a formula did not give a value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EvalError {
    /// A variable has no value for this text.
    Unavailable(String),
    /// An operator met a value it does not take, or divided by zero.
    Type(&'static str),
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unavailable(name) => write!(f, "`{name}` has no value for this text"),
            Self::Type(message) => f.write_str(message),
        }
    }
}

impl error::Error for EvalError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Not,
}

#[derive(Clone, Debug)]
enum Expr {
    Number(Value),
    Variable(String),
    Unary(Op, Box<Expr>),
    Binary(Op, Box<Expr>, Box<Expr>),
    Call(Function, Vec<Expr>),
}

/// A function a formula can call.
#[derive(Clone, Copy, Debug)]
enum Function {
    /// `round(x)` and `round(x, places)`.
    Round,
    One(One),
    Two(Two),
}

/// A function of one number.
type One = fn(f64) -> f64;

/// A function of two numbers.
type Two = fn(f64, f64) -> f64;

/// The functions of Tengo's `math` module that take one number.
const ONE: [(&str, One); 24] = [
    ("abs", f64::abs),
    ("acos", f64::acos),
    ("acosh", f64::acosh),
    ("asin", f64::asin),
    ("asinh", f64::asinh),
    ("atan", f64::atan),
    ("atanh", f64::atanh),
    ("cbrt", f64::cbrt),
    ("ceil", f64::ceil),
    ("cos", f64::cos),
    ("cosh", f64::cosh),
    ("exp", f64::exp),
    ("exp2", f64::exp2),
    ("expm1", f64::exp_m1),
    ("floor", f64::floor),
    ("log", f64::ln),
    ("log10", f64::log10),
    ("log1p", f64::ln_1p),
    ("log2", f64::log2),
    ("sin", f64::sin),
    ("sinh", f64::sinh),
    ("sqrt", f64::sqrt),
    ("tan", f64::tan),
    ("trunc", f64::trunc),
];

/// The functions of Tengo's `math` module that take two numbers.
const TWO: [(&str, Two); 6] = [
    ("atan2", f64::atan2),
    ("hypot", f64::hypot),
    ("max", f64::max),
    ("min", f64::min),
    ("mod", |x, y| x % y),
    ("pow", f64::powf),
];

/// The constants of Tengo's `math` module.
const CONSTANTS: [(&str, f64); 6] = [
    ("e", E),
    ("pi", PI),
    ("phi", GOLDEN_RATIO),
    ("sqrt2", SQRT_2),
    ("ln2", LN_2),
    ("ln10", LN_10),
];

#[derive(Clone, Debug, PartialEq)]
enum Token {
    Number(Value),
    Name(String),
    Op(Op),
    Open,
    Close,
    Comma,
    Dot,
}

/// A parsed formula.
#[derive(Clone, Debug)]
pub struct Formula {
    root: Expr,
    kind: Kind,
}

/// What a formula gives, as far as its shape shows.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Kind {
    Number,
    Truth,
    Either,
}

impl Formula {
    /// Parses `source`.
    ///
    /// `known` says whether a name is a variable.
    ///
    /// # Errors
    ///
    /// Returns a [`FormulaError`] when `source` is not one expression, names
    /// an unknown variable or function, or calls a function with the wrong
    /// number of arguments.
    pub fn parse(source: &str, known: &dyn Fn(&str) -> bool) -> Result<Self, FormulaError> {
        let tokens = lex(source)?;
        let mut parser = Parser {
            tokens,
            at: 0,
            known,
        };
        let root = parser.or()?;
        if let Some((token, offset)) = parser.tokens.get(parser.at) {
            return Err(error(
                *offset,
                &format!("unexpected {} after the expression", describe(token)),
            ));
        }
        let kind = kind_of(&root);
        Ok(Self { root, kind })
    }

    /// Whether the formula always gives a truth value.
    #[must_use]
    pub fn is_truth(&self) -> bool {
        self.kind == Kind::Truth
    }

    /// Whether the formula always gives a number.
    #[must_use]
    pub fn is_number(&self) -> bool {
        self.kind == Kind::Number
    }

    /// The value of the formula, with `variable` giving each variable.
    ///
    /// # Errors
    ///
    /// Returns an [`EvalError`] when a variable has no value, an operator
    /// meets a value it does not take, or an integer divides by zero.
    pub fn eval(&self, variable: &dyn Fn(&str) -> Option<f64>) -> Result<Value, EvalError> {
        eval(&self.root, variable)
    }
}

fn error(offset: usize, message: &str) -> FormulaError {
    FormulaError {
        message: format!("{message}, at byte {offset}"),
    }
}

fn describe(token: &Token) -> String {
    match token {
        Token::Number(Value::Int(whole)) => format!("`{whole}`"),
        Token::Number(Value::Float(number)) => format!("`{number}`"),
        Token::Number(Value::Bool(truth)) => format!("`{truth}`"),
        Token::Name(name) => format!("`{name}`"),
        Token::Op(_) => "an operator".to_owned(),
        Token::Open => "`(`".to_owned(),
        Token::Close => "`)`".to_owned(),
        Token::Comma => "`,`".to_owned(),
        Token::Dot => "`.`".to_owned(),
    }
}

/// The tokens of `source`, each with the byte it starts at.
fn lex(source: &str) -> Result<Vec<(Token, usize)>, FormulaError> {
    let mut tokens = Vec::new();
    let mut chars = source.char_indices().peekable();
    while let Some(&(start, c)) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
        } else if c.is_ascii_digit() {
            let mut end = start;
            let mut float = false;
            let mut exponent = false;
            while let Some(&(at, d)) = chars.peek() {
                let sign = exponent
                    && (d == '+' || d == '-')
                    && source
                        .get(..at)
                        .is_some_and(|before| before.ends_with(['e', 'E']));
                let continues = d.is_ascii_digit()
                    || (d == '.' && !float && !exponent)
                    || ((d == 'e' || d == 'E') && !exponent)
                    || sign;
                if !continues {
                    break;
                }
                float |= d == '.';
                if d == 'e' || d == 'E' {
                    exponent = true;
                    float = true;
                }
                end = at + d.len_utf8();
                chars.next();
            }
            let text = source.get(start..end).unwrap_or_default();
            let value = if float {
                text.parse().map(Value::Float).ok()
            } else {
                text.parse()
                    .map(Value::Int)
                    .ok()
                    .or_else(|| text.parse().map(Value::Float).ok())
            };
            let value = value.ok_or_else(|| error(start, &format!("`{text}` is not a number")))?;
            tokens.push((Token::Number(value), start));
        } else if c.is_alphabetic() || c == '_' {
            let mut end = start;
            while let Some(&(at, d)) = chars.peek() {
                if !(d.is_alphanumeric() || d == '_') {
                    break;
                }
                end = at + d.len_utf8();
                chars.next();
            }
            let name = source.get(start..end).unwrap_or_default().to_owned();
            tokens.push((Token::Name(name), start));
        } else {
            chars.next();
            let next = chars.peek().map(|&(_, d)| d);
            let mut two = |second: char, token: Token, single: Option<Token>| {
                if next == Some(second) {
                    chars.next();
                    Some(token)
                } else {
                    single
                }
            };
            let token = match c {
                '(' => Some(Token::Open),
                ')' => Some(Token::Close),
                ',' => Some(Token::Comma),
                '.' => Some(Token::Dot),
                '+' => Some(Token::Op(Op::Add)),
                '-' => Some(Token::Op(Op::Sub)),
                '*' => Some(Token::Op(Op::Mul)),
                '/' => Some(Token::Op(Op::Div)),
                '%' => Some(Token::Op(Op::Rem)),
                '=' => two('=', Token::Op(Op::Eq), None),
                '!' => two('=', Token::Op(Op::Ne), Some(Token::Op(Op::Not))),
                '<' => two('=', Token::Op(Op::Le), Some(Token::Op(Op::Lt))),
                '>' => two('=', Token::Op(Op::Ge), Some(Token::Op(Op::Gt))),
                '&' => two('&', Token::Op(Op::And), None),
                '|' => two('|', Token::Op(Op::Or), None),
                _ => None,
            };
            let token = token.ok_or_else(|| error(start, &format!("unexpected `{c}`")))?;
            tokens.push((token, start));
        }
    }
    Ok(tokens)
}

struct Parser<'k> {
    tokens: Vec<(Token, usize)>,
    at: usize,
    known: &'k dyn Fn(&str) -> bool,
}

impl Parser<'_> {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.at).map(|(token, _)| token)
    }

    fn here(&self) -> usize {
        self.tokens.get(self.at).map_or_else(
            || self.tokens.last().map_or(0, |(_, at)| *at + 1),
            |(_, at)| *at,
        )
    }

    /// Takes the next token when it is one of `ops`.
    fn operator(&mut self, ops: &[Op]) -> Option<Op> {
        if let Some(Token::Op(op)) = self.peek()
            && ops.contains(op)
        {
            let op = *op;
            self.at += 1;
            return Some(op);
        }
        None
    }

    fn binary(
        &mut self,
        ops: &[Op],
        next: fn(&mut Self) -> Result<Expr, FormulaError>,
    ) -> Result<Expr, FormulaError> {
        let mut left = next(self)?;
        while let Some(op) = self.operator(ops) {
            let right = next(self)?;
            left = Expr::Binary(op, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn or(&mut self) -> Result<Expr, FormulaError> {
        self.binary(&[Op::Or], Self::and)
    }

    fn and(&mut self) -> Result<Expr, FormulaError> {
        self.binary(&[Op::And], Self::comparison)
    }

    fn comparison(&mut self) -> Result<Expr, FormulaError> {
        self.binary(&[Op::Eq, Op::Ne, Op::Lt, Op::Le, Op::Gt, Op::Ge], Self::sum)
    }

    fn sum(&mut self) -> Result<Expr, FormulaError> {
        self.binary(&[Op::Add, Op::Sub], Self::product)
    }

    fn product(&mut self) -> Result<Expr, FormulaError> {
        self.binary(&[Op::Mul, Op::Div, Op::Rem], Self::unary)
    }

    fn unary(&mut self) -> Result<Expr, FormulaError> {
        if let Some(op) = self.operator(&[Op::Sub, Op::Add, Op::Not]) {
            let operand = self.unary()?;
            return Ok(if op == Op::Add {
                operand
            } else {
                Expr::Unary(op, Box::new(operand))
            });
        }
        self.primary()
    }

    fn expect(&mut self, token: &Token, what: &str) -> Result<(), FormulaError> {
        if self.peek() == Some(token) {
            self.at += 1;
            Ok(())
        } else {
            Err(error(self.here(), &format!("expected {what}")))
        }
    }

    fn primary(&mut self) -> Result<Expr, FormulaError> {
        let start = self.here();
        match self.tokens.get(self.at).map(|(token, _)| token.clone()) {
            Some(Token::Number(value)) => {
                self.at += 1;
                Ok(Expr::Number(value))
            }
            Some(Token::Open) => {
                self.at += 1;
                let inner = self.or()?;
                self.expect(&Token::Close, "`)`")?;
                Ok(inner)
            }
            Some(Token::Name(name)) => {
                self.at += 1;
                self.name(&name, start)
            }
            Some(other) => Err(error(
                start,
                &format!("expected a value, found {}", describe(&other)),
            )),
            None => Err(error(start, "expected a value, found the end")),
        }
    }

    /// What the name `name`, read at byte `start`, stands for: a variable, a
    /// constant or a call.
    fn name(&mut self, name: &str, start: usize) -> Result<Expr, FormulaError> {
        if name == "math" && self.peek() == Some(&Token::Dot) {
            self.at += 1;
            let Some(Token::Name(member)) = self.peek().cloned() else {
                return Err(error(self.here(), "expected a name after `math.`"));
            };
            self.at += 1;
            if self.peek() != Some(&Token::Open) {
                return CONSTANTS
                    .iter()
                    .find(|(constant, _)| *constant == member)
                    .map(|(_, value)| Expr::Number(Value::Float(*value)))
                    .ok_or_else(|| error(start, &format!("`math.{member}` is not a constant")));
            }
            let function = ONE
                .iter()
                .find(|(function, _)| *function == member)
                .map(|(_, call)| Function::One(*call))
                .or_else(|| {
                    TWO.iter()
                        .find(|(function, _)| *function == member)
                        .map(|(_, call)| Function::Two(*call))
                })
                .ok_or_else(|| error(start, &format!("`math.{member}` is not a function")))?;
            return self.call(function, start);
        }
        if self.peek() == Some(&Token::Open) {
            if name == "round" {
                return self.call(Function::Round, start);
            }
            return Err(error(start, &format!("`{name}` is not a function")));
        }
        if (self.known)(name) {
            Ok(Expr::Variable(name.to_owned()))
        } else {
            Err(error(start, &format!("`{name}` is not a variable")))
        }
    }

    fn call(&mut self, function: Function, start: usize) -> Result<Expr, FormulaError> {
        self.expect(&Token::Open, "`(`")?;
        let mut arguments = Vec::new();
        if self.peek() != Some(&Token::Close) {
            loop {
                arguments.push(self.or()?);
                if self.peek() == Some(&Token::Comma) {
                    self.at += 1;
                } else {
                    break;
                }
            }
        }
        self.expect(&Token::Close, "`)`")?;
        let wanted = match function {
            Function::Round => 1..=2,
            Function::One(_) => 1..=1,
            Function::Two(_) => 2..=2,
        };
        if wanted.contains(&arguments.len()) {
            Ok(Expr::Call(function, arguments))
        } else {
            Err(error(
                start,
                &format!(
                    "a call takes {} to {} arguments",
                    wanted.start(),
                    wanted.end()
                ),
            ))
        }
    }
}

fn kind_of(expr: &Expr) -> Kind {
    match expr {
        Expr::Number(Value::Bool(_)) | Expr::Unary(Op::Not, _) => Kind::Truth,
        Expr::Number(_) | Expr::Variable(_) | Expr::Call(..) | Expr::Unary(..) => Kind::Number,
        Expr::Binary(op, left, right) => match op {
            Op::Add | Op::Sub | Op::Mul | Op::Div | Op::Rem => Kind::Number,
            Op::Eq | Op::Ne | Op::Lt | Op::Le | Op::Gt | Op::Ge => Kind::Truth,
            Op::And | Op::Or | Op::Not => {
                if kind_of(left) == Kind::Truth && kind_of(right) == Kind::Truth {
                    Kind::Truth
                } else {
                    Kind::Either
                }
            }
        },
    }
}

fn eval(expr: &Expr, variable: &dyn Fn(&str) -> Option<f64>) -> Result<Value, EvalError> {
    match expr {
        Expr::Number(value) => Ok(*value),
        Expr::Variable(name) => variable(name)
            .map(Value::Float)
            .ok_or_else(|| EvalError::Unavailable(name.clone())),
        Expr::Unary(Op::Not, operand) => Ok(Value::Bool(eval(operand, variable)?.falsy())),
        Expr::Unary(_, operand) => match eval(operand, variable)? {
            Value::Int(whole) => whole
                .checked_neg()
                .map(Value::Int)
                .ok_or(EvalError::Type("an integer overflows")),
            Value::Float(number) => Ok(Value::Float(-number)),
            Value::Bool(_) => Err(EvalError::Type("`-` takes a number")),
        },
        Expr::Binary(Op::And, left, right) => {
            let first = eval(left, variable)?;
            if first.falsy() {
                Ok(first)
            } else {
                eval(right, variable)
            }
        }
        Expr::Binary(Op::Or, left, right) => {
            let first = eval(left, variable)?;
            if first.falsy() {
                eval(right, variable)
            } else {
                Ok(first)
            }
        }
        Expr::Binary(op, left, right) => apply(*op, eval(left, variable)?, eval(right, variable)?),
        Expr::Call(function, arguments) => {
            let numbers = arguments
                .iter()
                .map(|argument| {
                    eval(argument, variable)?
                        .number()
                        .ok_or(EvalError::Type("a function takes numbers"))
                })
                .collect::<Result<Vec<_>, _>>()?;
            match (function, numbers.as_slice()) {
                (Function::One(call), [x]) => Ok(Value::Float(call(*x))),
                (Function::Two(call), [x, y]) => Ok(Value::Float(call(*x, *y))),
                (Function::Round, [x]) => Ok(Value::Float(x.round())),
                (Function::Round, [x, places]) => {
                    let scale = 10_f64.powf(places.trunc());
                    Ok(Value::Float((x * scale).round() / scale))
                }
                _ => Err(EvalError::Type("a call has the wrong number of arguments")),
            }
        }
    }
}

#[expect(
    clippy::float_cmp,
    reason = "A formula compares floats exactly, as Tengo does."
)]
fn apply(op: Op, left: Value, right: Value) -> Result<Value, EvalError> {
    if let (Value::Bool(a), Value::Bool(b)) = (left, right) {
        return match op {
            Op::Eq => Ok(Value::Bool(a == b)),
            Op::Ne => Ok(Value::Bool(a != b)),
            Op::Add
            | Op::Sub
            | Op::Mul
            | Op::Div
            | Op::Rem
            | Op::Lt
            | Op::Le
            | Op::Gt
            | Op::Ge
            | Op::And
            | Op::Or
            | Op::Not => Err(EvalError::Type("an operator takes numbers")),
        };
    }
    if let (Value::Int(a), Value::Int(b)) = (left, right) {
        return apply_ints(op, a, b);
    }
    let (Some(a), Some(b)) = (left.number(), right.number()) else {
        return Err(EvalError::Type("an operator takes numbers"));
    };
    Ok(match op {
        Op::Add => Value::Float(a + b),
        Op::Sub => Value::Float(a - b),
        Op::Mul => Value::Float(a * b),
        Op::Div => Value::Float(a / b),
        Op::Rem => Value::Float(a % b),
        Op::Eq => Value::Bool(a == b),
        Op::Ne => Value::Bool(a != b),
        Op::Lt => Value::Bool(a < b),
        Op::Le => Value::Bool(a <= b),
        Op::Gt => Value::Bool(a > b),
        Op::Ge => Value::Bool(a >= b),
        Op::And | Op::Or | Op::Not => return Err(EvalError::Type("an operator takes numbers")),
    })
}

fn apply_ints(op: Op, a: i64, b: i64) -> Result<Value, EvalError> {
    let whole = |value: Option<i64>| {
        value
            .map(Value::Int)
            .ok_or(EvalError::Type("an integer overflows or divides by zero"))
    };
    match op {
        Op::Add => whole(a.checked_add(b)),
        Op::Sub => whole(a.checked_sub(b)),
        Op::Mul => whole(a.checked_mul(b)),
        Op::Div => whole(a.checked_div(b)),
        Op::Rem => whole(a.checked_rem(b)),
        Op::Eq => Ok(Value::Bool(a == b)),
        Op::Ne => Ok(Value::Bool(a != b)),
        Op::Lt => Ok(Value::Bool(a < b)),
        Op::Le => Ok(Value::Bool(a <= b)),
        Op::Gt => Ok(Value::Bool(a > b)),
        Op::Ge => Ok(Value::Bool(a >= b)),
        Op::And | Op::Or | Op::Not => Err(EvalError::Type("an operator takes numbers")),
    }
}
