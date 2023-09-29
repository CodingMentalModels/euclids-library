use std::collections::{BTreeMap, HashSet};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Resource)]
pub struct SpecLookup<T: PartialEq + Eq>(BTreeMap<String, T>);

impl<T: Clone + PartialEq + Eq> SpecLookup<T> {
    pub fn empty() -> Self {
        Self(BTreeMap::new())
    }

    pub fn from_vec(items: Vec<T>, get_key_fn: impl Fn(T) -> String) -> Self {
        Self(
            items
                .into_iter()
                .map(|item| (get_key_fn(item.clone()), item))
                .collect(),
        )
    }

    pub fn get(&self, name: &str) -> Option<&T> {
        self.0.get(name)
    }

    pub fn get_names(&self) -> HashSet<&String> {
        self.0.keys().collect()
    }

    pub fn as_vec(&self) -> Vec<(String, T)> {
        self.0.clone().into_iter().collect()
    }
}
