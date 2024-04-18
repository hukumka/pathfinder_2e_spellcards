use anyhow::Result;
use freetype::{Face, Library};
use pathfinder_geometry::{rect::RectF, vector::Vector2F};
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::path::Path;

const LINE_THICKNESS: f32 = 1.0;

pub struct Font<T> {
    font: Face,
    font_ref: T,
    size_cache: RefCell<HashMap<char, Option<f32>>>,
    units_per_em: f32,
}

#[derive(Copy, Clone)]
pub enum FontKind {
    Text,
    Bold,
    Italic,
    ActionCount,
}

impl FontKind {
    pub fn path(self) -> &'static str {
        match self {
            FontKind::Text | FontKind::Italic => "static/Helvetica.ttf",
            FontKind::Bold => "static/Helvetica-Bold.ttf",
            FontKind::ActionCount => "static/Pathfinder2eActions.ttf",
        }
    }
}

pub trait FontProvider: Sized {
    type Init;

    fn build_font(provider_source: &mut Self::Init, font: FontKind) -> Result<Self>;
}

impl<T: FontProvider> Font<T> {
    pub fn build(provider_source: &mut T::Init, font: FontKind) -> Result<Self> {
        let font_ref = T::build_font(provider_source, font)?;

        let font_path = font.path();
        let font = Library::init()?.new_face(font_path, 0)?;
        let units_per_em = font.em_size() as f32;
        Ok(Font {
            font,
            font_ref,
            size_cache: RefCell::new(HashMap::new()),
            units_per_em,
        })
    }
}

impl FontProvider for () {
    type Init = ();

    fn build_font(_: &mut Self::Init, _: FontKind) -> Result<Self> {
        Ok(())
    }
}

impl<T> Font<T> {
    pub fn font_ref(&self) -> &T {
        &self.font_ref
    }

    pub fn face(&self) -> &Face {
        &self.font
    }

    fn char_width(&self, c: char) -> Option<f32> {
        let mut map = self.size_cache.borrow_mut();
        if let Some(result) = map.get(&c) {
            return *result;
        }
        let _ = self
            .font
            .load_char(c as usize, freetype::face::LoadFlag::RENDER);
        let width = self.font.glyph().advance().x as f32;

        map.insert(c, Some(width));
        Some(width)
    }

    fn scale(&self, size: f32) -> f32 {
        size / self.units_per_em
    }
}

/// Polygon to draw boxes
pub struct Polygon {
    pub points: Vec<Vector2F>,
}

/// Scene to display
pub struct Scene<'a, T> {
    pub polygons: Vec<Polygon>,
    pub parts: Vec<TextChunk<'a, 'a, T>>,
}

/// Builder for rich text rendering.
///
/// Coordinates are measured in `Pt`.
/// Coordinates are
pub struct SceneBuilder<'a, T> {
    /// Prepared content.
    chunks: Vec<TextChunk<'a, 'a, T>>,
    polygons: Vec<Polygon>,
    /// Content which is still being laid out. Positions will change
    /// once line will be finilized.
    current_line: Vec<Block<'a, T>>,
    /// Bounding box inside which we try to fit content.
    bounding_box: RectF,
    current_font: &'a Font<T>,

    /// x position in current line for left line of bounding box.
    x_offset: f32,
    /// y position of current line from top line of bounding box.
    y_offset: f32,

    align: AlignStrategy,
    font_size: f32,

    line_space: f32,
    chunk_space: f32,
}

impl<'a, T> SceneBuilder<'a, T> {
    pub fn new(default_font: &'a Font<T>, bounding_box: RectF) -> Self {
        let mut result = Self {
            chunks: vec![],
            polygons: vec![],
            current_line: vec![],
            bounding_box,
            current_font: default_font,
            x_offset: 0.0,
            y_offset: 0.0,
            align: AlignStrategy::AlignLeft,
            font_size: 10.0,
            line_space: 0.0,
            chunk_space: 0.0,
        };
        result.set_default_chunk_space();
        result
    }

