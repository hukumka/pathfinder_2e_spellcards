mod db;
mod gtk;
mod json_utils;
mod markdown;
mod render;
mod rich_text;
mod spell;

use crate::db::SimpleSpellDB;
use crate::gtk::run_gtk_app;

fn main() -> anyhow::Result<()> {
    run_gtk_app(SimpleSpellDB::new("nethys_data/spells.json")?);
    Ok(())
    // let db = SimpleSpellDB::new("nethys_data/spells.json")?;
    // let spells = SimpleSpellDB::new("nethys_data/spells.json")?.search(&Query {
    //     name_query: "hero".to_string(),
    //     ..Query::default()
    // });
    // dbg!(&spells);
    // write_to_pdf(
    //     std::fs::File::create("output.pdf")?,
    //     spells.iter().map(|s| s.as_ref()),
    // )?;
    // Ok(())
}
