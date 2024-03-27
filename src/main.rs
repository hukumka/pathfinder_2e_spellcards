mod json_utils;
mod markdown;
mod render;
mod rich_text;
mod spell;

use std::collections::HashMap;

use json_utils::JsonValueExt;
use render::write_to_pdf;
use spell::Spell;

fn main() -> anyhow::Result<()> {
    let spells = json::parse(include_str!("../nethys_data/spells.json"))?
        .as_array()?
        .into_iter()
        .map(|obj| {
            let result = Spell::parse(obj.as_object()?)?;
            Ok((result.name.to_lowercase(), result))
        })
        .collect::<anyhow::Result<HashMap<_, _>>>()?;

    let mut to_render = vec![];
    for line in std::fs::read_to_string("./spells.txt").unwrap().lines() {
        if let Some(spell) = spells.get(&line.to_lowercase()) {
            to_render.push(spell.clone());
        } else {
            println!("Spell not found: {line}");
        }
    }

    write_to_pdf(std::fs::File::create("output.pdf")?, &to_render)?;
    Ok(())
}
