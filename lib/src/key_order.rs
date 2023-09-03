use super::{constants::*, path::Path};
use serde_json::Value;
use std::cmp::Ordering;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeyOrderMismatch<'a> {
    pub path: Path<'a>,
    pub first: &'a str,
    pub second: &'a str,
}

pub fn check_key_order(value: &Value) -> Vec<KeyOrderMismatch> {
    super::util::nodes_with_path(value)
        .iter()
        .filter(|(path, _)| !path.allows_arbitrary_keys())
        .filter_map(|(path, value)| {
            value.as_object().and_then(|fields| {
                let keys = fields.keys().map(|key| Key(key)).collect::<Vec<_>>();

                keys.windows(2)
                    .find(|window| window[0] >= window[1])
                    .map(|bad_window| KeyOrderMismatch {
                        path: path.clone(),
                        first: bad_window[0].0,
                        second: bad_window[1].0,
                    })
            })
        })
        .collect()
}

#[derive(Clone, Copy, Eq, PartialEq)]
struct Key<'a>(&'a str);

impl<'a> PartialOrd for Key<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self == other {
            Some(Ordering::Equal)
        } else {
            match (self.0, other.0) {
                (ID_KEY, _) => Some(Ordering::Less),
                (_, ID_KEY) => Some(Ordering::Greater),
                (TITLE_KEY, _) => Some(Ordering::Less),
                (_, TITLE_KEY) => Some(Ordering::Greater),
                (DESCRIPTION_KEY, _) => Some(Ordering::Less),
                (_, DESCRIPTION_KEY) => Some(Ordering::Greater),
                (COMMENT_KEY, _) => Some(Ordering::Less),
                (_, COMMENT_KEY) => Some(Ordering::Greater),
                (TYPE_KEY, _) => Some(Ordering::Less),
                (_, TYPE_KEY) => Some(Ordering::Greater),
                (ADDITIONAL_PROPERTIES_KEY, _) => Some(Ordering::Less),
                (_, ADDITIONAL_PROPERTIES_KEY) => Some(Ordering::Greater),
                (PROPERTIES_KEY, _) => Some(Ordering::Less),
                (_, PROPERTIES_KEY) => Some(Ordering::Greater),
                (REQUIRED_KEY, _) => Some(Ordering::Less),
                (_, REQUIRED_KEY) => Some(Ordering::Greater),
                (EXAMPLES_KEY, _) => Some(Ordering::Greater),
                (_, EXAMPLES_KEY) => Some(Ordering::Less),
                (_, _) => None,
            }
        }
    }
}
