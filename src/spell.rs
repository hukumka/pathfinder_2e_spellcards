use crate::json_utils::ObjectExt;
use anyhow::{anyhow, bail, Result};
use json::object::Object;
use std::borrow::Cow;

#[derive(Debug, Clone)]
pub struct Spell {
    pub id: usize,
    pub name: String,
    pub level: u8,
    pub spell_type: SpellType,
    pub traits: Vec<String>,
    pub actions: Actions,
    pub properties: Vec<Property>,
    pub description: String,
    pub summary: String,
    pub heightened: Option<String>,
    pub extras: Vec<String>,
    pub traditions: Traditions,
}

#[derive(Debug, Copy, Clone)]
pub struct Traditions {
    pub is_arcane: bool,
    pub is_primal: bool,
    pub is_divine: bool,
    pub is_occult: bool,
}

/// Various properties like area, target or distance
#[derive(Debug, Clone)]
pub struct Property {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub enum SpellType {
    Spell,
    Focus,
    Cantrip,
}

#[derive(Debug, Clone)]
pub enum Actions {
    Number(u8),
    Range(u8, u8),
    Reaction,
    FreeAction,
    Other(String),
}

impl Spell {
    pub fn parse(object: &Object) -> Result<Spell> {
        Self::parse_(object).map_err(|err| {
            let name = object
                .get_typed("name")
                .unwrap_or_else(|_| "no-name".to_string());
            err.context(format!("Unable to parse spell `{name}`."))
        })
    }

    fn parse_(object: &Object) -> Result<Spell> {
        let name = object
            .get_typed("name")
            .map_err(|err| err.context("Unable to parse Spell."))?;
        let (description, heightened, extras) =
            Self::parse_markdown(&object.get_typed::<String>("markdown")?)?;
        let traditions = Traditions::parse(
            object
                .get_typed_maybe::<Vec<String>>("tradition")?
                .unwrap_or(vec![]),
        );

        Ok(Spell {
            id: Self::parse_id(object)?,
            name,
            level: object.get_typed("level")?,
            spell_type: SpellType::parse(&object.get_typed::<String>("category")?)?,
            traits: Self::parse_traits(object)?,
            actions: Actions::parse(object.get_typed::<String>("actions")?)?,
            properties: Self::parse_properties(object)?,
            description,
            summary: object.get_typed::<String>("summary")?,
            heightened,
            extras,
            traditions,
        })
    }

    fn parse_id(object: &Object) -> Result<usize> {
        let id = object.get_typed::<String>("id")?;
        if !id.starts_with("spell-") {
            bail!("Invalid Id format!");
        }
        Ok(id[6..].parse()?)
    }

    fn parse_markdown(markdown: &str) -> Result<(String, Option<String>, Vec<String>)> {
        match markdown.split("---").collect::<Vec<_>>().as_slice() {
            [_, description, heightened, ref extras @ ..] => Ok((
                description.trim().to_string(),
                Some(heightened.trim().to_string()),
                extras.iter().map(|s| s.to_string()).collect(),
            )),
            [_, description] => Ok((description.to_string(), None, vec![])),
            _ => Err(anyhow!("Unable to extract description and heightened.")),
        }
    }

    fn parse_properties(object: &Object) -> Result<Vec<Property>> {
        let str_properties = &[
            ("area", "Area"),
            ("duration_raw", "Duration"),
            ("target", "Target"),
            ("saving_throw", "Defence"),
            ("range_raw", "Range"),
            ("trigger", "Trigger"),
        ];

        let result = str_properties
            .iter()
            .filter_map(|(key, name)| Self::construct_propertry(object, key, name))
            .collect::<Result<Vec<Property>>>()?;

        Ok(result)
    }

    fn construct_propertry(object: &Object, key: &str, key_name: &str) -> Option<Result<Property>> {
        let value = object.get_typed_maybe::<String>(key).transpose()?;
        let value = match value {
            Ok(value) => value,
            Err(error) => {
                return Some(Err(error));
            }
        };
        Some(Ok(Property {
            name: key_name.to_string(),
            value,
        }))
    }

    fn parse_traits(object: &Object) -> Result<Vec<String>> {
        let mut traits: Vec<String> = object.get_typed("trait")?;
        let components: Option<Vec<String>> = object.get_typed_maybe("component")?;
        if let Some(components) = components {
            if components.contains(&"somatic".to_string()) {
                traits.push("Manipulate".to_string());
            }
            if components.contains(&"verbal".to_string()) {
                traits.push("Concentrate".to_string());
            }
        }
        Ok(traits)
    }
}

impl Traditions {
    fn parse(traditions: Vec<String>) -> Self {
        let mut result = Self {
            is_arcane: false,
            is_primal: false,
            is_divine: false,
            is_occult: false,
        };
        for tradition in &traditions {
            match tradition.as_str() {
                "Arcane" => {
                    result.is_arcane = true;
                }
                "Primal" => {
                    result.is_primal = true;
                }
                "Occult" => {
                    result.is_occult = true;
                }
                "Divine" => {
                    result.is_divine = true;
                }
                _ => {}
            }
        }
        result
    }
}

impl SpellType {
    fn parse(name: &str) -> Result<Self> {
        match name {
            "spell" => Ok(Self::Spell),
            "focus" => Ok(Self::Focus),
            "cantrip" => Ok(Self::Cantrip),
            _ => Err(anyhow!("Field `category` contains invalid value.")),
        }
    }
}

impl Actions {
    fn parse(source: String) -> Result<Self> {
        let result = Self::parse_range(&source)
            .or_else(|| Self::numeric_parse(&source))
            .unwrap_or(Self::Other(source));
        Ok(result)
    }

    fn numeric_parse(source: &str) -> Option<Self> {
        match source {
            "Reaction" => Some(Self::Reaction),
            "Single Action" => Some(Self::Number(1)),
            "Two Actions" => Some(Self::Number(2)),
            "Three Actions" => Some(Self::Number(3)),
            "Free Action" => Some(Self::FreeAction),
            _ => None,
        }
    }

    fn parse_range(source: &str) -> Option<Self> {
        let mut parts: Vec<_> = source.split("to").collect();
        if parts.len() != 2 {
            parts = source.split("or").collect();
        }
        if parts.len() != 2 {
            return None;
        }
        let left = Self::numeric_parse(parts[0].trim())?;
        let right = Self::numeric_parse(parts[1].trim())?;
        if let (Self::Number(from), Self::Number(to)) = (left, right) {
            Some(Self::Range(from, to))
        } else {
            None
        }
    }

    pub fn as_str(&self) -> Option<Cow<'static, str>> {
        match self {
            Actions::Reaction => Some(Cow::Borrowed("5")),
            Actions::FreeAction => Some(Cow::Borrowed("4")),
            Actions::Number(x) => Self::number_as_str(*x).map(Cow::Borrowed),
            _ => None,
        }
    }

    pub fn number_as_str(num: u8) -> Option<&'static str> {
        match num {
            1 => Some("1"),
            2 => Some("2"),
            3 => Some("3"),
            _ => None,
        }
    }
}
