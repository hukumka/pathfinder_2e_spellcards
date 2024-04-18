use crate::db::{Query, SimpleSpellDB, SpellDB};
use crate::render::{build_spell_scene, FontConfig, OwnedFontConfig};
use crate::rich_text::{FontProvider, Scene, SceneBuilder};
use crate::spell::Spell;
use gtk4::{gdk, gio, prelude::*, subclass::prelude::*, ApplicationWindow};
use gtk4::{glib, Application, SingleSelection, Widget};
use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
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
    provider.load_from_path("static/gtk.css");
    gtk4::style_context_add_provider_for_display(
        &gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

// Spell list
#[derive(Default)]
struct SpellObjectImpl {
    spell: RefCell<Option<Rc<Spell>>>,
}

#[glib::object_subclass]
impl ObjectSubclass for SpellObjectImpl {
    const NAME: &'static str = "SpellItem";
    type Type = SpellObject;
}

impl ObjectImpl for SpellObjectImpl {
    fn constructed(&self) {
        self.parent_constructed();
    }
}

glib::wrapper! {
    struct SpellObject(ObjectSubclass<SpellObjectImpl>);
}

impl SpellObject {
    fn new(spell: Rc<Spell>) -> Self {
        let result: SpellObject = glib::Object::builder().build();
        result.imp().spell.replace(Some(spell));
        result
    }

    fn spell(&self) -> Rc<Spell> {
        self.imp()
            .spell
            .clone()
            .into_inner()
            .expect("Must be initialized with a Spell")
    }
}

/// Represent callbacks that can only be constructed after passing callback to a function.
#[derive(Clone)]
struct LateFunctionBind<T, Input> {
    inner: Rc<RefCell<Option<T>>>,
    _input: std::marker::PhantomData<Input>,
}

impl<T, Input> Default for LateFunctionBind<T, Input> {
    fn default() -> Self {
        LateFunctionBind {
            inner: Rc::new(RefCell::new(None)),
            _input: Default::default(),
        }
    }
}

impl<T, Input> LateFunctionBind<T, Input>
where
    T: Fn(&Input) + Clone,
    Input: Clone,
{
    fn new() -> (Self, Self) {
        let result = Self::default();
        (result.clone(), result)
    }

    fn set_callback(&self, item: T) {
        let mut inner = self.inner.borrow_mut();
        *inner = Some(item);
    }

    fn call(&self, value: &Input) {
        if let Some(func) = self.inner.as_ref().borrow().as_ref() {
            func(value);
        }
    }
}

struct SelectedSpell {
    count: usize,
    spell: Rc<Spell>,
}

struct SpellRepository {
    spells: HashMap<usize, SelectedSpell>,
    font_config: OwnedFontConfig<CairoFont>,
    active: Option<Rc<Spell>>,
}

impl SpellRepository {
    fn new() -> Self {
        let font_config =
            OwnedFontConfig::<CairoFont>::new(&mut freetype::Library::init().unwrap())
                .expect("Unable to initialize fonts");
        let spells = HashMap::new();
        Self {
            spells,
            font_config,
            active: None,
        }
    }
}

fn build_ui(db: Rc<SimpleSpellDB>, app: &Application) {
    let repository = Rc::new(RefCell::new(SpellRepository::new()));
    let layout = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .build();

    let left_sidebar = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .css_classes(["search_sidebar"])
        .build();

    // Construct widget containing search results
    let (on_result_select, on_result_select_pad) = LateFunctionBind::new();
    let (search_result_model, search_result_widget) =
        build_search_results_display(move |list_item| on_result_select.call(list_item));
    update_search_results(db.clone(), search_result_model.clone(), Default::default());

    // Construct search widget
    left_sidebar.append(&build_search(move |query| {
        update_search_results(db.clone(), search_result_model.clone(), query)
    }));

    // Construct spell preview widget
    let spell_preview = gtk4::DrawingArea::builder()
        .width_request(400)
        .hexpand(true)
        .vexpand_set(true)
        .build();
    let repository_moved = repository.clone();
    spell_preview.set_draw_func(move |_, context, w, h| {
        let repository = repository_moved.as_ref().borrow();
        if let Some(spell) = repository.active.as_ref() {
            let config = repository.font_config.config();
            let (scene, _) =
                build_spell_scene(&config, spell.as_ref()).expect("Scene must not be too large");
            draw_scene(context, w, h, scene);
        }
    });

    layout.append(&left_sidebar);
    layout.append(&search_result_widget);
    layout.append(&spell_preview);

    on_result_select_pad.set_callback(move |item| {
        if !item.is_selected() {
            return;
        }
        if let Some(spell_object) = item.item().and_downcast::<SpellObject>() {
            println!("Spell selected: {}", spell_object.spell().name);
            repository.borrow_mut().active = Some(spell_object.spell());
            spell_preview.queue_draw();
        } else {
        }
    });

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Spell Card generator")
        .child(&layout)
        .build();

    window.present();
}

fn update_search_results(db: Rc<SimpleSpellDB>, model: gio::ListStore, query: Query) {
    let items = db
        .as_ref()
        .search(&query)
        .into_iter()
        .map(SpellObject::new)
        .collect::<Vec<_>>();
    model.remove_all();
    model.extend_from_slice(&items);
}

fn build_search_results_display(
    on_select: impl Fn(&gtk4::ListItem) + Clone + 'static,
) -> (gio::ListStore, impl IsA<Widget>) {
    let model = gio::ListStore::new::<SpellObject>();
    let factory = gtk4::SignalListItemFactory::new();
    factory.connect_setup(move |_, list_item| {
        let child = setup_spell_item_widget();
        list_item.set_child(Some(&child));
        list_item.connect_selected_notify(on_select.clone());
    });
    factory.connect_bind(move |_, list_item| {
        let object = list_item
            .item()
            .and_downcast::<SpellObject>()
            .expect("Must be SpellObject");
        let child = list_item
            .child()
            .and_downcast::<SpellItemWidget>()
            .expect("Must be SpellItemWidget");
        bind_spell_item_widget_value(&child, &object);
    });

    let list_view = gtk4::ListView::builder()
        .factory(&factory)
        .model(&SingleSelection::new(Some(model.clone())))
        .css_classes(["spells"])
        .build();

    let scrolled_list_view = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .min_content_width(250)
        .width_request(250)
        .child(&list_view)
        .build();

    (model, scrolled_list_view)
}

type SpellItemWidget = gtk4::Label;

fn setup_spell_item_widget() -> SpellItemWidget {
    gtk4::Label::new(None)
}

fn bind_spell_item_widget_value(widget: &SpellItemWidget, value: &SpellObject) {
    widget.set_label(
        &value
            .imp()
            .spell
            .borrow()
            .as_ref()
            .map(|x| x.name.as_str())
            .unwrap_or(""),
    );
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
            if text.contains(|c: char| !c.is_digit(10)) {
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
        let font = provider_source.new_face(font.path(), 0)?;
        Ok(CairoFont {
            font: cairo::FontFace::create_from_ft(&font)?,
        })
    }
}
