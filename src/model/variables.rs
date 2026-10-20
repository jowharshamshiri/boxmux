//! Variable substitution in configuration strings.
//!
//! `${NAME}` is replaced by the value of `NAME`, and `${NAME:default}` falls back to `default`
//! (which may itself contain references) when `NAME` is not defined. Values are looked up from the innermost box outwards, then the
//! layout, the app, and finally the environment. `$${` writes a literal `${`.
//!
//! Anything else that starts with `${` is shell syntax (`${i}` of an undefined variable,
//! `${FOO:-x}`, `${#list[@]}`, `${file%.txt}`) and is left exactly as written, so scripts reach
//! the shell intact. A reference to a name that is not defined anywhere and has no default is
//! also left as written: in a script the shell resolves it at run time.

use std::collections::HashMap;

/// One scope of variable definitions, innermost first when stacked.
pub type Scope = HashMap<String, String>;

/// Resolves variable names through a stack of scopes, then the process environment.
pub struct Resolver<'a> {
    scopes: Vec<&'a Scope>,
    environment: &'a dyn Fn(&str) -> Option<String>,
}

impl<'a> Resolver<'a> {
    /// `scopes` are searched in order, so pass the innermost scope first.
    pub fn new(scopes: Vec<&'a Scope>, environment: &'a dyn Fn(&str) -> Option<String>) -> Self {
        Resolver {
            scopes,
            environment,
        }
    }

    fn lookup(&self, name: &str) -> Option<String> {
        self.scopes
            .iter()
            .find_map(|scope| scope.get(name).cloned())
            .or_else(|| (self.environment)(name))
    }

    /// Substitutes every reference in `text`. Substituted values are not scanned again, so a
    /// value containing `${...}` is inserted verbatim.
    pub fn substitute(&self, text: &str) -> String {
        let mut out = String::with_capacity(text.len());
        let mut rest = text;
        while let Some(pos) = rest.find('$') {
            out.push_str(&rest[..pos]);
            let after = &rest[pos..];
            if let Some(escaped) = after.strip_prefix("$${") {
                out.push_str("${");
                rest = escaped;
                continue;
            }
            match parse_reference(after) {
                Some(reference) => {
                    match self
                        .lookup(reference.name)
                        .or_else(|| reference.default.map(|d| self.substitute(d)))
                    {
                        Some(value) => out.push_str(&value),
                        None => out.push_str(&after[..reference.len]),
                    }
                    rest = &after[reference.len..];
                }
                None => {
                    out.push('$');
                    rest = &after[1..];
                }
            }
        }
        out.push_str(rest);
        out
    }
}

/// Converts a `variables` mapping from the YAML document into a scope. Values may be strings,
/// numbers or booleans; anything else is an error naming the variable.
fn scope_from_value(value: &serde_json::Value, location: &str) -> Result<Scope, String> {
    let mapping = match value {
        serde_json::Value::Object(mapping) => mapping,
        serde_json::Value::Null => return Ok(Scope::new()),
        _ => {
            return Err(format!(
                "{}.variables must be a mapping of names to values",
                location
            ))
        }
    };
    mapping
        .iter()
        .map(|(name, value)| {
            let text = scalar_text(value).ok_or_else(|| {
                format!(
                    "{}.variables.{} must be a string, number or boolean",
                    location, name
                )
            })?;
            Ok((name.to_string(), text))
        })
        .collect()
}

fn scalar_text(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(s) => Some(s.clone()),
        serde_json::Value::Number(n) => Some(n.to_string()),
        serde_json::Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

/// Deserializes a `variables` field: names mapped to strings, numbers or booleans.
pub fn deserialize_scope<'de, D>(deserializer: D) -> Result<Option<Scope>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    let value = Option::<serde_json::Value>::deserialize(deserializer)?;
    value
        .map(|value| scope_from_value(&value, "variables").map_err(serde::de::Error::custom))
        .transpose()
}

