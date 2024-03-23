use crate::json_utils::ObjectExt;
use anyhow::{anyhow, Result};
use json::object::Object;

#[derive(Debug, Clone)]
pub struct Spell {
    pub name: String,
    pub level: u8,
    pub spell_type: SpellType,
    pub traits: Vec<String>,
    pub actions: Actions,
    pub properties: Vec<Property>,
    pub description: String,
    pub heightened: Option<String>,
    pub extras: Vec<String>,
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
    Reaction,
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

        Ok(Spell {
            name,
            level: object.get_typed("level")?,
            spell_type: SpellType::parse(&object.get_typed::<String>("category")?)?,
            traits: Self::parse_traits(object)?,
            actions: Actions::parse(object.get_typed::<String>("actions")?)?,
            properties: Self::parse_properties(object)?,
            description,
            heightened,
            extras,
        })
    }

    fn parse_markdown(markdown: &str) -> Result<(String, Option<String>, Vec<String>)> {
        match markdown.split("---").collect::<Vec<_>>().as_slice() {
            &[_, description, heightened, ref extras @ ..] => Ok((
                description.trim().to_string(),
                Some(heightened.trim().to_string()),
                extras.iter().map(|s| s.to_string()).collect(),
            )),
            &[_, description] => Ok((description.to_string(), None, vec![])),
            _ => Err(anyhow!("Unable to extract description and heightened.")),
        }
    }

    fn parse_properties(object: &Object) -> Result<Vec<Property>> {
        let direct_properties = &[
            ("area", "Area"),
            ("duration_raw", "Duration"),
            ("target", "Target"),
            ("saving_throw", "Defence"),
        ];

        let result = direct_properties
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
        match source.as_str() {
            "Reaction" => Ok(Self::Reaction),
            "Singe Action" => Ok(Self::Number(1)),
            "Two Actions" => Ok(Self::Number(2)),
            "Three Actions" => Ok(Self::Number(3)),
            _ => Ok(Self::Other(source)),
        }
    }

    pub fn as_str(&self) -> Option<&'static str> {
        match self {
            Actions::Reaction => Some("5"),
            Actions::Number(3) => Some("3"),
            Actions::Number(2) => Some("2"),
            Actions::Number(1) => Some("1"),
            _ => None,
        }
    }
}
