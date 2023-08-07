use super::{consts::*, refs::Reference};
use serde_json::Value;
use std::collections::HashMap;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Missing schema ID")]
    MissingId(Value),
    #[error("Invalid sub-schema ID")]
    InvalidId(String),
    #[error("Invalid reference")]
    InvalidRef(#[from] super::refs::Error),
    #[error("Missing $defs in base schema")]
    MissingDefs(Value),
}

pub fn compose(base: &Value, sub_schemas: &[(Option<&str>, Value)]) -> Result<Value, Error> {
    let mut result = base.clone();
    let defs = result
        .get_mut(DEFS_KEY)
        .and_then(|value| value.as_object_mut())
        .ok_or_else(|| Error::MissingDefs(base.clone()))?;

    let mut prefixes = HashMap::new();

    for (prefix, sub_schema) in sub_schemas {
        let id = get_id(sub_schema)?;
        prefixes.insert(id, prefix.map(str::to_string));

        let (path_prefix, path_name) = match id.parse::<Reference>() {
            Ok(Reference::PathOnly {
                path_prefix,
                path_name,
            }) => Ok((path_prefix, path_name)),
            _ => Err(Error::InvalidId(id.to_string())),
        }?;

        let mut new_sub_schema = sub_schema.clone();

        modify_references(&mut new_sub_schema, &|old_reference| {
            Ok(match old_reference {
                Reference::FragmentOnly { fragment_name } => Some(Reference::new(
                    path_prefix.clone(),
                    path_name.clone(),
                    fragment_name.clone(),
                )),
                _ => None,
            })
        })?;

        if let Some(top_level_def) = get_top_level_def(&new_sub_schema) {
            defs.insert(
                format!("{}{}", prefix.unwrap_or_default(), path_name),
                top_level_def,
            );
        }

        if let Some(fields) = new_sub_schema
            .get(DEFS_KEY)
            .and_then(|value| value.as_object())
        {
            for (key, value) in fields {
                defs.insert(
                    format!("{}{}", prefix.unwrap_or_default(), key),
                    value.clone(),
                );
            }
        }
    }

    modify_references(&mut result, &|old_reference| {
        old_reference
            .path()
            .map(|value| {
                let prefix = prefixes
                    .get(&value.as_ref())
                    .ok_or_else(|| Error::InvalidId(old_reference.to_string()))?;

                Ok(Reference::from_fragment_name(format!(
                    "{}{}",
                    prefix
                        .as_ref()
                        .map(|value| value.as_str())
                        .unwrap_or_default(),
                    old_reference.name()
                )))
            })
            .map_or(Ok(None), |value| value.map(Some))
    })?;

    Ok(result)
}

fn get_id(value: &Value) -> Result<&str, Error> {
    value
        .get(ID_KEY)
        .and_then(|value| value.as_str())
        .ok_or_else(|| Error::MissingId(value.clone()))
}

fn get_top_level_def(value: &Value) -> Option<Value> {
    if let Some(fields) = value.as_object() {
        if fields.keys().any(|key| key != ID_KEY && key != DEFS_KEY) {
            let mut result = value.clone();
            let fields = result.as_object_mut().unwrap();
            fields.remove(DEFS_KEY);

            Some(result)
        } else {
            None
        }
    } else {
        None
    }
}

fn modify_references<F: Fn(&Reference) -> Result<Option<Reference>, Error>>(
    value: &mut Value,
    f: &F,
) -> Result<(), Error> {
    if let Some(values) = value.as_array_mut() {
        for value in values {
            modify_references(value, f)?;
        }
    } else if let Some(fields) = value.as_object_mut() {
        if let Some(reference) = fields.get_mut(REF_KEY) {
            if let Some(previous_value) = reference.as_str() {
                let previous_reference = previous_value.parse::<Reference>()?;

                if let Some(new_reference) = f(&previous_reference)? {
                    *reference = Value::String(new_reference.to_string());
                }
            }
        }

        for value in fields.values_mut() {
            modify_references(value, f)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn test_compose() {
        let base_schema = serde_json::from_str::<Value>(
            r###"
        {
            "type": "object",
            "properties": {
                "foo": {
                    "$ref": "/schemas/bar"
                },
                "baz": {
                    "$ref": "/schemas/qux#/$defs/oof"
                },
                "p": {
                    "$ref": "/schemas/prefixed#/$defs/prefixed_thing"
                }
            },
            "$defs": {}
        }
        "###,
        )
        .unwrap();

        let sub_schema_bar = serde_json::from_str::<Value>(
            r###"
        {
            "$id": "/schemas/bar",
            "type": "integer"
        }
        "###,
        )
        .unwrap();

        let sub_schema_qux = serde_json::from_str::<Value>(
            r###"
        {
            "$id": "/schemas/qux",
            "$defs": {
                "oof": {
                    "enum": ["ABC", "DEF"]
                }
            }
        }
        "###,
        )
        .unwrap();

        let sub_schema_top_level_and_defs = serde_json::from_str::<Value>(
            r###"
        {
            "$id": "/schemas/top_level",
            "enum": [1, 2, 3],            
            "$defs": {
                "top-level_stuff": {
                    "type": "boolean"
                }
            }
        }
        "###,
        )
        .unwrap();

        let sub_schema_prefixed = serde_json::from_str::<Value>(
            r###"
        {
            "$id": "/schemas/prefixed",
            "$defs": {
                "prefixed_thing": {
                    "type": "array",
                    "items": "number"
                }
            }
        }
        "###,
        )
        .unwrap();

        let expected = serde_json::from_str::<Value>(
            r###"
        {
            "type": "object",
            "properties": {
                "foo": {
                    "$ref": "#/$defs/bar"
                },
                "baz": {
                    "$ref": "#/$defs/oof"
                },
                "p": {
                    "$ref": "#/$defs/abcd_prefixed_thing"
                }
            },
            "$defs": {
                "bar": {
                    "$id": "/schemas/bar",
                    "type": "integer"
                },
                "oof": {
                    "enum": ["ABC", "DEF"]
                },
                "top_level": {
                    "$id": "/schemas/top_level",
                    "enum": [1, 2, 3]
                },
                "top-level_stuff": {
                    "type": "boolean"
                },
                "abcd_prefixed_thing": {
                    "type": "array",
                    "items": "number"
                }
            }
        }
        "###,
        )
        .unwrap();

        let composed = compose(
            &base_schema,
            &vec![
                (None, sub_schema_bar),
                (None, sub_schema_qux),
                (None, sub_schema_top_level_and_defs),
                (Some("abcd_"), sub_schema_prefixed),
            ],
        )
        .unwrap();

        assert_eq!(composed, expected);
    }
}
