use crate::rich_text::{Font, SceneBuilder};
use pulldown_cmark::{Event, Parser, Tag};
use std::borrow::Cow;

#[derive(Copy, Clone)]
pub struct MdConfig<'a> {
    pub text_font: &'a Font,
    pub bold_font: &'a Font,
    pub italic_font: &'a Font,
}

#[derive(Default)]
struct MdState {
    is_bold: bool,
    is_italic: bool,
}

impl MdState {
    fn apply(&mut self, tag: Tag) {
        match tag {
            Tag::Strong => self.is_bold = true,
            Tag::Emphasis => self.is_italic = true,
            _ => {}
        }
    }

    fn unapply(&mut self, tag: Tag) {
        match tag {
            Tag::Strong => self.is_bold = false,
            Tag::Emphasis => self.is_italic = false,
            _ => {}
        }
    }

    fn get_font<'a>(&self, config: &MdConfig<'a>) -> &'a Font {
        if self.is_bold {
            config.bold_font
        } else if self.is_italic {
            config.italic_font
        } else {
            config.text_font
        }
    }
}

impl<'a> SceneBuilder<'a> {
    pub fn add_markdown(&mut self, config: &MdConfig<'a>, markdown: &'a str) -> &mut Self {
        let mut md_state = MdState::default();
        let mut tag_stack = vec![];

        let mut iter = markdown.split("\n\n");
        for part in Parser::new(iter.next().unwrap()) {
            self.add_event(config, &mut md_state, &mut tag_stack, part);
        }
        for line in iter {
            self.finish_line();
            for part in Parser::new(&line) {
                self.add_event(config, &mut md_state, &mut tag_stack, part);
            }
        }
        self
    }

    fn add_event(
        &mut self,
        config: &MdConfig<'a>,
        md_state: &mut MdState,
        tag_stack: &mut Vec<Tag<'a>>,
        event: Event<'a>,
    ) {
        match event {
            Event::Text(text) => {
                let old_font = self.get_font();
                self.set_font(md_state.get_font(config));
                self.add_text(text);
                self.set_font(old_font);
            }
            Event::HardBreak | Event::SoftBreak => {
                self.finish_line();
            }
            Event::Start(Tag::Link { title, .. }) => {
                let old_font = self.get_font();
                self.set_font(config.italic_font)
                    .add_text(title)
                    .set_font(old_font);
            }
            Event::Start(tag) => {
                md_state.apply(tag.clone());
                tag_stack.push(tag);
            }
            Event::End(_) => {
                if let Some(tag) = tag_stack.pop() {
                    md_state.unapply(tag);
                }
            }
            _ => {}
        }
    }
}
