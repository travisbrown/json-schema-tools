use indexmap::map::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct File {
    #[serde(flatten)]
    pub metadata: Metadata,
    #[serde(flatten)]
    pub schema: Option<TypeDef>,
    #[serde(rename = "$defs")]
    pub definitions: Option<IndexMap<String, Type>>,
}

impl File {
    pub fn objects(&self) -> Vec<(Vec<String>, Object)> {
        let mut result = vec![];

        if let Some(schema) = &self.schema {
            objects_rec(&Type::new(schema), &[], &mut result);
        }

        if let Some(definitions) = &self.definitions {
            for (key, value) in definitions {
                objects_rec(value, &[key.clone()], &mut result);
            }
        }

        result
    }
}

fn objects_rec(schema: &Type, path: &[String], acc: &mut Vec<(Vec<String>, Object)>) {
    match &schema.schema {
        TypeDef::NamedType(NamedType::Array { items }) => {
            let mut new_path = path.to_vec();
            new_path.push("array".to_string());

            objects_rec(items, &new_path, acc);
        }
        TypeDef::NamedType(other) => {
            if let Some(object) = other.as_object() {
                acc.push((path.to_vec(), object.clone()));

                for (key, value) in &object.properties {
                    let mut new_path = path.to_vec();
                    new_path.push(key.clone());

                    objects_rec(value, &new_path, acc);
                }
            }
        }
        TypeDef::OneOf { value } => {
            for (i, schema) in value.iter().enumerate() {
                let mut new_path = path.to_vec();
                new_path.push(format!("oneOf[{}]", i));

                objects_rec(schema, &new_path, acc);
            }
        }
        _ => {}
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Type {
    #[serde(flatten)]
    pub metadata: Metadata,
    #[serde(flatten)]
    pub schema: TypeDef,
}

impl Type {
    fn new(schema: &TypeDef) -> Self {
        Self {
            metadata: Metadata::default(),
            schema: schema.clone(),
        }
    }
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Metadata {
    #[serde(rename = "$id")]
    pub id: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "$comment")]
    pub comment: Option<String>,
    pub examples: Option<Vec<Value>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
#[serde(deny_unknown_fields)]
pub enum TypeDef {
    NamedType(NamedType),
    Ref {
        #[serde(rename = "$ref")]
        value: String,
    },
    Enum {
        #[serde(rename = "enum")]
        value: Vec<String>,
    },
    Const {
        #[serde(rename = "const")]
        value: Value,
    },
    OneOf {
        #[serde(rename = "oneOf")]
        value: Vec<Type>,
    },
    Empty {},
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
#[serde(deny_unknown_fields)]
pub enum NamedType {
    #[serde(rename = "boolean")]
    Boolean {},
    #[serde(rename = "string")]
    String { pattern: Option<String> },
    #[serde(rename = "integer")]
    Integer {
        minimum: Option<i64>,
        maximum: Option<i64>,
    },
    #[serde(rename = "number")]
    Number {
        minimum: Option<f64>,
        maximum: Option<f64>,
    },
    #[serde(rename = "array")]
    Array { items: Box<Type> },
    #[serde(rename = "object")]
    Object {
        #[serde(
            rename = "additionalProperties",
            default = "additional_properties_default",
            skip_serializing_if = "additional_properties_is_default"
        )]
        additional_properties: bool,
        #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
        properties: IndexMap<String, Type>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        required: Vec<String>,
    },
}

impl NamedType {
    fn as_object(&self) -> Option<Object> {
        match self {
            Self::Object {
                additional_properties,
                properties,
                required,
            } => Some(Object {
                additional_properties: *additional_properties,
                properties: properties.clone(),
                required: required.clone(),
            }),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Object {
    pub additional_properties: bool,
    pub properties: IndexMap<String, Type>,
    pub required: Vec<String>,
}

fn additional_properties_default() -> bool {
    true
}

fn additional_properties_is_default(value: &bool) -> bool {
    *value
}
