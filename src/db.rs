use crate::json_utils::JsonValueExt;
use crate::spell::{Spell, Traditions};
use anyhow::Result;
use std::{fs::read_to_string, path::Path};

#[derive(Copy, Clone)]
pub struct Query<'a> {
    name_query: &'a str,
    spell_rank: Option<u8>,
    is_arcane: bool,
    is_primal: bool,
    is_divine: bool,
    is_occult: bool,
}

impl<'a> Query<'a> {
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
        name.contains(self.name_query)
    }

    fn test_tradition(&self, traditions: &Traditions) -> bool {
        (self.is_arcane && traditions.is_arcane)
            || (self.is_divine && traditions.is_divine)
            || (self.is_primal && traditions.is_primal)
            || (self.is_occult && traditions.is_occult)
    }
}

pub trait SpellDB {
    fn search<'a>(&self, query: Query<'a>) -> Vec<Spell>;
}

/// Simplest possible implementation of spell database. Hella inefficient.
pub struct SimpleSpellDB {
    spells: Vec<Spell>,
}

impl SimpleSpellDB {
    fn new(path: impl AsRef<Path>) -> Result<Self> {
        let data = read_to_string(path)?;
        let spells = json::parse(&data)?
            .as_array()?
            .into_iter()
            .map(|obj| Spell::parse(obj.as_object()?))
            .collect::<anyhow::Result<Vec<_>>>()?;
        Ok(Self { spells })
    }
}

impl SpellDB for SimpleSpellDB {
    fn search<'a>(&self, query: Query<'a>) -> Vec<Spell> {
        self.spells
            .iter()
            .filter(|spell| query.test(spell))
            .cloned()
            .collect()
    }
}
