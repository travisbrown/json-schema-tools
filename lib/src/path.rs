use super::consts::{DEFS_KEY, PROPERTIES_KEY};
use std::fmt::Display;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Entry<'a> {
    Key(&'a str),
    Index(usize),
}

impl<'a> From<&'a str> for Entry<'a> {
    fn from(value: &'a str) -> Self {
        Self::Key(value)
    }
}

impl<'a> From<usize> for Entry<'a> {
    fn from(value: usize) -> Self {
        Self::Index(value)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Path<'a> {
    entries: Vec<Entry<'a>>,
}

impl<'a> Path<'a> {
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn push<E>(&mut self, value: E)
    where
        Entry<'a>: From<E>,
    {
        self.entries.push(Entry::from(value));
    }

    /// This path points to a JSON object with arbitrary keys
    pub fn has_free_keys(&self) -> bool {
        self.entries
            .last()
            .filter(|entry| matches!(entry, Entry::Key(PROPERTIES_KEY) | Entry::Key(DEFS_KEY)))
            .is_some()
    }
}

impl<'a> Display for Path<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for entry in &self.entries {
            match entry {
                Entry::Key(key) => {
                    write!(f, ".{}", key)?;
                }
                Entry::Index(index) => {
                    write!(f, "[{}]", index)?;
                }
            }
        }

        Ok(())
    }
}
