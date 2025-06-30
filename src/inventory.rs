//! Define how the inventory works in the game.
//!
//! The inventory is basically a collection of items that the player can access.
//! The items are items he has gathered.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::Item;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inventory {
    items: HashMap<Item, usize>,
}

#[derive(Debug, Clone, Copy)]
pub struct HasNone;

impl Inventory {
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
        }
    }

    pub fn count_of(&self, item: &Item) -> usize {
        self.items.get(item).cloned().unwrap_or(0)
    }

    pub fn insert(&mut self, item: Item) {
        *self.items.entry(item).or_insert(0) += 1;
    }

    pub fn remove(&mut self, item: &Item) -> Result<(), HasNone> {
        if let Some(count) = self.items.get_mut(item) {
            assert!(*count > 0, "All items in the inventory must have count > 0");
            *count -= 1;
            if *count == 0 {
                self.items.remove(item);
            }
            Ok(())
        } else {
            Err(HasNone)
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (Item, usize)> {
        self.items.iter()
            .map(|(item, &count)| (item.clone(), count))
    }
}

impl Default for Inventory {
    fn default() -> Self {
        Self::new()
    }
}
