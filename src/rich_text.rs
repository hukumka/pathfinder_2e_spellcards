use anyhow::Result;
use font_kit::font;
use font_kit::handle::Handle;
use pathfinder_geometry::{rect::RectF, vector::Vector2F};
use printpdf::{IndirectFontRef, PdfDocumentReference};
use std::fmt;
use std::fs::File;
use std::path::Path;

pub struct Font {
    font: font::Font,
    font_ref: IndirectFontRef,
}

impl Font {
    /// Add Helvetica font to document, and construct reference to it.
    pub fn add_helvetica(
        doc: &mut PdfDocumentReference,
        font: printpdf::BuiltinFont,
    ) -> Result<Self> {
        let font_ref = doc
            .add_builtin_font(font)
            .map_err(|e| anyhow::Error::from(e).context("Unable to load font ref"))?;

        let font =
            Handle::from_path(AsRef::<Path>::as_ref("static/Helvetica.ttf").into(), 0).load()?;
        Ok(Font { font, font_ref })
    }

    /// Add External font to document, and construct reference to it.
    pub fn add_external_font(
        doc: &mut PdfDocumentReference,
        path: &impl AsRef<Path>,
    ) -> Result<Self> {
        let font_ref = doc.add_external_font(File::open(path)?)?;
        let font = Handle::from_path(path.as_ref().into(), 0).load()?;
        Ok(Font { font, font_ref })
    }

    pub fn font_ref(&self) -> &IndirectFontRef {
        &self.font_ref
    }

    fn char_width(&self, c: char) -> Option<f32> {
        let glyph = self.font.glyph_for_char(c)?;
        let offset = self.font.advance(glyph).ok()?;
        Some(offset.x())
    }

    fn scale(&self, size: f32) -> f32 {
        size / (self.font.metrics().units_per_em as f32)
    }
}

pub struct RichText<'a> {
    pub parts: Vec<RichTextBlock<'a>>,
}

pub enum RichTextBlock<'a> {
    Text(RichTextPart<'a>),
    LineBreak,
}

pub struct RichTextPart<'a> {
    pub text: String,
    pub font: &'a Font,
    pub font_size: f32,
}

/// Part of rich text, positioned
/// within layout, and ready for rendering.
pub struct TextChunk<'a> {
    pub text: &'a str,
    pub rect: RectF,
    pub font: &'a Font,
    pub font_size: f32,
}

impl<'a> fmt::Debug for TextChunk<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "TextChunk(text={text:#?}, rect={rect:#?})",
            text = self.text,
            rect = self.rect
        )
    }
}

pub enum JustifyContent {
    AlignLeft,
    AlignRight,
    JustifyEven,
}

pub struct RichTextLayoutBuilder<'a> {
    max_width: f32,
    lines: Vec<Vec<TextChunk<'a>>>,
    // Chunks in this incomplete line are positioned at
    // the same y_offset as previous line, and need to be
    // height adjusted afterwards.
    // It is needed, since line height could not be computed
    // until entire line is known.
    current_line: Vec<TextChunk<'a>>,
    x_offset: f32,
    y_offset: f32,
    split_policy: TextSplitPolicy,
    chunk_spacing: f32,
    line_spacing: f32,
    justify: JustifyContent,
    chunk_padding: f32,
}

impl<'a> TextChunk<'a> {
    fn height(&self) -> f32 {
        self.font_size
    }
}

#[derive(Copy, Clone)]
pub enum TextSplitPolicy {
    Words,
    Chars,
}

impl TextSplitPolicy {
    fn next<'a>(self, text: &'a str, offset: usize) -> usize {
        let slice = &text[offset..];
        let size = match self {
            Self::Words => Self::next_word(slice),
            Self::Chars => Self::next_char(slice),
        };
        size + offset
    }

    fn next_char(text: &str) -> usize {
        text.chars().next().map(|c| c.len_utf8()).unwrap_or(0)
    }

    fn next_word(text: &str) -> usize {
        let stripped = text.trim_start();
        let spaces_skipped = text.len() - stripped.len();
        let first_whitespace = stripped.char_indices().find(|(_, c)| c.is_whitespace());
        if let Some((loc, _)) = first_whitespace {
            spaces_skipped + loc
        } else {
            text.len()
        }
    }
}

pub struct Layout<'a> {
    /// Total height taken by the layout bounding box.
    pub height: f32,
    /// Text lines
    pub lines: Vec<Vec<TextChunk<'a>>>,
}

