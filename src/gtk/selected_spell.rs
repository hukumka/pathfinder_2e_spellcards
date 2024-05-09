use crate::spell::Spell;
use gtk4::{gio, glib, prelude::*, subclass::prelude::*, Widget};
use gtk4::{SignalListItemFactory, SingleSelection};
use std::rc::Rc;

mod spell_model_impl {
    use crate::spell::Spell;
    use gtk4::glib::{self, Properties};
    use gtk4::prelude::*;
    use gtk4::subclass::prelude::*;
    use std::cell::Cell;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::SelectedSpellModel)]
    pub struct SelectedSpellModelImpl {
        pub spell: RefCell<Option<Rc<Spell>>>,
        #[property(get, set)]
        count: Cell<u32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SelectedSpellModelImpl {
        const NAME: &'static str = "SelectedSpellModel";
        type Type = super::SelectedSpellModel;
    }

    #[glib::derived_properties]
    impl ObjectImpl for SelectedSpellModelImpl {}

    impl SelectedSpellModelImpl {
        pub fn spell(&self) -> Rc<Spell> {
            self.spell.clone().into_inner().unwrap()
        }
    }
}

glib::wrapper! {
    pub struct SelectedSpellModel(ObjectSubclass<spell_model_impl::SelectedSpellModelImpl>);
}

impl SelectedSpellModel {
    fn new(spell: Rc<Spell>) -> Self {
        let result: Self = glib::Object::builder().property("count", 1u32).build();
        result.imp().spell.replace(Some(spell));
        result
    }
}

mod selected_row_impl {
    use gtk4::glib::{self, Binding, Properties};
    use gtk4::prelude::*;
    use gtk4::subclass::prelude::*;
    use std::cell::RefCell;

    #[derive(Properties, Default)]
    #[properties(wrapper_type = super::SelectedSpellRow)]
    pub struct SelectedSpellRowImpl {
        #[property(get, set)]
        label: RefCell<gtk4::Label>,
        #[property(get, set)]
        count_label: RefCell<gtk4::Label>,
        #[property(get, set)]
        remove_button: RefCell<gtk4::Button>,
        #[property(get, set)]
        add_button: RefCell<gtk4::Button>,
        #[property(get, set)]
        binding: RefCell<Option<Binding>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SelectedSpellRowImpl {
        const NAME: &'static str = "SelectedSpellRow";
        type Type = super::SelectedSpellRow;
        type ParentType = gtk4::Box;
    }

    #[glib::derived_properties]
    impl ObjectImpl for SelectedSpellRowImpl {}

    impl WidgetImpl for SelectedSpellRowImpl {}
    impl BoxImpl for SelectedSpellRowImpl {}
}

glib::wrapper! {
    pub struct SelectedSpellRow (ObjectSubclass<selected_row_impl::SelectedSpellRowImpl>)
        @extends gtk4::Box, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Orientable;
}

impl SelectedSpellRow {
    pub fn new(
        label: gtk4::Label,
        count: gtk4::Label,
        add_button: gtk4::Button,
        remove_button: gtk4::Button,
    ) -> Self {
        label.set_hexpand(true);
        count.set_width_request(40);
        let result: Self = glib::Object::builder().build();
        result.set_orientation(gtk4::Orientation::Horizontal);
        result.set_spacing(5);
        result.append(&label);
        result.append(&remove_button);
        result.append(&count);
        result.append(&add_button);
        result.set_label(label);
        result.set_count_label(count);
        result.set_add_button(add_button);
        result.set_remove_button(remove_button);
        result
    }
}

#[derive(Clone)]
pub struct SelectedSpellCollection {
    model: gio::ListStore,
}

impl SelectedSpellCollection {
    pub fn new() -> (SelectedSpellCollection, impl IsA<Widget>) {
        let model = gio::ListStore::new::<SelectedSpellModel>();
        let result = Self { model };
        let factory = result.setup_factory();
        let widget = result.build_widget(factory);
        (result, widget)
    }

