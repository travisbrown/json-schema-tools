use super::path::Path;
use serde_json::Value;

/// Collect all JSON values with their path (depth-first)
pub fn nodes_with_path(value: &Value) -> Vec<(Path, &Value)> {
    nodes_with_path_rec(value, Path::default(), vec![])
}

fn nodes_with_path_rec<'a>(
    value: &'a Value,
    path: Path<'a>,
    mut acc: Vec<(Path<'a>, &'a Value)>,
) -> Vec<(Path<'a>, &'a Value)> {
    acc.push((path.clone(), value));

    if let Some(values) = value.as_array() {
        for (i, child) in values.iter().enumerate() {
            let mut child_path = path.clone();
            child_path.push(i);
            acc = nodes_with_path_rec(child, child_path, acc);
        }
    } else if let Some(fields) = value.as_object() {
        for (key, child) in fields {
            let mut child_path = path.clone();
            child_path.push(key.as_ref());
            acc = nodes_with_path_rec(child, child_path, acc);
        }
    }

    acc
}