/// Substitutes variables in every string value of a configuration document, with the scopes
/// that apply where the value appears: a box sees its own variables, then those of the boxes
/// around it, its layout and the app. Mapping keys and the `variables` mappings themselves are
/// not substituted.
pub fn substitute_document(
    document: &mut serde_json::Value,
    environment: &dyn Fn(&str) -> Option<String>,
) -> Result<(), String> {
    let Some(app) = document.get_mut("app") else {
        return Ok(());
    };
    let app_scope = match app.get("variables") {
        Some(variables) => scope_from_value(variables, "app")?,
        None => Scope::new(),
    };
    let mut scopes = vec![app_scope];
    substitute_node(app, &mut scopes, "app", environment)
}

/// Walks `node`, whose own `variables` (if any) are already the last entry of `scopes`.
fn substitute_node(
    node: &mut serde_json::Value,
    scopes: &mut Vec<Scope>,
    location: &str,
    environment: &dyn Fn(&str) -> Option<String>,
) -> Result<(), String> {
    let serde_json::Value::Object(mapping) = node else {
        return Ok(());
    };
    for (key, value) in mapping.iter_mut() {
        match key.as_str() {
            "variables" => {}
            "layouts" | "children" => {
                let serde_json::Value::Array(items) = value else {
                    continue;
                };
                for (index, item) in items.iter_mut().enumerate() {
                    let item_location = format!("{}.{}[{}]", location, key, index);
                    let scope = match item.get("variables") {
                        Some(variables) => scope_from_value(variables, &item_location)?,
                        None => Scope::new(),
                    };
                    scopes.push(scope);
                    let result = substitute_node(item, scopes, &item_location, environment);
                    scopes.pop();
                    result?;
                }
            }
            _ => substitute_strings(value, scopes, environment),
        }
    }
    Ok(())
}

/// Substitutes in every string inside `value` (nested sequences and mappings included).
fn substitute_strings(
    value: &mut serde_json::Value,
    scopes: &[Scope],
    environment: &dyn Fn(&str) -> Option<String>,
) {
    match value {
        serde_json::Value::String(text) => {
            let innermost_first: Vec<&Scope> = scopes.iter().rev().collect();
            *text = Resolver::new(innermost_first, environment).substitute(text);
        }
        serde_json::Value::Array(items) => {
            for item in items {
                substitute_strings(item, scopes, environment);
            }
        }
        serde_json::Value::Object(mapping) => {
            for (_, item) in mapping.iter_mut() {
                substitute_strings(item, scopes, environment);
            }
        }
        serde_json::Value::Null | serde_json::Value::Bool(_) | serde_json::Value::Number(_) => {}
    }
}

struct Reference<'t> {
    name: &'t str,
    default: Option<&'t str>,
    /// Length of the whole reference, `${` through `}`.
    len: usize,
}

