use crate::markdown::MdConfig;
use crate::rich_text::{
    AlignStrategy, Font, FontKind, FontProvider, Scene, SceneBuilder, TextChunk,
};
use crate::spell::{Actions, Spell};
use anyhow::{anyhow, Result};
use pathfinder_geometry::rect::RectF;
use pathfinder_geometry::vector::Vector2F;
use printpdf::{
    path::{PaintMode, WindingOrder},
    Color, Mm, PdfDocument, PdfLayerReference, Point, Polygon, Pt, Rgb,
};
use printpdf::{BuiltinFont, IndirectFontRef, PdfDocumentReference};
use std::io::{BufWriter, Write};

// Everything is measured in Mm
const A4_WIDTH: f32 = 210.0;
const A4_HEIGHT: f32 = 297.0;
const CARD_WIDTH: f32 = 63.0;
const CARD_HEIGHT: f32 = 88.0;

const GRID_WIDTH: usize = 3;
const GRID_HEIGHT: usize = 3;

const X_PADDING: f32 = 2.0;
const Y_PADDING: f32 = 2.0;
const X_PADDING_PAGE: f32 =
    (A4_WIDTH - CARD_WIDTH * (GRID_WIDTH as f32) - X_PADDING * (GRID_WIDTH as f32 - 1.0)) * 0.5;
const Y_PADDING_PAGE: f32 =
    (A4_HEIGHT - CARD_HEIGHT * (GRID_HEIGHT as f32) - Y_PADDING * (GRID_HEIGHT as f32 - 1.0)) * 0.5;
const MARGIN: f32 = 1.0;
const CARD_WIDTH_INNER: f32 = CARD_WIDTH - 2.0 * MARGIN;
const CARD_HEIGHT_INNER: f32 = CARD_HEIGHT - 2.0 * MARGIN;

const HEADER_LINE_SPACE: f32 = 1.0;
const LINE_SPACE: f32 = 0.5;
const TRAIT_PADDING: f32 = 0.8;
const TRAIT_CHUNK_SPACE: f32 = 0.3;

const GENERAL_TEXT_FONT_SIZE: f32 = 7.7;

#[derive(Copy, Clone)]
pub struct FontConfig<'a, T> {
    md_config: MdConfig<'a, T>,
    action_count_font: &'a Font<T>,
}

pub struct OwnedFontConfig<T> {
    text: Font<T>,
    bold: Font<T>,
    italic: Font<T>,
    action_count: Font<T>,
}

impl FontProvider for IndirectFontRef {
    type Init = PdfDocumentReference;

    fn build_font(provider: &mut Self::Init, font: FontKind) -> Result<IndirectFontRef> {
        let font = match font {
            FontKind::ActionCount => {
                return Ok(provider.add_external_font(font.bytes())?);
            }
            FontKind::Text => BuiltinFont::Helvetica,
            FontKind::Bold => BuiltinFont::HelveticaBold,
            FontKind::Italic => BuiltinFont::HelveticaOblique,
        };

        let result = provider
            .add_builtin_font(font)
            .map_err(|e| anyhow::Error::from(e).context("Unable to load font ref"))?;
        Ok(result)
    }
}

impl<T: FontProvider> OwnedFontConfig<T> {
    pub fn new(doc: &mut T::Init) -> Result<Self> {
        let text = Font::<T>::build(doc, FontKind::Text)
            .map_err(|e| e.context("Unable to load Helvetica"))?;

        let bold = Font::<T>::build(doc, FontKind::Bold)
            .map_err(|e| e.context("Unable to load Helvetica Bold"))?;

        let italic = Font::<T>::build(doc, FontKind::Italic)
            .map_err(|e| e.context("Unable to load Helvetica Italic"))?;

        let action_count = Font::<T>::build(doc, FontKind::ActionCount)
            .map_err(|e| e.context("Unable to load Pathfinder Icons font"))?;
        Ok(Self {
            text,
            bold,
            italic,
            action_count,
        })
    }

    pub fn config(&self) -> FontConfig<'_, T> {
        FontConfig {
            md_config: MdConfig {
                text_font: &self.text,
                bold_font: &self.bold,
                italic_font: &self.italic,
            },
            action_count_font: &self.action_count,
        }
    }
}

