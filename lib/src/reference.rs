use super::constants::DEFS_KEY;
use regex::Regex;
use std::fmt::Display;
use std::str::FromStr;

lazy_static::lazy_static! {
    static ref REF_PATTERN: Regex = Regex::new(r"^(?:((?:/\w+)*)/(\w+))?(?:#/\$defs/(\w+))?$").unwrap();
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum UnsupportedRefReason {
    HasScheme,
    HasQuery,
    InvalidStructure,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Unsupported reference")]
    UnsupportedRef {
        reason: UnsupportedRefReason,
        value: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Reference {
    PathOnly {
        path_prefix: Vec<String>,
        path_name: String,
    },
    FragmentOnly {
        fragment_name: String,
    },
    Both {
        path_prefix: Vec<String>,
        path_name: String,
        fragment_name: String,
    },
}

impl Reference {
    pub fn new(path_prefix: Vec<String>, path_name: String, fragment_name: String) -> Self {
        Reference::Both {
            path_prefix,
            path_name,
            fragment_name,
        }
    }

    pub fn from_path(path_prefix: Vec<String>, path_name: String) -> Self {
        Reference::PathOnly {
            path_prefix,
            path_name,
        }
    }

    pub fn from_fragment_name(fragment_name: String) -> Self {
        Reference::FragmentOnly { fragment_name }
    }

    pub fn path(&self) -> Option<String> {
        match self {
            Self::PathOnly { .. } => Some(format!("{}", self)),
            Self::Both {
                path_prefix,
                path_name,
                ..
            } => Self::from_path(path_prefix.clone(), path_name.clone()).path(),
            Self::FragmentOnly { .. } => None,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::PathOnly { path_name, .. } => path_name,
            Self::Both { fragment_name, .. } => fragment_name,
            Self::FragmentOnly { fragment_name } => fragment_name,
        }
    }
}

impl Display for Reference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PathOnly {
                path_prefix,
                path_name,
            } => {
                for part in path_prefix {
                    write!(f, "/{}", part)?;
                }
                write!(f, "/{}", path_name)
            }
            Self::FragmentOnly { fragment_name } => {
                write!(f, "#/{}/{}", DEFS_KEY, fragment_name)
            }
            Self::Both {
                path_prefix,
                path_name,
                fragment_name,
            } => {
                for part in path_prefix {
                    write!(f, "/{}", part)?;
                }
                write!(f, "/{}#/{}/{}", path_name, DEFS_KEY, fragment_name)
            }
        }
    }
}

impl FromStr for Reference {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with('/') && !s.starts_with('#') {
            Err(Error::UnsupportedRef {
                reason: UnsupportedRefReason::HasScheme,
                value: s.to_string(),
            })
        } else if s.contains('?') {
            Err(Error::UnsupportedRef {
                reason: UnsupportedRefReason::HasQuery,
                value: s.to_string(),
            })
        } else {
            let parts = REF_PATTERN
                .captures(s)
                .map(|captures| {
                    captures
                        .iter()
                        .skip(1)
                        .map(|value| value.map(|value| value.as_str()))
                        .collect::<Vec<_>>()
                })
                .ok_or_else(|| Error::UnsupportedRef {
                    reason: UnsupportedRefReason::InvalidStructure,
                    value: s.to_string(),
                })?;

            match parts.as_slice() {
                [Some(path_prefix), Some(path_name), Some(fragment_name)] => Ok(Reference::new(
                    path_prefix
                        .split('/')
                        .skip(1)
                        .map(|value| value.to_string())
                        .collect(),
                    path_name.to_string(),
                    fragment_name.to_string(),
                )),
                [Some(path_prefix), Some(path_name), None] => Ok(Reference::from_path(
                    path_prefix
                        .split('/')
                        .skip(1)
                        .map(|value| value.to_string())
                        .collect(),
                    path_name.to_string(),
                )),
                [None, None, Some(fragment_name)] => {
                    Ok(Reference::from_fragment_name(fragment_name.to_string()))
                }
                _ => Err(Error::UnsupportedRef {
                    reason: UnsupportedRefReason::InvalidStructure,
                    value: s.to_string(),
                }),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pairs() -> Vec<(&'static str, Reference)> {
        vec![
            (
                "/foo/bar/baz#/$defs/qux",
                Reference::new(
                    vec!["foo".to_string(), "bar".to_string()],
                    "baz".to_string(),
                    "qux".to_string(),
                ),
            ),
            (
                "/foo/bar/baz",
                Reference::from_path(
                    vec!["foo".to_string(), "bar".to_string()],
                    "baz".to_string(),
                ),
            ),
            (
                "#/$defs/qux",
                Reference::from_fragment_name("qux".to_string()),
            ),
        ]
    }

    #[test]
    fn ref_parse() {
        for (input, expected) in pairs() {
            assert_eq!(input.parse::<Reference>().unwrap(), expected);
        }
    }

    #[test]
    fn ref_display() {
        for (input, parsed) in pairs() {
            assert_eq!(input, parsed.to_string());
        }
    }
}
