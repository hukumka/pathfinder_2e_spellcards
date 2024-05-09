use crate::spell::Spell;
use gtk4::glib::Properties;
use gtk4::{gio, glib, prelude::*, subclass::prelude::*, Widget};
use gtk4::{SignalListItemFactory, SingleSelection};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default)]
struct SpellModelImpl {
    spell: RefCell<Option<Rc<Spell>>>,
}

#[glib::object_subclass]
impl ObjectSubclass for SpellModelImpl {
    const NAME: &'static str = "SpellItem";
    type Type = SpellModel;
}

impl ObjectImpl for SpellModelImpl {}

impl SpellModelImpl {
    fn spell(&self) -> Rc<Spell> {
        self.spell.clone().into_inner().unwrap()
    }
}

glib::wrapper! {
    struct SpellModel(ObjectSubclass<SpellModelImpl>);
}

impl SpellModel {
    fn new(spell: Rc<Spell>) -> Self {
        let result: SpellModel = glib::Object::builder().build();
        result.imp().spell.replace(Some(spell));
        result
    }
}

#[derive(Properties, Default)]
#[properties(wrapper_type = SpellRow)]
struct SpellRowImpl {
    #[property(get, set)]
    label: RefCell<gtk4::Label>,
    #[property(get, set)]
    add_button: RefCell<gtk4::Button>,
}

#[glib::object_subclass]
impl ObjectSubclass for SpellRowImpl {
    const NAME: &'static str = "SpellRow";
    type Type = SpellRow;
    type ParentType = gtk4::Box;
}

#[glib::derived_properties]
impl ObjectImpl for SpellRowImpl {}

impl WidgetImpl for SpellRowImpl {}
impl BoxImpl for SpellRowImpl {}

glib::wrapper! {
    struct SpellRow (ObjectSubclass<SpellRowImpl>)
        @extends gtk4::Box, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Orientable;
}

impl SpellRow {
    pub fn new(label: gtk4::Label, add_button: gtk4::Button) -> Self {
        label.set_hexpand(true);
        let result: Self = glib::Object::builder().build();
        result.set_orientation(gtk4::Orientation::Horizontal);
        result.set_spacing(5);
        result.append(&label);
        result.append(&add_button);
        result.set_label(label);
        result.set_add_button(add_button);
        result
    }
}

type SpellCallback = Box<dyn Fn(Rc<Spell>)>;

#[derive(Clone)]
pub struct SpellCollection {
    model: gio::ListStore,
    spell_selected: Rc<RefCell<SpellCallback>>,
    spell_added: Rc<RefCell<SpellCallback>>,
}

impl SpellCollection {
    pub fn new() -> (Self, impl IsA<Widget>) {
        let model = gio::ListStore::new::<SpellModel>();
        let result = Self {
            model,
            spell_selected: Rc::new(RefCell::new(Box::new(|_| {}))),
            spell_added: Rc::new(RefCell::new(Box::new(|_| {}))),
        };
        let factory = result.setup_factory();
        let widget = result.build_widget(factory);
        (result, widget)
    }

    pub fn set_spells(&self, spells: &[Rc<Spell>]) {
        let items = spells
            .iter()
            .map(|spell| SpellModel::new(spell.clone()))
            .collect::<Vec<_>>();
        self.model.remove_all();
        self.model.extend_from_slice(&items);
    }

    pub fn connect_spell_selected(&self, selected: impl Fn(Rc<Spell>) + 'static) {
        let _ = self.spell_selected.as_ref().replace(Box::new(selected));
    }

    pub fn connect_spell_added(&self, added: impl Fn(Rc<Spell>) + 'static) {
        let _ = self.spell_added.as_ref().replace(Box::new(added));
    }

    fn build_widget(&self, factory: SignalListItemFactory) -> impl IsA<Widget> {
        let list_view = gtk4::ListView::builder()
            .factory(&factory)
            .model(&SingleSelection::new(Some(self.model.clone())))
            .build();
        gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vexpand(true)
            .child(&list_view)
            .build()
    }

    fn setup_factory(&self) -> SignalListItemFactory {
        let factory = SignalListItemFactory::new();
        let collection = self.clone();
        factory.connect_setup(move |_, list_item| {
            let list_item = list_item
                .downcast_ref::<gtk4::ListItem>()
                .expect("Must be ListItem");
            let row_widget = collection.build_row_widget();
            list_item.set_child(Some(&row_widget));

            let collection_moved = collection.clone();
            list_item.connect_selected_notify(move |item| {
                if item.is_selected() {
                    let model = item
                        .item()
                        .and_downcast::<SpellModel>()
                        .expect("Must be SpellModel");
                    collection_moved.spell_selected.as_ref().borrow()(model.imp().spell());
                }
            });

            let list_item = list_item.clone();
            row_widget.add_button().connect_clicked(move |_| {
                let model = list_item
                    .item()
                    .and_downcast::<SpellModel>()
                    .expect("Must be SpellModel");
                collection_moved.spell_added.as_ref().borrow()(model.imp().spell());
            });
        });
        factory.connect_bind(move |_, list_item| {
            let list_item = list_item
                .downcast_ref::<gtk4::ListItem>()
                .expect("Must be ListItem");
            let model = list_item
                .item()
                .and_downcast::<SpellModel>()
                .expect("Must be SpellModel");
            let child = list_item
                .child()
                .and_downcast::<SpellRow>()
                .expect("Must be SpellRow");
            let label = child.label();
            label.set_text(&model.imp().spell().name);
        });
        factory
    }

    fn build_row_widget(&self) -> SpellRow {
        let label = gtk4::Label::new(None);
        let add_button = gtk4::Button::builder()
            .icon_name("list-add-symbolic")
            .build();
        SpellRow::new(label, add_button)
    }
}
