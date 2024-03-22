mod json_utils;
mod markdown;
mod render;
mod rich_text;
mod spell;

use json_utils::JsonValueExt;
use render::write_to_pdf;
use spell::Spell;

fn main() -> anyhow::Result<()> {
    let spells = json::parse(include_str!("../nethys_data/spells.json"))?
        .as_array()?
        .into_iter()
        .map(|obj| Spell::parse(obj.as_object()?))
        .collect::<anyhow::Result<Vec<_>>>()?;

    write_to_pdf(std::fs::File::create("output.pdf")?, &spells)?;
    Ok(())
}
