use crate::markdown::MdConfig;
use crate::rich_text::{AlignStrategy, Font, Scene, SceneBuilder, TextChunk};
use crate::spell::Spell;
use anyhow::{anyhow, Result};
use pathfinder_geometry::rect::RectF;
use pathfinder_geometry::vector::Vector2F;
use printpdf::{
    path::{PaintMode, WindingOrder},
    Color, Mm, PdfDocument, PdfLayerReference, Point, Polygon, Pt, Rgb,
};
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

#[derive(Copy, Clone)]
struct FontConfig<'a> {
    md_config: MdConfig<'a>,
    action_count_font: &'a Font,
}

/// Write document containing all spells into `output`
pub fn write_to_pdf<T: Write>(output: T, spells: &[Spell]) -> Result<()> {
    let (mut doc, page1, layer1) =
        PdfDocument::new("Spells", Mm(A4_WIDTH), Mm(A4_HEIGHT), "Layer1");

    let text_font = Font::add_helvetica(&mut doc, printpdf::BuiltinFont::Helvetica)
        .map_err(|e| e.context("Unable to load Helvetica"))?;

    let font_bold = Font::add_helvetica(&mut doc, printpdf::BuiltinFont::HelveticaBold)
        .map_err(|e| e.context("Unable to load Helvetica"))?;

    let italic_font = Font::add_helvetica(&mut doc, printpdf::BuiltinFont::HelveticaOblique)
        .map_err(|e| e.context("Unable to load Helvetica"))?;

    let action_count_font = Font::add_external_font(&mut doc, &"static/Pathfinder2eActions.ttf")
        .map_err(|e| e.context("Unable to load Pathfinder Icons font"))?;

    let font_config = FontConfig {
        md_config: MdConfig {
            text_font: &text_font,
            bold_font: &font_bold,
            italic_font: &italic_font,
        },
        action_count_font: &action_count_font,
    };

    let mut layer = doc.get_page(page1).get_layer(layer1);

    let mut errors = vec![];

    let positions = (0..GRID_HEIGHT)
        .flat_map(|y| (0..GRID_WIDTH).map(move |x| (x, y)))
        .collect::<Vec<_>>();
    let mut positions_iter = positions.iter().cloned();
    let mut position = positions_iter.next().unwrap();
    init_page(&mut layer);
    for spell in spells {
        if let Ok(scene) = build_spell_scene(&font_config, spell) {
            render_scene(&mut layer, position, &scene);
            if let Some(new_position) = positions_iter.next() {
                position = new_position;
            } else {
                // Start new page
                let (page_index, layer_index) = doc.add_page(Mm(A4_WIDTH), Mm(A4_HEIGHT), "Layer");
                layer = doc.get_page(page_index).get_layer(layer_index);
                init_page(&mut layer);
                positions_iter = positions.iter().cloned();
                position = positions_iter.next().unwrap();
            }
        } else {
            errors.push(spell.name.clone());
        }
    }

    doc.save(&mut BufWriter::new(output))?;
    if errors.is_empty() {
        Ok(())
    } else {
        Err(anyhow!("failed spells: {:#?}", errors))
    }
}

fn init_page(layer: &mut PdfLayerReference) {
    layer.set_outline_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)));
    layer.set_outline_thickness(0.0);
}

/// Write spell
fn build_spell_scene<'a>(config: &'a FontConfig<'a>, spell: &'a Spell) -> Result<Scene<'a>> {
    let rect = RectF::new(
        Vector2F::zero(),
        Vector2F::new(mm_to_pt(CARD_WIDTH_INNER), mm_to_pt(CARD_HEIGHT_INNER)),
    );
    let mut builder = SceneBuilder::<'a>::new(&config.md_config.text_font, rect);
    builder.add_rect(rect.dilate(mm_to_pt(MARGIN) + 1.0));

    builder
        .set_line_space(mm_to_pt(HEADER_LINE_SPACE))
        // Draw header
        .set_alignment(AlignStrategy::JustifyEven)
        .set_font_size(12.0) // Name
        .add_text(&spell.name);

    if let Some(action) = spell.actions.as_str() {
        builder
            .set_font_size(14.0)
            .set_font(config.action_count_font) // Action count;
            .add_text(action)
            .set_font(config.md_config.text_font);
    }
    builder
        .set_font_size(12.0) // Spell level
        .add_text(format!("{}", spell.level))
        .finish_line();

    // Draw traits
    builder
        .set_line_space(mm_to_pt(LINE_SPACE))
        .set_font_size(8.0)
        .set_chunk_space(mm_to_pt(TRAIT_CHUNK_SPACE))
        .set_alignment(AlignStrategy::AlignLeft);
    for trait_ in &spell.traits {
        builder.add_boxed_text(trait_.as_str(), mm_to_pt(TRAIT_PADDING));
    }
    builder
        .set_font_size(8.5)
        .set_default_chunk_space()
        .finish_line();
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
    builder.set_font_size(8.5);
    builder.add_markdown(&config.md_config, &spell.description);
    if let Some(heighened) = &spell.heightened {
        builder.add_separator_line();
        builder
            .add_markdown(&config.md_config, heighened.as_str())
            .finish_line();
    }

    if builder.is_out_of_bounds() {
        Err(anyhow!(
            "Spell `{spell_name}` does not fit card format!",
            spell_name = spell.name
        ))
    } else {
        Ok(builder.scene())
    }
}

fn render_scene(layer: &mut PdfLayerReference, (x, y): (usize, usize), scene: &Scene) {
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

fn draw_text(layer: &mut PdfLayerReference, offset: Point, text: &TextChunk) {
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
