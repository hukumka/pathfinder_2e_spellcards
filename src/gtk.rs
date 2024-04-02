use gtk::prelude::*;
use relm4::{factory::FactoryVecDeque, prelude::*};
use std::rc::Rc;

use crate::{
    db::{Query, SimpleSpellDB, SpellDB},
    spell::Spell,
};

/// Component holding all parameters to filter displayed spells.
///
/// Emits `SeachModelOutput` message, then filters are edited
#[derive(Debug)]
struct SearchModel {
    query: Query,
}

#[derive(Debug)]
struct SearchModelOutput {
    query: Query,
}

#[derive(Debug)]
enum SearchModelInput {
    UpdateNameSearch(String),
    UpdateSpellRank(Option<u8>),
    ArcaneToggled,
    PrimalToggled,
    DivineToggled,
    OccultToggled,
}

#[relm4::component]
impl SimpleComponent for SearchModel {
    type Init = ();
    type Input = SearchModelInput;
    type Output = SearchModelOutput;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 10,

            // Seach by name
            #[name(name_search)]
            gtk::SearchEntry {
                set_placeholder_text: Some("Search by name"),
                connect_search_changed[sender] => move |search| {
                    let text = search.text().to_string();
                    sender.input(SearchModelInput::UpdateNameSearch(text));
                }
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 10,

                // Rank filter
                #[name(rank_filter)]
                gtk::Entry {
                    set_placeholder_text: Some("Rank"),
                    set_input_purpose: gtk::InputPurpose::Digits,
                    connect_changed[sender] => move |rank| {
                        let text = rank.buffer().text();
                        if text.is_empty() {
                            sender.input(SearchModelInput::UpdateSpellRank(None));
                            return;
                        }
                        let value = text.parse::<u8>().unwrap();
                        if value != 0 && value <= 10 {
                            sender.input(SearchModelInput::UpdateSpellRank(Some(value)));
                        }
                    }
                },
                // Tradition filters
                gtk::CheckButton {
                    set_label: Some("Arcane"),
                    connect_toggled[sender] => move |_checkbox| {
                        sender.input(SearchModelInput::ArcaneToggled);
                    }
                },
                gtk::CheckButton {
                    set_label: Some("Primal"),
                    connect_toggled[sender] => move |_checkbox| {
                        sender.input(SearchModelInput::PrimalToggled);
                    }
                },
                gtk::CheckButton {
                    set_label: Some("Divine"),
                    connect_toggled[sender] => move |_checkbox| {
                        sender.input(SearchModelInput::DivineToggled);
                    }
                },
                gtk::CheckButton {
                    set_label: Some("Occult"),
                    connect_toggled[sender] => move |_checkbox| {
                        sender.input(SearchModelInput::OccultToggled);
                    }
                },
            }
        }
    }

    fn update(&mut self, message: SearchModelInput, sender: ComponentSender<Self>) {
        match message {
            SearchModelInput::UpdateNameSearch(search) => {
                self.query.name_query = search;
            }
            SearchModelInput::UpdateSpellRank(rank) => {
                self.query.spell_rank = rank;
            }
            SearchModelInput::ArcaneToggled => {
                self.query.is_arcane = !self.query.is_arcane;
            }
            SearchModelInput::PrimalToggled => {
                self.query.is_primal = !self.query.is_primal;
            }
            SearchModelInput::DivineToggled => {
                self.query.is_divine = !self.query.is_divine;
            }
            SearchModelInput::OccultToggled => {
                self.query.is_occult = !self.query.is_occult;
            }
        }
        sender
            .output(SearchModelOutput {
                query: self.query.clone(),
            })
            .unwrap();
    }

    fn init(
        _: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = SearchModel {
            query: Query::default(),
        };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }
}

/// Component displaying single spell entry in search result window
struct SpellDescription {
    spell: Rc<Spell>,
}

#[relm4::factory]
impl FactoryComponent for SpellDescription {
    type Init = Rc<Spell>;
    type Input = ();
    type Output = ();
    type CommandOutput = ();
    type ParentWidget = gtk::FlowBox;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_width_request: 150,
            set_spacing: 5,
            gtk::Label {
                set_label: self.spell.name.as_str(),
            },
            gtk::Label {
                set_label: self.spell.summary.as_str(),
                set_wrap: true,
            },
        }
    }

    fn init_model(spell: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self { spell }
    }

    fn update(&mut self, _msg: Self::Input, _sender: FactorySender<Self>) {}
}

pub struct App {
    search: Controller<SearchModel>,
    search_results: FactoryVecDeque<SpellDescription>,
    db: SimpleSpellDB,
}

#[derive(Debug)]
pub enum AppMessage {
    UpdateSearch(Query),
}

#[relm4::component(pub)]
impl SimpleComponent for App {
    type Init = SimpleSpellDB;
    type Input = AppMessage;
    type Output = ();

    view! {
        gtk::Window {
            set_title: Some("Spell card generator"),

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,

                #[local_ref]
                search_widget -> gtk::Box,

                gtk::ScrolledWindow {
                    set_hscrollbar_policy: gtk::PolicyType::Never,
                    set_min_content_width: 360,
                    #[local_ref]
                    search_result_widget -> gtk::FlowBox,
                }
            }
        }
    }

    fn init(
        db: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let search_results = FactoryVecDeque::builder()
            .launch(gtk::FlowBox::new())
            .forward(sender.input_sender(), |_| panic!());
        let search = SearchModel::builder()
            .launch(())
            .forward(sender.input_sender(), |output| {
                AppMessage::UpdateSearch(output.query)
            });
        let model = App {
            search,
            search_results,
            db,
        };
        let search_widget = model.search.widget();
        let search_result_widget = model.search_results.widget();
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: AppMessage, _sender: ComponentSender<Self>) {
        match message {
            AppMessage::UpdateSearch(query) => {
                let spells = self.db.search(&query);
                dbg!(spells.len());
                let mut guard = self.search_results.guard();
                guard.clear();
                for spell in spells {
                    guard.push_back(spell);
                }
            }
        }
    }
}
