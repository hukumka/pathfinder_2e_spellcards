use crate::json_utils::JsonValueExt;
use crate::spell::{Spell, Traditions};
use anyhow::Result;
use std::rc::Rc;

#[derive(Debug, Clone, Default)]
pub struct Query {
    pub name_query: String,
    pub spell_rank: Option<u8>,
    pub is_arcane: bool,
    pub is_primal: bool,
    pub is_divine: bool,
    pub is_occult: bool,
}

impl Query {
    fn test(&self, spell: &Spell) -> bool {
        self.test_name(&spell.name)
            && self.test_rank(spell.level)
            && self.test_tradition(&spell.traditions)
    }

    fn test_rank(&self, rank: u8) -> bool {
        if let Some(query_rank) = self.spell_rank {
            query_rank == rank
        } else {
            true
        }
    }

    fn test_name(&self, name: &str) -> bool {
        name.to_lowercase()
            .contains(&self.name_query.to_lowercase())
    }

    fn test_tradition(&self, traditions: &Traditions) -> bool {
        let is_mismatch = (self.is_arcane && !traditions.is_arcane)
            || (self.is_divine && !traditions.is_divine)
            || (self.is_primal && !traditions.is_primal)
            || (self.is_occult && !traditions.is_occult);
        !is_mismatch
    }
}

pub trait SpellDB {
    fn search(&self, query: &Query) -> Vec<Rc<Spell>>;
}

/// Simplest possible implementation of spell database. Hella inefficient.
pub struct SimpleSpellDB {
    spells: Vec<Spell>,
}

impl SimpleSpellDB {
    pub fn new(data: &'static str) -> Result<Self> {
        let spells = json::parse(data)?
            .as_array()?
            .iter()
            .map(|obj| Spell::parse(obj.as_object()?))
            .collect::<anyhow::Result<Vec<_>>>()?;
        Ok(Self { spells })
    }
}

impl SpellDB for SimpleSpellDB {
    fn search<'a>(&self, query: &Query) -> Vec<Rc<Spell>> {
        self.spells
            .iter()
            .filter(|spell| query.test(spell))
            .map(|spell| Rc::new(spell.clone()))
            .collect()
    }
}
