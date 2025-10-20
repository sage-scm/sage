use toml::value::{Table, Value};

pub(crate) fn insert_value(root: &mut Table, path: &[String], value: Value) {
    if path.is_empty() {
        return;
    }

    if path.len() == 1 {
        root.insert(path[0].clone(), value);
        return;
    }

    let entry = root
        .entry(path[0].clone())
        .or_insert_with(|| Value::Table(Table::new()));

    match entry {
        Value::Table(table) => insert_value(table, &path[1..], value),
        _ => {
            let mut table = Table::new();
            insert_value(&mut table, &path[1..], value);
            *entry = Value::Table(table);
        }
    }
}

pub(crate) fn remove_value(root: &mut Table, path: &[String]) -> bool {
    if path.is_empty() {
        return false;
    }

    if path.len() == 1 {
        return root.remove(&path[0]).is_some();
    }

    let key = &path[0];
    if let Some(Value::Table(child)) = root.get_mut(key) {
        let removed = remove_value(child, &path[1..]);
        if removed && child.is_empty() {
            root.remove(key);
        }
        removed
    } else {
        false
    }
}

pub(crate) fn parse_scalar_value(raw: &str) -> Value {
    let trimmed = raw.trim();
    if trimmed.eq_ignore_ascii_case("true") {
        Value::Boolean(true)
    } else if trimmed.eq_ignore_ascii_case("false") {
        Value::Boolean(false)
    } else if let Ok(int) = trimmed.parse::<i64>() {
        Value::Integer(int)
    } else if let Ok(float) = trimmed.parse::<f64>() {
        Value::Float(float)
    } else {
        Value::String(raw.to_string())
    }
}