/// Write document containing all spells into `output`
pub fn write_to_pdf<'a, T: Write>(
    output: T,
    spells: impl IntoIterator<Item = &'a Spell>,
) -> Result<()> {
    let (mut doc, page1, layer1) =
        PdfDocument::new("Spells", Mm(A4_WIDTH), Mm(A4_HEIGHT), "Layer1");

    let owned_font_config = OwnedFontConfig::<IndirectFontRef>::new(&mut doc)?;
    let font_config = owned_font_config.config();
    let mut layer = doc.get_page(page1).get_layer(layer1);

    init_page(&mut layer);
    let pages = build_pages(&font_config, spells);
    if pages.len() >= GRID_WIDTH {
        draw_page(&mut layer, &pages[..GRID_WIDTH]);
        for page in pages[GRID_WIDTH..].chunks(GRID_WIDTH) {
            let (page_index, layer_index) = doc.add_page(Mm(A4_WIDTH), Mm(A4_HEIGHT), "Layer");
            layer = doc.get_page(page_index).get_layer(layer_index);
            init_page(&mut layer);
            draw_page(&mut layer, page);
        }
    } else {
        draw_page(&mut layer, &pages);
    }

    doc.save(&mut BufWriter::new(output))?;
    Ok(())
}

fn draw_page(layer: &mut PdfLayerReference, page: &[[PageCell; GRID_HEIGHT]]) {
    for (x, row) in page.iter().enumerate() {
        for (y, scene) in row.iter().enumerate() {
            if let PageCell::Filled(scene) = scene {
                render_scene(layer, (x, y), scene);
            }
        }
    }
}

pub enum PageCell<'a> {
    Filled(Scene<'a, IndirectFontRef>),
    Empty,
}

fn build_pages<'a, 'b: 'a>(
    font_config: &'a FontConfig<'a, IndirectFontRef>,
    spells: impl IntoIterator<Item = &'b Spell>,
) -> Vec<[PageCell<'a>; GRID_HEIGHT]> {
    let mut doubles = vec![];
    let mut normal = vec![];
    for spell in spells {
        match build_spell_scene(font_config, spell) {
            Ok((scene, true)) => doubles.push(scene),
            Ok((scene, false)) => normal.push(scene),
            Err(error) => {
                eprintln!("Failed to render spell: {}. {}", spell.name, error);
            }
        }
    }

    let mut pad: [PageCell; GRID_HEIGHT] = std::array::from_fn(|_| PageCell::Empty);
    let mut pad_index = 0;
    let mut result = vec![];

    while !(doubles.is_empty() && normal.is_empty()) {
        if pad_index + 2 <= GRID_HEIGHT && !doubles.is_empty() {
            pad[pad_index] = PageCell::Filled(doubles.pop().unwrap());
            pad_index += 2;
        } else {
            pad[pad_index] = PageCell::Filled(normal.pop().unwrap());
            pad_index += 1;
        }
        if pad_index == GRID_HEIGHT {
            pad_index = 0;
            let mut tmp = std::array::from_fn(|_| PageCell::Empty);
            std::mem::swap(&mut pad, &mut tmp);
            result.push(tmp);
        }
    }
    if pad_index > 0 {
        result.push(pad);
    }

    result
}

fn init_page(layer: &mut PdfLayerReference) {
    layer.set_outline_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)));
    layer.set_outline_thickness(0.0);
}