    pub fn scene(self) -> Scene<'a, T> {
        Scene {
            polygons: self.polygons,
            parts: self.chunks,
        }
    }

    pub fn get_bounding_box(&self) -> RectF {
        self.bounding_box
    }

    pub fn double_box(&mut self) {
        self.bounding_box = RectF::new(
            self.bounding_box.origin(),
            Vector2F::new(self.bounding_box.width(), self.bounding_box.height() * 2.0),
        );
    }

    pub fn is_out_of_bounds(&self) -> bool {
        return self.y_offset >= self.bounding_box.height();
    }

    pub fn set_font(&mut self, font: &'a Font<T>) -> &mut Self {
        self.current_font = font;
        self
    }

    pub fn set_line_space(&mut self, line_space: f32) -> &mut Self {
        self.line_space = line_space;
        self
    }

    pub fn get_font(&mut self) -> &'a Font<T> {
        self.current_font
    }

    pub fn set_font_size(&mut self, font_size: f32) -> &mut Self {
        self.font_size = font_size;
        self
    }

    pub fn set_alignment(&mut self, align: AlignStrategy) -> &mut Self {
        self.align = align;
        self
    }

    pub fn add_separator_line(&mut self) -> &mut Self {
        self.finish_line();
        self.y_offset += self.line_space * 2.0;
        self.polygons.push(Polygon {
            points: vec![
                self.bounding_box.origin() + Vector2F::new(0.0, self.y_offset),
                self.bounding_box.upper_right() + Vector2F::new(0.0, self.y_offset),
            ],
        });
        self.y_offset += self.line_space;
        self
    }

    pub fn add_rect(&mut self, rect: RectF) -> &mut Self {
        let rect = rect.contract(LINE_THICKNESS);
        self.polygons.push(Polygon {
            points: vec![
                rect.origin(),
                rect.upper_right(),
                rect.lower_right(),
                rect.lower_left(),
                rect.origin(),
            ],
        });
        self
    }

    pub fn add_boxed_text(&mut self, text: &'a str, padding: f32) -> &mut Self {
        let text_width = self.get_text_width(text);
        let width = text_width + 2.0 * padding;
        if width > self.bounding_box.width() {
            panic!(
                "Cannot fit `{text}`. Text required {width}Pt, but only {max_width}Pt available.",
                max_width = self.bounding_box.width()
            );
        }
        if width + self.x_offset > self.bounding_box.width() {
            self.finish_line();
        }

        let rect = RectF::new(
            Vector2F::new(self.x_offset + padding, self.y_offset + padding),
            Vector2F::new(text_width, self.font_size),
        );
        let block = Block::PaddedText {
            chunk: TextChunk {
                text: Cow::from(text),
                rect,
                font: self.current_font,
                font_size: self.font_size,
            },
            padding,
            border: true,
        };
        self.x_offset += width + self.chunk_space;
        self.current_line.push(block);
        self
    }

    pub fn add_text<'b: 'a>(&mut self, text: impl Into<Cow<'b, str>>) -> &mut Self {
        match text.into() {
            Cow::Borrowed(text) => self.add_text_str(text),
            Cow::Owned(text) => self.add_text_owned(text),
        }
    }

    fn add_text_owned(&mut self, text: String) -> &mut Self {
        let mut text = text.trim();
        while !text.is_empty() {
            let (chunk, remaining) = self.split_chunk(text);
            if let Some(TextChunk {
                text: chunk_text,
                rect,
                font,
                font_size,
            }) = chunk
            {
                let chunk_text: String = chunk_text.as_ref().to_string();
                self.x_offset += rect.width() + self.chunk_space;
                self.current_line.push(Block::Text(TextChunk {
                    text: Cow::from(chunk_text),
                    rect,
                    font,
                    font_size,
                }));
                text = remaining;
            } else {
                if self.current_line.is_empty() {
                    let text = &text[0..Self::next_word(text, 0)];
                    let width = self.get_text_width(text);
                    panic!("Cannot fit `{text}`. Text required {width}Pt, but only {max_width}Pt available.", max_width=self.bounding_box.width());
                } else {
                    self.finish_line();
                }
            }
        }
        self
    }

    fn add_text_str(&mut self, text: &'a str) -> &mut Self {
        let mut text = text.trim();
        while !text.is_empty() {
            let (chunk, remaining) = self.split_chunk(text);
            if let Some(chunk) = chunk {
                self.x_offset += chunk.rect.width() + self.chunk_space;
                self.current_line.push(Block::Text(chunk));
                text = remaining;
            } else {
                if self.current_line.is_empty() {
                    let text = &text[0..Self::next_word(text, 0)];
                    let width = self.get_text_width(text);
                    panic!("Cannot fit `{text}`. Text required {width}Pt, but only {max_width}Pt available.", max_width=self.bounding_box.width());
                } else {
                    self.finish_line();
                }
            }
        }
        self
    }

    pub fn set_default_chunk_space(&mut self) -> &mut Self {
        self.chunk_space = self.get_char_width(' ');
        self
    }

    pub fn set_chunk_space(&mut self, space: f32) -> &mut Self {
        self.chunk_space = space;
        self
    }

    fn get_char_width(&self, c: char) -> f32 {
        self.current_font.char_width(c).unwrap_or(0.0) * self.current_font.scale(self.font_size)
    }

    fn split_chunk<'b>(&self, text: &'b str) -> (Option<TextChunk<'a, 'b, T>>, &'b str) {
        let text = text.trim();
        let mut offset = 0;
        let mut last_part = None;
        while offset < text.len() {
            let new_offset = Self::next_word(text, offset);
            let chunk = self.try_fit_chunk(&text[..new_offset]);
            if chunk.is_some() {
                last_part = chunk;
                offset = new_offset;
            } else {
                return (last_part, &text[offset..]);
            }
        }

        (last_part, &text[offset..])
    }

    fn get_text_width(&self, text: &'a str) -> f32 {
        text.chars().map(|c| self.get_char_width(c)).sum::<f32>()
    }

    fn try_fit_chunk<'b>(&self, text: &'b str) -> Option<TextChunk<'a, 'b, T>> {
        let width = self.get_text_width(text);
        if self.x_offset + width > self.bounding_box.size().x() {
            return None;
        }
        let height = self.font_size;

        let rect = RectF::new(
            Vector2F::new(self.x_offset, self.y_offset),
            Vector2F::new(width, height),
        );
        let result = TextChunk {
            text: Cow::from(text),
            rect,
            font: self.current_font,
            font_size: self.font_size,
        };
        Some(result)
    }

    fn next_word(text: &str, offset: usize) -> usize {
        let slice = &text[offset..];
        let stripped = slice.trim_start();
        let spaces_skipped = slice.len() - stripped.len();
        let first_whitespace = stripped.char_indices().find(|(_, c)| c.is_whitespace());
        if let Some((loc, _)) = first_whitespace {
            offset + spaces_skipped + loc
        } else {
            text.len()
        }
    }

    pub fn finish_line(&mut self) -> &mut Self {
        if self.current_line.is_empty() {
            return self;
        }
        let mut line = vec![];
        std::mem::swap(&mut self.current_line, &mut line);
        let max_height = self.align_line_y(&mut line);
        match self.align {
            AlignStrategy::AlignLeft => {}
            AlignStrategy::AlignRight => {
                self.align_line_right(&mut line);
            }
            AlignStrategy::JustifyEven => {
                self.justify_line_even(&mut line);
            }
        }
        for block in line {
            self.add_block(block);
        }
        self.x_offset = 0.0;
        self.y_offset += max_height + self.line_space;
        self
    }

    fn add_block(&mut self, block: Block<'a, T>) {
        match block {
            Block::Text(chunk) => self.chunks.push(chunk),
            Block::PaddedText {
                chunk,
                padding,
                border,
            } => {
                if border {
                    self.add_rect(chunk.rect.dilate(padding));
                }
                self.chunks.push(chunk);
            }
        }
    }

    fn align_line_y(&self, line: &mut [Block<'a, T>]) -> f32 {
        let max_height = line
            .iter()
            .map(|chunk| chunk.height())
            .fold(0.0f32, |l, r| l.max(r));

        let bottom_line = self.y_offset + max_height;

        for chunk in line {
            chunk.align_to_bottom_line(bottom_line);
        }

        max_height
    }

    fn align_line_right(&self, line: &mut [Block<'a, T>]) {
        let mut x = self.bounding_box.width();
        for chunk in line.iter_mut().rev() {
            x -= chunk.width();
            chunk.align_to_left_line(x);
            x -= self.chunk_space;
        }
    }

    fn justify_line_even(&self, line: &mut [Block<'a, T>]) {
        if line.len() < 2 {
            return;
        }
        let total_spacing =
            self.bounding_box.width() - line.iter().map(|chunk| chunk.width()).sum::<f32>();
        let spacing = total_spacing / (line.len() - 1) as f32;
        let mut x = 0.0;
        for chunk in line {
            chunk.align_to_left_line(x);
            x += chunk.width() + spacing;
        }
    }
}

/// Part of rich text, positioned
/// within layout, and ready for rendering.
pub struct TextChunk<'a, 'b, T> {
    pub text: Cow<'b, str>,
    pub rect: RectF,
    pub font: &'a Font<T>,
    pub font_size: f32,
}

#[derive(Debug)]
pub enum Block<'a, T> {
    Text(TextChunk<'a, 'a, T>),
    PaddedText {
        chunk: TextChunk<'a, 'a, T>,
        padding: f32,
        border: bool,
    },
}

impl<'a, T> Block<'a, T> {
    fn height(&self) -> f32 {
        match self {
            Self::Text(chunk) => chunk.rect.height(),
            Self::PaddedText { chunk, padding, .. } => chunk.rect.height() + 2.0 * padding,
        }
    }

    fn width(&self) -> f32 {
        match self {
            Self::Text(chunk) => chunk.rect.width(),
            Self::PaddedText { chunk, padding, .. } => chunk.rect.width() + 2.0 * padding,
        }
    }

    fn align_to_left_line(&mut self, x_offset: f32) {
        match self {
            Self::Text(chunk) => {
                set_origin_x(&mut chunk.rect, x_offset);
            }
            Self::PaddedText { chunk, padding, .. } => {
                set_origin_x(&mut chunk.rect, x_offset + *padding);
            }
        }
    }

    fn align_to_bottom_line(&mut self, y_offset: f32) {
        match self {
            Self::Text(chunk) => {
                let height = chunk.rect.height();
                set_origin_y(&mut chunk.rect, y_offset - height);
            }
            Self::PaddedText { chunk, padding, .. } => {
                let height = chunk.rect.height();
                set_origin_y(&mut chunk.rect, y_offset - height - *padding);
            }
        }
    }
}

impl<'a, 'b, T> fmt::Debug for TextChunk<'a, 'b, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "TextChunk(text={text:#?}, rect={rect:#?})",
            text = self.text,
            rect = self.rect
        )
    }
}

pub enum AlignStrategy {
    AlignLeft,
    #[allow(dead_code)]
    AlignRight,
    JustifyEven,
}

fn set_origin_x(rect: &mut RectF, x: f32) {
    *rect = RectF::new(Vector2F::new(x, rect.origin_y()), rect.size());
}

fn set_origin_y(rect: &mut RectF, y: f32) {
    *rect = RectF::new(Vector2F::new(rect.origin_x(), y), rect.size());
}
