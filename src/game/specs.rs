use std::collections::{BTreeMap, HashSet};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Resource)]
pub struct SpecLookup<T: PartialEq + Eq>(BTreeMap<String, T>);

impl<T: PartialEq + Eq> SpecLookup<T> {
    pub fn get(&self, name: &str) -> Option<&T> {
        self.0.get(name)
    }

    pub fn get_names(&self) -> HashSet<&String> {
        self.0.keys().collect()
    }
}