/// Write spell
pub fn build_spell_scene<'a, T>(
    config: &'a FontConfig<'a, T>,
    spell: &'a Spell,
) -> Result<(Scene<'a, T>, bool)> {
    let rect = RectF::new(
        Vector2F::zero(),
        Vector2F::new(mm_to_pt(CARD_WIDTH_INNER), mm_to_pt(CARD_HEIGHT_INNER)),
    );
    let mut builder = SceneBuilder::<'a, T>::new(config.md_config.text_font, rect);

    builder
        .set_line_space(mm_to_pt(HEADER_LINE_SPACE))
        // Draw header
        .set_alignment(AlignStrategy::JustifyEven)
        .set_font_size(11.0) // Name
        .add_text(&spell.name);

    if let Actions::Range(from, to) = &spell.actions {
        builder
            .set_font_size(14.0)
            .set_font(config.action_count_font) // Action count;
            .add_text(Actions::number_as_str(*from).unwrap_or(""))
            .set_font(config.md_config.text_font)
            .set_font_size(11.0)
            .add_text("to")
            .set_font(config.action_count_font) // Action count;
            .set_font_size(14.0)
            .add_text(Actions::number_as_str(*to).unwrap_or(""))
            .set_font(config.md_config.text_font);
    } else if let Some(action) = spell.actions.as_str() {
        builder
            .set_font_size(14.0)
            .set_font(config.action_count_font) // Action count;
            .add_text(action)
            .set_font(config.md_config.text_font);
    }
    builder
        .set_font_size(11.0) // Spell level
        .add_text(format!("{}", spell.level))
        .finish_line();

    // Draw traits
    builder
        .set_line_space(mm_to_pt(LINE_SPACE))
        .set_font_size(GENERAL_TEXT_FONT_SIZE)
        .set_chunk_space(mm_to_pt(TRAIT_CHUNK_SPACE))
        .set_alignment(AlignStrategy::AlignLeft);
    for trait_ in &spell.traits {
        builder.add_boxed_text(trait_.as_str(), mm_to_pt(TRAIT_PADDING));
    }
    builder.set_default_chunk_space().finish_line();
    // Draw properties
    for property in &spell.properties {
        builder
            .set_font(config.md_config.bold_font)
            .add_text(property.name.as_str())
            .set_font(config.md_config.text_font)
            .add_text(property.value.as_str())
            .finish_line();
    }
    builder.add_separator_line();
    builder.add_markdown(&config.md_config, &spell.description);
    if let Some(heighened) = &spell.heightened {
        builder.add_separator_line();
        builder
            .add_markdown(&config.md_config, heighened.as_str())
            .finish_line();
    }
    builder.finish_line();

    let is_double = if builder.is_out_of_bounds() {
        builder.double_box();
        true
    } else {
        false
    };
    builder.add_rect(builder.get_bounding_box().dilate(mm_to_pt(MARGIN) + 1.0));

    if builder.is_out_of_bounds() {
        Err(anyhow!(
            "Spell `{spell_name}` does not fit card format!",
            spell_name = spell.name
        ))
    } else {
        Ok((builder.scene(), is_double))
    }
}

fn render_scene(
    layer: &mut PdfLayerReference,
    (x, y): (usize, usize),
    scene: &Scene<'_, IndirectFontRef>,
) {
    let offset = Point::new(
        Mm(X_PADDING_PAGE + (CARD_WIDTH + X_PADDING) * x as f32),
        Mm(Y_PADDING_PAGE + (CARD_HEIGHT + Y_PADDING) * (GRID_HEIGHT - 1 - y) as f32),
    );
    for chunk in &scene.parts {
        draw_text(layer, offset, chunk);
    }
    let polygons = scene
        .polygons
        .iter()
        .map(|poly| {
            poly.points
                .iter()
                .map(|x| (text_coords_to_render(offset, *x), false))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    layer.add_polygon(Polygon {
        rings: polygons,
        mode: PaintMode::Stroke,
        winding_order: WindingOrder::NonZero,
    });
}

fn draw_text(
    layer: &mut PdfLayerReference,
    offset: Point,
    text: &TextChunk<'_, '_, IndirectFontRef>,
) {
    let origin = text_coords_to_render(offset, text.rect.lower_left());
    layer.use_text(
        text.text.clone(),
        text.font_size,
        Mm::from(origin.x),
        Mm::from(origin.y),
        text.font.font_ref(),
    );
}

fn text_coords_to_render(offset: Point, text_pos: Vector2F) -> Point {
    let height_in_pt = Pt::from(Mm(CARD_HEIGHT_INNER)).0;
    let x = offset.x.0 + text_pos.x() + mm_to_pt(MARGIN);
    let y = offset.y.0 + height_in_pt - text_pos.y() + mm_to_pt(MARGIN);
    Point::new(Mm::from(Pt(x)), Mm::from(Pt(y)))
}

fn mm_to_pt(x: f32) -> f32 {
    Pt::from(Mm(x)).0
}
