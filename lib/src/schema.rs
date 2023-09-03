use indexmap::map::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A schema file that may contain a top-level schema and related definitions
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SchemaFile {
    #[serde(flatten)]
    pub metadata: Metadata,
    #[serde(flatten)]
    pub schema: Option<SchemaDef>,
    #[serde(rename = "$defs")]
    pub definitions: Option<IndexMap<String, Schema>>,
}

impl SchemaFile {
    pub fn objects(&self) -> Vec<(Vec<String>, SchemaObject)> {
        let mut result = vec![];

        if let Some(schema) = &self.schema {
            Self::objects_rec(&Schema::new(schema), &[], &mut result);
        }

        if let Some(definitions) = &self.definitions {
            for (key, value) in definitions {
                Self::objects_rec(value, &[key.clone()], &mut result);
            }
        }

        result
    }

    fn objects_rec(schema: &Schema, path: &[String], acc: &mut Vec<(Vec<String>, SchemaObject)>) {
        match &schema.schema {
            SchemaDef::Type(SchemaType::Array { items }) => {
                let mut new_path = path.to_vec();
                new_path.push("array".to_string());

                Self::objects_rec(items, &new_path, acc);
            }
            SchemaDef::Type(other) => {
                if let Some(object) = other.as_object() {
                    acc.push((path.to_vec(), object.clone()));

                    for (key, value) in &object.properties {
                        let mut new_path = path.to_vec();
                        new_path.push(key.clone());

                        Self::objects_rec(value, &new_path, acc);
                    }
                }
            }
            SchemaDef::OneOf { value } => {
                for (i, schema) in value.iter().enumerate() {
                    let mut new_path = path.to_vec();
                    new_path.push(format!("oneOf[{}]", i));

                    Self::objects_rec(schema, &new_path, acc);
                }
            }
            _ => {}
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Schema {
    #[serde(flatten)]
    pub metadata: Metadata,
    #[serde(flatten)]
    pub schema: SchemaDef,
}

impl Schema {
    fn new(schema: &SchemaDef) -> Self {
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
pub enum SchemaDef {
    Type(SchemaType),
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
        value: Vec<Schema>,
    },
    Empty {},
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
#[serde(deny_unknown_fields)]
pub enum SchemaType {
    #[serde(rename = "null")]
    Null {},
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
    Array { items: Box<Schema> },
    #[serde(rename = "object")]
    Object(SchemaObject),
}

impl SchemaType {
    fn as_object(&self) -> Option<SchemaObject> {
        match self {
            Self::Object(object) => Some(object.clone()),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SchemaObject {
    #[serde(
        rename = "additionalProperties",
        default = "SchemaObject::additional_properties_default",
        skip_serializing_if = "SchemaObject::additional_properties_is_default"
    )]
    pub additional_properties: bool,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub properties: IndexMap<String, Schema>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required: Vec<String>,
}

impl SchemaObject {
    fn additional_properties_default() -> bool {
        true
    }

    fn additional_properties_is_default(value: &bool) -> bool {
        *value
    }
}
