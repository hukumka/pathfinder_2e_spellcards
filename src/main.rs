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
}
