//! Rule inheritance, as Vale 3.26.0 reads it.
//!
//! A rule whose `extends` holds a dot names another rule instead of a check
//! type. The child starts from the parent's fields and lays its own on top:
//! a bare key replaces the parent's value, `key+` adds to a list or a mapping,
//! and `key-` removes entries from one. The parent's `tests` never carry over.
//!
//! [`merged`] resolves the chain and writes the result as JSON text, which is
//! valid YAML, so the rule reads through the same reader as any other file.
//! It lays a config's parameters over the fields last, as a bare key replaces
//! a field of a rule.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;

use ctl_core::input::Input;
use serde::Deserialize;

use super::{LoadError, LoadProblem, Param};

/// How many rules a chain may hold before it is taken for a cycle.
const MAX_DEPTH: usize = 10;

/// Whether an `extends` value names a rule: a check type is one lowercase
/// word, and a rule name holds at least one dot.
pub(super) fn is_rule_ref(extends: &str) -> bool {
    extends.contains('.')
}

/// One YAML value, kept apart from any rule type so a rule file can be edited
/// before it is read as one.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(untagged)]
enum Node {
    Null(()),
    Bool(bool),
    Int(i64),
    Float(f64),
    Text(String),
    List(Vec<Node>),
    Map(BTreeMap<String, Node>),
}

/// The fields of one rule file.
type Fields = BTreeMap<String, Node>;

impl Node {
    /// The value as a person reads it in a message.
    fn shown(&self) -> String {
        match self {
            Self::Null(()) => "null".to_owned(),
            Self::Bool(value) => value.to_string(),
            Self::Int(value) => value.to_string(),
            Self::Float(value) => value.to_string(),
            Self::Text(value) => value.clone(),
            Self::List(_) => "a list".to_owned(),
            Self::Map(_) => "a mapping".to_owned(),
        }
    }

    /// The value as JSON text.
    ///
    /// # Errors
    ///
    /// A number JSON cannot hold: not finite.
    fn write(&self, out: &mut String) -> Result<(), String> {
        match self {
            Self::Null(()) => out.push_str("null"),
            Self::Bool(value) => out.push_str(if *value { "true" } else { "false" }),
            Self::Int(value) => out.push_str(&value.to_string()),
            Self::Float(value) if value.is_finite() => {
                let _ = write!(out, "{value:?}");
            }
            Self::Float(value) => return Err(format!("`{value}` is not a number a rule can hold")),
            Self::Text(value) => write_text(value, out),
            Self::List(items) => {
                out.push('[');
                for (index, item) in items.iter().enumerate() {
                    if index > 0 {
                        out.push(',');
                    }
                    item.write(out)?;
                }
                out.push(']');
            }
            Self::Map(map) => write_map(map, out)?,
        }
        Ok(())
    }
}

/// `value` as a JSON string.
fn write_text(value: &str, out: &mut String) {
    out.push('"');
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if u32::from(c) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", u32::from(c));
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

/// `map` as a JSON object.
fn write_map(map: &Fields, out: &mut String) -> Result<(), String> {
    out.push('{');
    for (index, (key, value)) in map.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        write_text(key, out);
        out.push(':');
        value.write(out)?;
    }
    out.push('}');
    Ok(())
}

/// A problem with a rule file, with the field it concerns and no line: the
/// merged text has none a person wrote.
fn problem(file: &str, field: &str, message: impl Into<String>) -> LoadProblem {
    LoadProblem {
        file: file.to_owned(),
        line: None,
        column: None,
        field: field.to_owned(),
        message: message.into(),
        unrunnable: false,
    }
}

/// The fields of the rule file `text`, named `file` in errors.
fn parse(file: &str, text: &str) -> Result<Fields, LoadProblem> {
    match Input::new(file, text).parse::<Node>() {
        Ok(Node::Map(fields)) => Ok(fields),
        Ok(_) => Err(problem(file, "", "a rule file is a mapping of fields")),
        Err(error) => Err(error.problems.into_iter().next().map_or_else(
            || problem(file, "", "is not YAML"),
            |first| LoadProblem {
                file: file.to_owned(),
                line: first.line,
                column: first.column,
                field: first.field,
                message: first.message,
                unrunnable: false,
            },
        )),
    }
}

