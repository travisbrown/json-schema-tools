use super::schema::SchemaFile;
use serde_json::Value;

#[derive(Debug)]
pub enum Issue<'a> {
    Json(serde_json::Error),
    MisorderedKeys(super::key_order::KeyOrderMismatch<'a>),
    UnrestrictedProperties(Vec<String>),
    OptionalField(Vec<String>, String),
    MisorderedRequires(Vec<String>),
}

pub fn lint(schema_file_value: &Value) -> Vec<Issue> {
    let mut result = vec![];

    for key_order_mismatch in super::key_order::check_key_order(schema_file_value) {
        result.push(Issue::MisorderedKeys(key_order_mismatch));
    }

    match serde_json::from_value::<SchemaFile>(schema_file_value.clone()) {
        Ok(schema_file) => {
            for (path, object) in schema_file.objects() {
                if object.additional_properties {
                    result.push(Issue::UnrestrictedProperties(path.clone()));
                }

                let mut optional_fields = object.properties.keys().collect::<Vec<_>>();
                optional_fields.retain(|value| !object.required.contains(value));

                for optional_field in optional_fields {
                    result.push(Issue::OptionalField(path.clone(), optional_field.clone()));
                }

                let mut required_fields = object.properties.keys().collect::<Vec<_>>();
                required_fields.retain(|value| object.required.contains(value));

                if required_fields != object.required.iter().collect::<Vec<_>>() {
                    result.push(Issue::MisorderedRequires(path.clone()));
                }
            }
        }
        Err(error) => {
            result.push(Issue::Json(error));
        }
    }

    result
}
