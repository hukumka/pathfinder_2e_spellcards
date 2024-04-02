mod db;
mod gtk;
mod json_utils;
mod markdown;
mod render;
mod rich_text;
mod spell;

use crate::db::SimpleSpellDB;
use crate::gtk::App;

fn main() -> anyhow::Result<()> {
    let db = SimpleSpellDB::new("nethys_data/spells.json")?;
    let app = relm4::RelmApp::new("hukumka.spellcard_generator");
    app.run::<App>(db);
    Ok(())
}
