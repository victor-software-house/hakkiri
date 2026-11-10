//! Formulas: the expression syntax of a `metric` rule.

#![expect(
    clippy::panic_in_result_fn,
    reason = "A test returns Result for `?` and fails through an assertion."
)]

use std::error::Error;

use hakkiri::formula::{EvalError, Formula, Value};

type TestResult = Result<(), Box<dyn Error>>;

/// The value of `source`, where `x` is 7 and `y` is 2.
fn value(source: &str) -> Result<Value, Box<dyn Error>> {
    let known = |name: &str| name == "x" || name == "y";
    let formula = Formula::parse(source, &known)?;
    Ok(formula.eval(&|name| match name {
        "x" => Some(7.0),
        "y" => Some(2.0),
        _ => None,
    })?)
}

#[test]
fn operators_bind_as_in_tengo() -> TestResult {
    assert_eq!(value("1 + 2 * 3")?, Value::Int(7));
    assert_eq!(value("(1 + 2) * 3")?, Value::Int(9));
    assert_eq!(value("-2 * -3")?, Value::Int(6));
    assert_eq!(value("x / y")?, Value::Float(3.5));
    assert_eq!(value("1 < 2 && 2 < 3 || 0 > 1")?, Value::Bool(true));
    assert_eq!(value("!(x > y)")?, Value::Bool(false));
    Ok(())
}

#[test]
fn integers_divide_as_integers() -> TestResult {
    assert_eq!(value("7 / 2")?, Value::Int(3));
    assert_eq!(value("7 % 4")?, Value::Int(3));
    assert_eq!(value("7.0 / 2")?, Value::Float(3.5));
    Ok(())
}

#[test]
fn an_integer_cannot_divide_by_zero() {
    assert!(value("1 / 0").is_err());
}

#[test]
fn functions_read_as_in_tengo() -> TestResult {
    assert_eq!(value("math.sqrt(16)")?, Value::Float(4.0));
    assert_eq!(value("math.max(x, y)")?, Value::Float(7.0));
    assert_eq!(value("round(2.5)")?, Value::Float(3.0));
    assert_eq!(value("round(x / 3, 1)")?, Value::Float(2.3));
    Ok(())
}

#[test]
fn a_logical_operator_gives_an_operand() -> TestResult {
    assert_eq!(value("0 || 5")?, Value::Int(5));
    assert_eq!(value("3 && 5")?, Value::Int(5));
    Ok(())
}

#[test]
fn what_cannot_parse_is_refused() {
    let known = |name: &str| name == "x";
    for source in [
        "x +",
        "(x",
        "x x",
        "y",
        "frobnicate(x)",
        "math.sqrt()",
        "math.nothing(x)",
        "round(1, 2, 3)",
        "x $ 2",
    ] {
        assert!(Formula::parse(source, &known).is_err(), "`{source}` parsed");
    }
}

#[test]
fn a_variable_with_no_value_is_unavailable() -> TestResult {
    let formula = Formula::parse("x", &|name| name == "x")?;
    assert_eq!(
        formula.eval(&|_| None),
        Err(EvalError::Unavailable("x".to_owned()))
    );
    Ok(())
}
