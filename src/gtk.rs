mod search_spells;
mod selected_spell;

use crate::db::{Query, SimpleSpellDB, SpellDB};
use crate::render::{build_spell_scene, write_to_pdf, OwnedFontConfig};
use crate::rich_text::{FontProvider, Scene};
use crate::spell::Spell;
use freetype::Library;
use gtk4::{gdk, gio, prelude::*, ApplicationWindow};
use gtk4::{glib, Application, Widget};
use search_spells::SpellCollection;
use selected_spell::SelectedSpellCollection;
use std::cell::RefCell;
use std::rc::Rc;

const APP_ID: &str = "org.hukumka.SpellcardGenerator";

pub fn run_gtk_app(db: SimpleSpellDB) -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();
    let db = Rc::new(db);
    app.connect_activate(move |app| build_ui(Rc::clone(&db), app));
    app.connect_startup(|_| load_css());
    app.run()
}

fn load_css() {
    let provider = gtk4::CssProvider::new();
    provider.load_from_data(include_str!("../static/gtk.css"));
    gtk4::style_context_add_provider_for_display(
        &gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

#[derive(Clone)]
struct AppState {
    db: Rc<SimpleSpellDB>,
    selected_spells: SelectedSpellCollection,
    search_results: SpellCollection,
    active_spell: Rc<RefCell<Option<Rc<Spell>>>>,
    window: ApplicationWindow,
}

impl AppState {
    fn new(db: Rc<SimpleSpellDB>, main_window: &ApplicationWindow) -> (Self, impl IsA<Widget>) {
        let (selected_spells, selected_spells_widget) = SelectedSpellCollection::new();
        let (search_results, search_results_widget) = SpellCollection::new();
        let active_spell = Rc::new(RefCell::new(None));
        let result = Self {
            db,
            selected_spells,
            search_results,
            active_spell,
            window: main_window.clone(),
        };

        let widget = result.build_widget(selected_spells_widget, search_results_widget);
        (result, widget)
    }

    fn build_widget(
        &self,
        selected_spells: impl IsA<Widget>,
        search_results: impl IsA<Widget>,
    ) -> impl IsA<Widget> {
        let layout = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .build();

        let left_sidebar = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .css_classes(["search_sidebar"])
            .build();

        let app_state = self.clone();
        left_sidebar.append(&build_search(move |query| {
            let result = app_state.db.search(&query);
            app_state.search_results.set_spells(&result);
        }));
        left_sidebar.append(&selected_spells);
        let export_button = gtk4::Button::builder().label("Export").build();
        left_sidebar.append(&export_button);

        let spell_preview_widget = self.build_search_preview_widget();
        layout.append(&left_sidebar);
        layout.append(&search_results);
        layout.append(&spell_preview_widget);

        self.connect_spell_activated(spell_preview_widget);
        self.connect_spell_added();
        self.connect_spell_removed();
        self.connect_export_dialog(export_button);

        layout
    }

    fn connect_export_dialog(&self, button: gtk4::Button) {
        let selected_spells = self.selected_spells.clone();
        let window = self.window.clone();
        button.connect_clicked(move |_| {
            let filter = gtk4::FileFilter::new();
            filter.add_suffix("pdf");
            filter.add_mime_type("pdf");
            let filters = gio::ListStore::new::<gtk4::FileFilter>();
            filters.append(&filter);
            let cancelable: Option<&gio::Cancellable> = None;
            let selected_spells_moved = selected_spells.clone();
            let window_moved = window.clone();
            gtk4::FileDialog::builder()
                .title("Save as")
                .filters(&filters)
                .build()
                .save(Some(&window), cancelable, move |file| {
                    if let Err(error) = Self::save_selected_spells(file, &selected_spells_moved) {
                        gtk4::AlertDialog::builder()
                            .detail(error.to_string())
                            .message("Error then exporting")
                            .build()
                            .show(Some(&window_moved));
                    }
                });
        });
    }

    fn save_selected_spells(
        file: Result<gio::File, glib::Error>,
        spells: &SelectedSpellCollection,
    ) -> anyhow::Result<()> {
        let file = file?;
        let path = file
            .path()
            .ok_or_else(|| anyhow::anyhow!("Cannot obtain path"))?;
        let file = std::fs::File::create(path)?;
        let spells = spells.collect_spells();
        write_to_pdf(file, spells.iter().map(|s| s.as_ref()))?;
        Ok(())
    }

    fn connect_spell_activated(&self, widget: impl IsA<Widget>) {
        let active_spell = self.active_spell.clone();
        self.search_results.connect_spell_selected(move |spell| {
            active_spell.replace(Some(spell));
            widget.queue_draw();
        });
    }

    fn connect_spell_added(&self) {
        let selected_spells = self.selected_spells.clone();
        let spell_added = move |spell: Rc<Spell>| {
            selected_spells.add_spell(spell);
        };
        self.search_results.connect_spell_added(spell_added);
    }

    fn connect_spell_removed(&self) {}

    fn build_search_preview_widget(&self) -> impl IsA<Widget> {
        let spell_preview = gtk4::DrawingArea::builder()
            .width_request(400)
            .hexpand(true)
            .vexpand_set(true)
            .build();

        let active_spell = self.active_spell.clone();
        let font_config: OwnedFontConfig<CairoFont> =
            OwnedFontConfig::new(&mut Library::init().unwrap()).unwrap();

        spell_preview.set_draw_func(move |_, context, w, h| {
            if let Some(spell) = active_spell.as_ref().borrow().as_ref() {
                let config = font_config.config();
                let (scene, _) = build_spell_scene(&config, spell.as_ref())
                    .expect("Scene must not be too large");
                draw_scene(context, w, h, scene);
            }
        });
        spell_preview
    }
}

fn build_ui(db: Rc<SimpleSpellDB>, app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Spell Card generator")
        .build();
    let (_, main_widget) = AppState::new(db, &window);
    window.set_child(Some(&main_widget));

    window.present();
}

fn build_search(on_search: impl Fn(Query) + Clone + 'static) -> impl IsA<Widget> {
    // Creating widgets and layout
    let search = gtk4::SearchEntry::builder()
        .placeholder_text("spell name")
        .build();
    let is_arcane = gtk4::CheckButton::builder().label("Arcane").build();
    let is_primal = gtk4::CheckButton::builder().label("Primal").build();
    let is_divine = gtk4::CheckButton::builder().label("Divine").build();
    let is_occult = gtk4::CheckButton::builder().label("Occult").build();
    let rank = gtk4::Entry::builder()
        .input_purpose(gtk4::InputPurpose::Digits)
        .max_length(2)
        .placeholder_text("rank")
        .build();

    let layout = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .build();

    let subbar = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .build();
    subbar.append(&rank);
    subbar.append(&is_arcane);
    subbar.append(&is_primal);
    subbar.append(&is_divine);
    subbar.append(&is_occult);

    layout.append(&search);
    layout.append(&subbar);

    // Handles user inputs
    let search_captured = search.clone();
    let is_arcane_captured = is_arcane.clone();
    let is_primal_captured = is_primal.clone();
    let is_divine_captured = is_divine.clone();
    let is_occult_captured = is_occult.clone();
    let rank_captured = rank.clone();

    let search_signal_handler = move || {
        let rank = rank_captured.text().parse::<u8>().ok();
        let is_arcane = is_arcane_captured.is_active();
        let is_primal = is_primal_captured.is_active();
        let is_occult = is_occult_captured.is_active();
        let is_divine = is_divine_captured.is_active();
        let query = search_captured.text();
        on_search(Query {
            name_query: query.to_string(),
            spell_rank: rank,
            is_arcane,
            is_primal,
            is_divine,
            is_occult,
        });
    };
    search.connect_search_changed(make_const_callback(&search_signal_handler));
    is_occult.connect_toggled(make_const_callback(&search_signal_handler));
    is_primal.connect_toggled(make_const_callback(&search_signal_handler));
    is_arcane.connect_toggled(make_const_callback(&search_signal_handler));
    is_divine.connect_toggled(make_const_callback(&search_signal_handler));
    rank.connect_changed(make_const_callback(&search_signal_handler));
    // Disable any inputs but numbers
    rank.delegate()
        .unwrap()
        .connect_insert_text(|rank, text, _| {
            if text.contains(|c: char| !c.is_ascii_digit()) {
                glib::signal::signal_stop_emission_by_name(rank, "insert-text");
            }
        });

    layout
}

/// Convinience function when working with gkt widgets.
///
/// Convert argument-less function reference into callback that takes appropriate widget.
fn make_const_callback<T>(callback: &(impl Fn() + 'static + Clone)) -> impl Fn(&T) + 'static {
    let cb = callback.clone();
    move |_| cb()
}

fn draw_scene(context: &cairo::Context, width: i32, height: i32, scene: Scene<'_, CairoFont>) {
    let width = width as f64;
    let height = height as f64;
    let (min_x, max_x, min_y, max_y) = scene
        .polygons
        .iter()
        .flat_map(|poly| poly.points.iter())
        .fold(
            (f64::MAX, f64::MIN, f64::MAX, f64::MIN),
            |(min_x, max_x, min_y, max_y), b| {
                let x = b.x() as f64;
                let y = b.y() as f64;
                (min_x.min(x), max_x.max(x), min_y.min(y), max_y.max(y))
            },
        );

    let scene_width = max_x - min_x;
    let scene_height = max_y - min_y;
    let padding = 30.0;
    let x_scale = (width - padding * 2.0) / scene_width;
    let y_scale = (height - padding * 2.0) / scene_height;
    let (scale, x_offset, y_offset) = if x_scale < y_scale {
        (
            x_scale,
            padding - min_x,
            (height - scene_height * x_scale) * 0.5 - min_y,
        )
    } else {
        (
            y_scale,
            (width - scene_width * y_scale) * 0.5 - min_x,
            padding - min_y,
        )
    };

    context.translate(x_offset, y_offset);
    context.scale(scale, scale);
    context.set_source_rgb(1.0, 1.0, 1.0);
    context.rectangle(min_x, min_y, scene_width, scene_height);
    context.fill().expect("Could not fill");
    context.set_source_rgb(0.0, 0.0, 0.0);

    context.set_line_width(0.5);
    for poly in &scene.polygons {
        context.move_to(poly.points[0].x() as f64, poly.points[0].y() as f64);
        for point in &poly.points[1..] {
            context.line_to(point.x() as f64, point.y() as f64);
        }
        context.stroke().expect("Cannot draw line");
    }

    for text in &scene.parts {
        context.set_font_size(text.font_size as f64 * 0.97);
        context.set_font_face(&text.font.font_ref().font);
        let pos = text.rect.lower_left();
        context.move_to(pos.x() as f64, pos.y() as f64);
        context.show_text(&text.text).expect("Cannot render text");
    }
}

struct CairoFont {
    font: cairo::FontFace,
}

impl FontProvider for CairoFont {
    type Init = freetype::Library;

    fn build_font(
        provider_source: &mut Self::Init,
        font: crate::rich_text::FontKind,
    ) -> anyhow::Result<Self> {
        let bytes = font.bytes();
        let mut data = Vec::with_capacity(bytes.len());
        data.extend_from_slice(bytes);
        let data = Rc::new(data);
        let font = provider_source.new_memory_face(data, 0)?;
        Ok(CairoFont {
            font: cairo::FontFace::create_from_ft(&font)?,
        })
    }
}
