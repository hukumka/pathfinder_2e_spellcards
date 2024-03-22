use crate::rich_text::{Font, RichText, RichTextBlock, RichTextPart};
use pulldown_cmark::{Event, Parser, Tag, TagEnd};

pub struct MdConfig<'a> {
    pub text_font: &'a Font,
    pub bold_font: &'a Font,
    pub italic_font: &'a Font,
    pub text_size: f32,
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

pub fn render_rich_text<'a>(config: &MdConfig<'a>, markdown: &str) -> RichText<'a> {
    let markdown = markdown
        .trim()
        .replace('\r', "")
        .replace("\n\n", "<br />\n");

    let mut parts = vec![];
    let mut md_state = MdState::default();
    let mut tag_stack = vec![];
    for part in Parser::new(&markdown) {
        match part {
            Event::Text(text) => parts.push(RichTextBlock::Text(RichTextPart {
                text: text.into_string(),
                font: md_state.get_font(config),
                font_size: config.text_size,
            })),
            Event::HardBreak => parts.push(RichTextBlock::LineBreak),
            Event::SoftBreak => parts.push(RichTextBlock::LineBreak),
            Event::Start(Tag::Link { title, .. }) => {
                parts.push(RichTextBlock::Text(RichTextPart {
                    text: title.into_string(),
                    font: config.italic_font,
                    font_size: config.text_size,
                }))
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
    RichText { parts }
}
