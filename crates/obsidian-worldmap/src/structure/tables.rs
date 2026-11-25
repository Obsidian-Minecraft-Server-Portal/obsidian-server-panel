//! The block states and biomes of one dimension, by id. Ids are assigned in the order states are first seen and never change,
//! so stored volumes stay valid as the tables grow. Id 0 is air in the state table and plains in the biome table.

use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};

use crate::chunk::BlockState;
use crate::error::{Result, WorldmapError};

pub const AIR_ID: u32 = 0;
pub const PLAINS_ID: u16 = 0;

#[derive(Serialize, Deserialize)]
struct StoredState {
    name: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    properties: BTreeMap<String, String>,
}

pub struct StateTable {
    states: Vec<BlockState>,
    ids: HashMap<BlockState, u32>,
}

impl Default for StateTable {
    fn default() -> Self {
        let mut table = Self {
            states: Vec::new(),
            ids: HashMap::new(),
        };
        table.extend([BlockState {
            name: "minecraft:air".into(),
            properties: BTreeMap::new(),
        }]);
        table
    }
}

impl StateTable {
    pub fn id(&self, state: &BlockState) -> Option<u32> {
        self.ids.get(state).copied()
    }

    pub fn states(&self) -> &[BlockState] {
        &self.states
    }

    /// Appends the states not held yet, in the order given.
    pub fn extend(&mut self, states: impl IntoIterator<Item = BlockState>) {
        for state in states {
            if !self.ids.contains_key(&state) {
                self.ids.insert(state.clone(), self.states.len() as u32);
                self.states.push(state);
            }
        }
    }

    pub fn to_json(&self) -> Result<Vec<u8>> {
        let stored: Vec<StoredState> = self
            .states
            .iter()
            .map(|state| StoredState {
                name: state.name.clone(),
                properties: state.properties.clone(),
            })
            .collect();
        Ok(serde_json::to_vec(&stored)?)
    }

    pub fn from_json(json: &[u8]) -> Result<Self> {
        let stored: Vec<StoredState> = serde_json::from_slice(json)?;
        let mut table = Self::default();
        table.extend(stored.into_iter().map(|state| BlockState {
            name: state.name,
            properties: state.properties,
        }));
        if table.states.first().map(|state| state.name.as_str()) != Some("minecraft:air") {
            return Err(WorldmapError::Structure("the state table does not start with air".into()));
        }
        Ok(table)
    }
}

#[derive(Default)]
pub struct BiomeTable {
    names: Vec<String>,
    ids: HashMap<String, u16>,
}

impl BiomeTable {
    pub fn with_plains() -> Self {
        let mut table = Self::default();
        table.extend(["minecraft:plains".to_owned()]);
        table
    }

    pub fn id(&self, name: &str) -> Option<u16> {
        self.ids.get(name).copied()
    }

    pub fn names(&self) -> &[String] {
        &self.names
    }

    pub fn extend(&mut self, names: impl IntoIterator<Item = String>) {
        for name in names {
            if !self.ids.contains_key(&name) {
                self.ids.insert(name.clone(), self.names.len() as u16);
                self.names.push(name);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stone() -> BlockState {
        BlockState {
            name: "minecraft:stone".into(),
            properties: BTreeMap::new(),
        }
    }

    #[test]
    fn keeps_air_first_and_assigns_ids_in_order() {
        let mut table = StateTable::default();
        table.extend([stone(), stone()]);
        assert_eq!(table.id(&BlockState { name: "minecraft:air".into(), properties: BTreeMap::new() }), Some(AIR_ID));
        assert_eq!(table.id(&stone()), Some(1));
    }

    #[test]
    fn survives_a_json_round_trip_with_ids_unchanged() {
        let mut table = StateTable::default();
        table.extend([stone()]);
        let restored = StateTable::from_json(&table.to_json().unwrap()).unwrap();
        assert_eq!(restored.states(), table.states());
    }

    #[test]
    fn keeps_plains_as_the_first_biome() {
        let mut biomes = BiomeTable::with_plains();
        biomes.extend(["minecraft:swamp".to_owned(), "minecraft:plains".to_owned()]);
        assert_eq!(biomes.id("minecraft:plains"), Some(PLAINS_ID));
        assert_eq!(biomes.id("minecraft:swamp"), Some(1));
    }
}
