use crate::rich_text::{Font, SceneBuilder};
use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use xml::reader::{EventReader, XmlEvent};

#[derive(Copy, Clone)]
pub struct MdConfig<'a, T> {
    pub text_font: &'a Font<T>,
    pub bold_font: &'a Font<T>,
    pub italic_font: &'a Font<T>,
}

impl<'a, T> SceneBuilder<'a, T> {
    pub fn add_markdown(&mut self, config: &MdConfig<'a, T>, markdown: &'a str) -> &mut Self {
        let mut tag_stack = vec![];

        let mut iter = markdown.split("\n\n").flat_map(|s| s.split("<br />"));
        let mut update_fn = |event| self.add_event(config, &mut tag_stack, event);
        traverse_markdown(iter.next().unwrap(), &mut update_fn);
        for line in iter {
            self.finish_line();
            let mut update_fn = |event| self.add_event(config, &mut tag_stack, event);
            traverse_markdown(line, &mut update_fn);
        }
        self
    }

    fn add_event(
        &mut self,
        config: &MdConfig<'a, T>,
        font_stack: &mut Vec<&'a Font<T>>,
        event: MixedEvent,
    ) {
        match event {
            MixedEvent::LineEnd => {
                println!("Explicit finish line");
                self.finish_line();
            }
            MixedEvent::Text(text) => {
                self.add_text(text);
            }
            MixedEvent::StartStyle(tag) => {
                font_stack.push(self.get_font());
                let font = match tag {
                    EmpasisTag::Bold => config.bold_font,
                    EmpasisTag::Italic => config.italic_font,
                };
                self.set_font(font);
            }
            MixedEvent::EndStyle => {
                let font = font_stack.pop().unwrap_or(config.text_font);
                self.set_font(font);
            }
        }
    }
}

enum MixedEvent {
    LineEnd,
    Text(String),
    StartStyle(EmpasisTag),
    EndStyle,
}

enum EmpasisTag {
    Bold,
    Italic,
}

fn traverse_markdown(markdown: &str, event_listener: &mut impl FnMut(MixedEvent)) {
    for event in Parser::new(markdown) {
        match event {
            Event::HardBreak | Event::SoftBreak => {
                event_listener(MixedEvent::LineEnd);
            }
            Event::Text(text) => {
                event_listener(MixedEvent::Text(text.into_string()));
            }
            Event::Start(Tag::Link { title, .. }) => {
                event_listener(MixedEvent::StartStyle(EmpasisTag::Italic));
                event_listener(MixedEvent::Text(title.into_string()));
                event_listener(MixedEvent::EndStyle);
            }
            Event::Start(Tag::Strong) => {
                event_listener(MixedEvent::StartStyle(EmpasisTag::Bold));
            }
            Event::Start(Tag::Emphasis) => {
                event_listener(MixedEvent::StartStyle(EmpasisTag::Italic));
            }
            Event::End(TagEnd::Strong | TagEnd::Emphasis) => {
                event_listener(MixedEvent::EndStyle);
            }
            Event::Html(html) => {
                traverse_html(html.as_bytes(), event_listener);
            }
            _ => {}
        }
    }
}

fn traverse_html(html: &[u8], event_listener: &mut impl FnMut(MixedEvent)) {
    for event in EventReader::new(html).into_iter().filter_map(|x| x.ok()) {
        match &event {
            XmlEvent::Characters(characters) => {
                traverse_markdown(characters, event_listener);
            }
            XmlEvent::StartElement { name, .. } => match name.local_name.as_str() {
                "li" => {
                    event_listener(MixedEvent::LineEnd);
                    event_listener(MixedEvent::Text("•".to_string()));
                }
                "tr" => {
                    event_listener(MixedEvent::LineEnd);
                }
                "td" => {
                    event_listener(MixedEvent::Text("|".to_string()));
                }
                _ => {}
            },
            _ => {}
        }
    }
}