impl<'a> RichTextLayoutBuilder<'a> {
    /// Lay out text
    pub fn build(mut self, text: &'a RichText<'a>) -> Layout<'a> {
        self.add_rich_text(text);
        Layout {
            height: self.y_offset,
            lines: self.lines,
        }
    }

    /// Construct new box layout with default parameters.
    /// Box width is measured in Pt.
    pub fn new(max_width: f32) -> Self {
        Self {
            max_width,
            lines: vec![],
            current_line: vec![],
            x_offset: 0.0,
            y_offset: 0.0,
            split_policy: TextSplitPolicy::Words,
            chunk_spacing: 0.0,
            line_spacing: 0.0,
            chunk_padding: 0.0,
            justify: JustifyContent::AlignLeft,
        }
    }

    pub fn with_split_policy(mut self, policy: TextSplitPolicy) -> Self {
        self.split_policy = policy;
        self
    }

    pub fn with_justify(mut self, justify: JustifyContent) -> Self {
        self.justify = justify;
        self
    }

    pub fn with_chunk_spacing(mut self, spacing: f32) -> Self {
        self.chunk_spacing = spacing;
        self
    }

    pub fn with_line_spacing(mut self, spacing: f32) -> Self {
        self.line_spacing = spacing;
        self
    }

    pub fn with_chunk_padding(mut self, padding: f32) -> Self {
        self.chunk_padding = padding;
        self
    }

    fn add_rich_text(&mut self, text: &'a RichText<'a>) {
        for part in &text.parts {
            match part {
                RichTextBlock::Text(text) => self.add_part(text),
                RichTextBlock::LineBreak => self.finish_line(),
            }
        }
        if !self.current_line.is_empty() {
            self.finish_line();
        }
        self.y_offset -= self.line_spacing;
    }

    fn add_part(&mut self, part: &'a RichTextPart<'a>) {
        let mut text = part.text.as_str().trim();
        while !text.is_empty() {
            let (chunk, remaining) = self.split_chunk(part, text);
            if let Some(chunk) = chunk {
                self.x_offset += chunk.rect.width() + self.chunk_padding * 2.0 + self.chunk_spacing;
                self.current_line.push(chunk);
                text = remaining;
            } else {
                if self.current_line.is_empty() {
                    let chunk_text = &text[0..self.split_policy.next(text, 0)];
                    let chunk_width = self.get_text_width(part, chunk_text);
                    let x_offset = self.x_offset;
                    panic!(
                        "Cannot fit any characters from `{text}`. width: {width}, width(`{chunk_text}` = {chunk_width}), x_offset={x_offset}",
                        width = self.max_width,
                    );
                } else {
                    self.finish_line();
                }
            }
        }
    }

    fn split_chunk(
        &self,
        part: &'a RichTextPart,
        text: &'a str,
    ) -> (Option<TextChunk<'a>>, &'a str) {
        let text = text.trim();
        let mut offset = 0;
        let mut last_part = None;
        while offset < text.len() {
            let new_offset = self.split_policy.next(text, offset);
            let chunk = self.try_fit_chunk(part, &text[..new_offset]);
            if chunk.is_some() {
                last_part = chunk;
                offset = new_offset;
            } else {
                return (last_part, &text[offset..]);
            }
        }

        (last_part, &text[offset..])
    }

    fn get_text_width(&self, part: &'a RichTextPart, text: &'a str) -> f32 {
        let result = text
            .chars()
            .map(|c| part.font.char_width(c).unwrap_or(0.0))
            .sum::<f32>()
            * part.font.scale(part.font_size);
        result
    }

    fn try_fit_chunk(&self, part: &'a RichTextPart, text: &'a str) -> Option<TextChunk<'a>> {
        let width = self.get_text_width(part, text);
        if self.x_offset + 2.0 * self.chunk_padding + width > self.max_width {
            return None;
        }
        let height = part.font_size;

        let rect = RectF::new(
            Vector2F::new(
                self.x_offset + self.chunk_padding,
                self.y_offset + self.chunk_padding,
            ),
            Vector2F::new(width, height),
        );
        let result = TextChunk {
            text,
            rect,
            font: part.font,
            font_size: part.font_size,
        };
        Some(result)
    }

    fn finish_line(&mut self) {
        let mut line = vec![];
        std::mem::swap(&mut line, &mut self.current_line);
        let max_height = self.align_line_y(&mut line);
        match self.justify {
            JustifyContent::AlignLeft => {}
            JustifyContent::AlignRight => {
                self.align_line_right(&mut line);
            }
            JustifyContent::JustifyEven => {
                self.justify_line_even(&mut line);
            }
        }
        self.lines.push(line);
        self.x_offset = 0.0;
        self.y_offset += max_height + 2.0 * self.chunk_padding + self.line_spacing;
    }

    fn align_line_y(&self, line: &mut [TextChunk]) -> f32 {
        let max_height = line
            .iter()
            .map(|chunk| chunk.height())
            .fold(0.0f32, |l, r| l.max(r));

        for chunk in line {
            let old_origin = chunk.rect.origin();
            let y = old_origin.y() + max_height - chunk.rect.height(); // Align text by bottom line.
            set_origin_y(&mut chunk.rect, y);
        }

        max_height
    }

    fn align_line_right(&self, line: &mut [TextChunk]) {
        let mut x = self.max_width;
        for chunk in line.iter_mut().rev() {
            x -= chunk.rect.width();
            set_origin_x(&mut chunk.rect, x);
            dbg!(x, &chunk);
            x -= self.chunk_spacing;
        }
    }

    fn justify_line_even(&self, line: &mut [TextChunk]) {
        if line.len() <= 2 {
            return;
        }
        let total_spacing =
            self.max_width - line.iter().map(|chunk| chunk.rect.width()).sum::<f32>();
        let spacing = total_spacing / (line.len() - 1) as f32;
        let mut x = 0.0;
        for chunk in line {
            set_origin_x(&mut chunk.rect, x);
            x += chunk.rect.width() + spacing;
        }
    }
}

fn set_origin_x(rect: &mut RectF, x: f32) {
    *rect = RectF::new(Vector2F::new(x, rect.origin_y()), rect.size());
}

fn set_origin_y(rect: &mut RectF, y: f32) {
    *rect = RectF::new(Vector2F::new(rect.origin_x(), y), rect.size());
}