/// The rule file `name` is: its first segment is a style folder of a package,
/// and every segment after it is a path component.
fn resolve(name: &str, search: &[PathBuf]) -> Result<PathBuf, String> {
    if name
        .split('.')
        .any(|part| part.is_empty() || part.contains(['/', '\\', ':', '\0']))
    {
        return Err(format!(
            "`{name}` is not a rule name: write the style, a dot, then the rule, such as `Std.Rule`, with no slash, colon or empty part between the dots"
        ));
    }
    let mut parts = name.split('.');
    let style = parts.next().unwrap_or_default();
    let relative = parts.collect::<Vec<_>>().join("/");
    let relative = format!("{relative}.yml");
    for package in search {
        let candidate = package.join(style).join(&relative);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    Err(format!(
        "`{name}` does not exist on the search path (looked for `{style}/{relative}`)"
    ))
}

impl From<&Param> for Node {
    fn from(param: &Param) -> Self {
        match param {
            Param::Bool(value) => Self::Bool(*value),
            Param::Int(value) => Self::Int(*value),
            Param::Float(value) => Self::Float(*value),
            Param::Text(value) => Self::Text(value.clone()),
        }
    }
}

/// `text`, the rule file `file`, with its chain of parents resolved against
/// the package folders in `search`, and each of `params` laid over a field of
/// the rule, as JSON text.
///
/// # Errors
///
/// The faults Vale refuses.
/// They are a parent that is missing, a cycle or a chain over ten rules, and
/// a `Vale.` parent.
/// They are also a key written both bare and with `+` or `-`, and an edit of
/// a field the parent lacks or of a value that is no list or mapping.
pub(super) fn merged(
    file: &str,
    text: &str,
    search: &[PathBuf],
    params: &[(String, Param)],
) -> Result<String, LoadError> {
    let fail = |problem| LoadError {
        problems: vec![problem],
    };
    let child = parse(file, text).map_err(fail)?;
    let mut fields = flatten(child, file, &mut Vec::new(), search).map_err(fail)?;
    for (name, param) in params {
        fields.insert(name.clone(), Node::from(param));
    }
    let mut out = String::new();
    write_map(&fields, &mut out).map_err(|message| fail(problem(file, "", message)))?;
    Ok(out)
}

/// `fields`, the rule file `file`, with the parent it names laid underneath,
/// and that parent's own.
fn flatten(
    fields: Fields,
    file: &str,
    seen: &mut Vec<String>,
    search: &[PathBuf],
) -> Result<Fields, LoadProblem> {
    let extends = match fields.get("extends") {
        Some(Node::Text(value)) if is_rule_ref(value) => value.clone(),
        _ => {
            for key in fields.keys() {
                if key.ends_with(['+', '-']) {
                    return Err(problem(
                        file,
                        key,
                        "needs a parent: only a rule that extends another rule can use `+` or `-`",
                    ));
                }
            }
            return Ok(fields);
        }
    };
    if extends.starts_with("Vale.") {
        return Err(problem(
            file,
            "extends",
            format!(
                "`{extends}` is a built-in rule generated at runtime; it has no file to extend"
            ),
        ));
    }
    if seen.len() >= MAX_DEPTH {
        return Err(problem(
            file,
            "extends",
            format!(
                "the inheritance chain is over {MAX_DEPTH} rules deep at `{extends}`; is there a cycle?"
            ),
        ));
    }
    if seen.contains(&extends) {
        return Err(problem(
            file,
            "extends",
            format!("inheritance cycle: `{extends}` is already in this chain"),
        ));
    }
    let path = resolve(&extends, search).map_err(|message| problem(file, "extends", message))?;
    let shown = path.display().to_string();
    let text = fs::read_to_string(&path).map_err(|error| problem(&shown, "", error.to_string()))?;
    let parent = parse(&shown, &text)?;
    seen.push(extends);
    let mut parent = flatten(parent, &shown, seen, search)?;
    seen.pop();
    // A parent's cases assert the parent's behaviour: a child that changes a
    // threshold or a token list brings its own.
    parent.remove("tests");
    overlay(parent, &fields, file)
}

/// `child` laid over `parent`.
fn overlay(parent: Fields, child: &Fields, file: &str) -> Result<Fields, LoadProblem> {
    let mut out = parent;
    for (key, value) in child {
        let base = key.trim_end_matches(['+', '-']);
        if base != key {
            if child.contains_key(base) {
                return Err(problem(
                    file,
                    key,
                    format!("both replaces `{base}` and edits it; pick one"),
                ));
            }
            let edited = edit(out.get(base), value, key, base)
                .map_err(|message| problem(file, key, message))?;
            out.insert(base.to_owned(), edited);
        } else if key != "extends" {
            out.insert(key.clone(), value.clone());
        }
    }
    Ok(out)
}

/// The value a `key+` or `key-` edit of `parent` gives.
fn edit(parent: Option<&Node>, value: &Node, key: &str, base: &str) -> Result<Node, String> {
    let add = key.ends_with('+');
    match parent {
        Some(Node::List(have)) => {
            let Node::List(entries) = value else {
                return Err("needs a list".to_owned());
            };
            if add {
                let mut out = have.clone();
                out.extend(entries.iter().cloned());
                return Ok(Node::List(out));
            }
            let mut out = have.clone();
            for entry in entries {
                let Some(at) = out.iter().position(|existing| existing == entry) else {
                    return Err(format!(
                        "the parent has no entry `{}` to remove",
                        entry.shown()
                    ));
                };
                out.remove(at);
            }
            Ok(Node::List(out))
        }
        Some(Node::Map(have)) => {
            if add {
                let Node::Map(entries) = value else {
                    return Err("needs a mapping".to_owned());
                };
                let mut out = have.clone();
                out.extend(entries.iter().map(|(k, v)| (k.clone(), v.clone())));
                return Ok(Node::Map(out));
            }
            let Node::List(names) = value else {
                return Err("needs a list of keys to remove".to_owned());
            };
            let mut out = have.clone();
            for name in names {
                let shown = name.shown();
                if out.remove(&shown).is_none() {
                    return Err(format!("the parent has no entry `{shown}` to remove"));
                }
            }
            Ok(Node::Map(out))
        }
        None => Err("the parent has no such field".to_owned()),
        Some(_) => Err(format!(
            "`{base}` is a single value; assign it directly instead"
        )),
    }
}