/// Parses `${NAME}` or `${NAME:default}` at the start of `text`. Returns `None` for anything
/// else, including the shell's `${NAME:-x}`, `${NAME:=x}`, `${NAME:?x}` and `${NAME:+x}`.
fn parse_reference(text: &str) -> Option<Reference<'_>> {
    let body = text.strip_prefix("${")?;
    let name_len = body
        .char_indices()
        .take_while(|(i, c)| c.is_ascii_alphabetic() || *c == '_' || (*i > 0 && c.is_ascii_digit()))
        .count();
    if name_len == 0 {
        return None;
    }
    let name = &body[..name_len];
    let after_name = &body[name_len..];
    if after_name.starts_with('}') {
        return Some(Reference {
            name,
            default: None,
            len: 2 + name_len + 1,
        });
    }
    let default_text = after_name.strip_prefix(':')?;
    if default_text.starts_with(['-', '=', '?', '+']) {
        return None;
    }
    // The default may itself contain references; its end is the brace that closes this one.
    let mut depth = 0usize;
    let mut close = None;
    let bytes = default_text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'$' if bytes.get(i + 1) == Some(&b'{') => {
                depth += 1;
                i += 2;
                continue;
            }
            b'}' if depth == 0 => {
                close = Some(i);
                break;
            }
            b'}' => depth -= 1,
            _ => {}
        }
        i += 1;
    }
    let close = close?;
    let default = &default_text[..close];
    Some(Reference {
        name,
        default: Some(default),
        len: 2 + name_len + 1 + close + 1,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn no_env(_: &str) -> Option<String> {
        None
    }

    fn scope(pairs: &[(&str, &str)]) -> Scope {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn substitutes_defined_variables_and_defaults() {
        let vars = scope(&[("HOST", "db1")]);
        let r = Resolver::new(vec![&vars], &no_env);
        assert_eq!(r.substitute("ping ${HOST}"), "ping db1");
        assert_eq!(r.substitute("port ${PORT:5432}"), "port 5432");
        assert_eq!(r.substitute("${HOST:ignored}"), "db1");
        assert_eq!(r.substitute("empty default:${NOPE:}."), "empty default:.");
    }

    #[test]
    fn inner_scope_wins_over_outer_and_environment() {
        let inner = scope(&[("NAME", "inner")]);
        let outer = scope(&[("NAME", "outer"), ("ONLY_OUTER", "o")]);
        let env = |name: &str| (name == "NAME").then(|| "env".to_string());
        let r = Resolver::new(vec![&inner, &outer], &env);
        assert_eq!(r.substitute("${NAME} ${ONLY_OUTER}"), "inner o");

        let r = Resolver::new(vec![], &env);
        assert_eq!(r.substitute("${NAME}"), "env");
    }

    #[test]
    fn shell_syntax_is_left_for_the_shell() {
        let vars = scope(&[("FOO", "bar")]);
        let r = Resolver::new(vec![&vars], &no_env);
        for script in [
            "for i in 1 2; do echo ${i}; done",
            "echo ${FOO:-fallback}",
            "echo ${FOO:=x} ${FOO:?missing} ${FOO:+set}",
            "echo ${#items[@]} ${items[0]}",
            "mv \"$f\" \"${f%.txt}.md\"",
            "echo $HOME $1 $$",
        ] {
            assert_eq!(r.substitute(script), script, "changed: {}", script);
        }
    }

    #[test]
    fn undefined_reference_without_default_is_kept() {
        let r = Resolver::new(vec![], &no_env);
        assert_eq!(r.substitute("echo ${UNSET_VAR}"), "echo ${UNSET_VAR}");
    }

    #[test]
    fn double_dollar_escapes_a_reference() {
        let vars = scope(&[("X", "1")]);
        let r = Resolver::new(vec![&vars], &no_env);
        assert_eq!(r.substitute("$${X} is ${X}"), "${X} is 1");
    }

    #[test]
    fn substituted_values_are_not_expanded_again() {
        let vars = scope(&[("A", "${B}"), ("B", "nope")]);
        let r = Resolver::new(vec![&vars], &no_env);
        assert_eq!(r.substitute("${A}"), "${B}");
    }

    #[test]
    fn unterminated_or_nested_references_are_left_alone() {
        let vars = scope(&[("A", "1")]);
        let r = Resolver::new(vec![&vars], &no_env);
        assert_eq!(r.substitute("${A"), "${A");
        assert_eq!(
            r.substitute("${X:-${A}}"),
            "${X:-1}",
            "shell syntax around a reference"
        );
        assert_eq!(r.substitute("${}"), "${}");
        assert_eq!(r.substitute("price: $5"), "price: $5");
    }

    #[test]
    fn defaults_may_contain_references() {
        let vars = scope(&[("A", "1")]);
        let r = Resolver::new(vec![&vars], &no_env);
        assert_eq!(r.substitute("${X:${A}}"), "1");
        assert_eq!(
            r.substitute("${A:${X}}"),
            "1",
            "a defined name ignores its default"
        );
        assert_eq!(r.substitute("${X:${Y:deep}}!"), "deep!");
        assert_eq!(
            r.substitute("${X:${Y}}"),
            "${Y}",
            "an undefined default stays as written"
        );
    }

    #[test]
    fn non_ascii_text_around_references_is_preserved() {
        let vars = scope(&[("NAME", "Zoë")]);
        let r = Resolver::new(vec![&vars], &no_env);
        assert_eq!(r.substitute("→ ${NAME} ✓"), "→ Zoë ✓");
    }
}