    pub fn collect_spells(&self) -> Vec<Rc<Spell>> {
        let mut result = vec![];
        let count = self.model.n_items();
        for index in 0..count {
            if let Some(spell_row) = self.model.item(index).and_downcast::<SelectedSpellModel>() {
                let spell = spell_row.imp().spell();
                for _ in 0..spell_row.count() {
                    result.push(spell.clone());
                }
            }
        }
        result
    }

    pub fn add_spell(&self, spell: Rc<Spell>) {
        let index = self.spell_index(spell.as_ref());
        if let Some(index) = index {
            let item = self
                .model
                .item(index)
                .and_downcast::<SelectedSpellModel>()
                .expect("Item must exist");
            item.set_count(item.count() + 1);
        } else {
            self.model.append(&SelectedSpellModel::new(spell));
        }
    }
    pub fn remove_spell(&self, spell: Rc<Spell>) {
        let index = self.spell_index(spell.as_ref());
        if let Some(index) = index {
            {
                let item = self
                    .model
                    .item(index)
                    .and_downcast::<SelectedSpellModel>()
                    .unwrap();
                let count = item.count();
                if count > 1 {
                    item.set_count(count - 1);
                    return;
                }
            }
            self.model.remove(index);
        }
    }

    fn spell_index(&self, spell: &Spell) -> Option<u32> {
        let count = self.model.n_items();
        (0..count).find(|i| {
            let item = self.model.item(*i).and_downcast::<SelectedSpellModel>();
            if let Some(item) = item {
                item.imp().spell().id == spell.id
            } else {
                false
            }
        })
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

            let list_item_moved = list_item.clone();
            let collection_moved = collection.clone();
            row_widget.remove_button().connect_clicked(move |_| {
                let model = list_item_moved
                    .item()
                    .and_downcast::<SelectedSpellModel>()
                    .expect("Must be SelectedSpellModel");
                collection_moved.remove_spell(model.imp().spell());
            });
            let list_item_moved = list_item.clone();
            let collection_moved = collection.clone();
            row_widget.add_button().connect_clicked(move |_| {
                let model = list_item_moved
                    .item()
                    .and_downcast::<SelectedSpellModel>()
                    .expect("Must be SelectedSpellModel");
                collection_moved.add_spell(model.imp().spell());
            });
        });
        factory.connect_bind(move |_, list_item| {
            let list_item = list_item
                .downcast_ref::<gtk4::ListItem>()
                .expect("Must be ListItem");
            let model = list_item
                .item()
                .and_downcast::<SelectedSpellModel>()
                .expect("Must be SelectedSpellModel");
            let child = list_item
                .child()
                .and_downcast::<SelectedSpellRow>()
                .expect("Must be SelectedSpellRow");
            let label = child.label();
            let count_label = child.count_label();

            label.set_text(&model.imp().spell().name);
            let binding = model
                .bind_property("count", &count_label, "label")
                .sync_create()
                .build();
            child.set_binding(binding);
        });
        factory.connect_unbind(move |_, list_item| {
            let list_item = list_item
                .downcast_ref::<gtk4::ListItem>()
                .expect("Must be ListItem");
            let child = list_item
                .child()
                .and_downcast::<SelectedSpellRow>()
                .expect("Must be SelectedSpellRow");
            if let Some(binding) = child.binding() {
                binding.unbind();
            }
        });
        factory
    }

    fn build_row_widget(&self) -> SelectedSpellRow {
        let label = gtk4::Label::new(None);
        let count_label = gtk4::Label::new(None);
        let remove_button = gtk4::Button::builder()
            .icon_name("list-remove-symbolic")
            .build();
        let add_button = gtk4::Button::builder()
            .icon_name("list-add-symbolic")
            .build();

        SelectedSpellRow::new(label, count_label, add_button, remove_button)
    }
}
