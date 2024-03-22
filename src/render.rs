use crate::rich_text::{
    Font, JustifyContent, Layout, RichText, RichTextLayoutBuilder, RichTextPart, TextChunk,
};
use crate::spell::{Property, Spell};
use anyhow::{anyhow, Result};
use font_kit::properties::{Properties, Weight};
use pathfinder_geometry::rect::RectF;
use pathfinder_geometry::vector::Vector2F;
use printpdf::Rect;
use printpdf::{
    path::{PaintMode, WindingOrder},
    Color, Line, Mm, PdfDocument, PdfLayerReference, Point, Polygon, Pt, Rgb,
};
use std::io::{BufWriter, Write};

// Everything is measured in Mm
const A4_WIDTH: f32 = 210.0;
const A4_HEIGHT: f32 = 297.0;
const CARD_WIDTH: f32 = 65.0;
const CARD_HEIGHT: f32 = 90.0;

const GRID_WIDTH: usize = 3;
const GRID_HEIGHT: usize = 3;

const X_PADDING: f32 = (A4_WIDTH - CARD_WIDTH * (GRID_WIDTH as f32)) / (GRID_WIDTH + 1) as f32;
const Y_PADDING: f32 = (A4_HEIGHT - CARD_HEIGHT * (GRID_HEIGHT as f32)) / (GRID_HEIGHT + 1) as f32;
const MARGIN: f32 = 1.0;
const CARD_WIDTH_INNER: f32 = CARD_WIDTH - 2.0 * MARGIN;
const CARD_HEIGHT_INNER: f32 = CARD_HEIGHT - 2.0 * MARGIN;

const HEADER_DISTANCE: f32 = 2.0;
const SECTION_DISTANCE: f32 = 1.0;
const TRAIT_PADDING: f32 = 0.5;
const TRAIT_Y_SPACING: f32 = 1.0;
const TRAIT_X_SPACING: f32 = 2.0;
const PROPERTY_CHUNK_SPACING: f32 = 7.0;

/// Write document containing all spells into `output`
pub fn write_to_pdf<T: Write>(output: T, spells: &[Spell]) -> Result<()> {
    let (mut doc, page1, layer1) =
        PdfDocument::new("Spells", Mm(A4_WIDTH), Mm(A4_HEIGHT), "Layer1");

    let card_font =
        Font::add_helvetica(&mut doc, false).map_err(|e| e.context("Unable to load Helvetica"))?;

    let card_font_bold =
        Font::add_helvetica(&mut doc, true).map_err(|e| e.context("Unable to load Helvetica"))?;

    let action_count_font = Font::add_external_font(&mut doc, &"static/Pathfinder2eActions.ttf")
        .map_err(|e| e.context("Unable to load Pathfinder Icons font"))?;

    let layer = doc.get_page(page1).get_layer(layer1);
    let chunks_size = GRID_HEIGHT * GRID_WIDTH;

    // Reuse card context
    let mut context = PageRenderingContext {
        action_count_font,
        layer,
        card_font,
        card_font_bold,
        offset: Point::new(Mm(0.0), Mm(0.0)),
    };

    write_page(&mut context, &spells[..chunks_size])?;
    for chunk in spells[chunks_size..].chunks(GRID_HEIGHT * GRID_WIDTH) {
        let (page_index, layer_index) = doc.add_page(Mm(A4_WIDTH), Mm(A4_HEIGHT), "Layer");
        context.layer = doc.get_page(page_index).get_layer(layer_index);
        write_page(&mut context, chunk)?;
    }

    doc.save(&mut BufWriter::new(output))?;
    Ok(())
}

/// Holds all nessesary references needed to draw single spell card.
struct PageRenderingContext {
    card_font: Font,
    card_font_bold: Font,
    action_count_font: Font,
    layer: PdfLayerReference,
    offset: Point,
}

/// Fill page with `spells`
fn write_page(layer: &mut PageRenderingContext, spells: &[Spell]) -> Result<()> {
    layer
        .layer
        .set_outline_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)));
    layer.layer.set_outline_thickness(0.0);
    let position = (0..GRID_HEIGHT).flat_map(|y| (0..GRID_WIDTH).map(move |x| (x, y)));
    for ((x, y), spell) in position.zip(spells) {
        let y = 2 - y;
        let offset = Point::new(
            Mm(X_PADDING + (CARD_WIDTH + X_PADDING) * x as f32),
            Mm(Y_PADDING + (CARD_HEIGHT + Y_PADDING) * y as f32),
        );
        layer.offset = offset;
        write_spell(layer, spell)?;
    }
    Ok(())
}

/// Write spell
fn write_spell(layer: &mut PageRenderingContext, spell: &Spell) -> Result<()> {
    draw_border(&mut layer.layer, &layer.offset);
    let mut height = 0.0;
    height += draw_header(layer, height, spell);
    height += draw_traits(layer, height, spell);
    for property in &spell.properties {
        height += draw_property(layer, height, property);
    }
    height += draw_separator_line(layer, height);
    height += draw_description(layer, height, spell);

    if height >= mm_to_pt(CARD_HEIGHT_INNER) {
        Err(anyhow!(
            "Spell `{spell_name}` does not fit card format!",
            spell_name = spell.name
        ))
    } else {
        Ok(())
    }
}

fn draw_header(layer: &mut PageRenderingContext, height: f32, spell: &Spell) -> f32 {
    let text = RichText {
        parts: vec![
            RichTextPart {
                text: spell.name.clone(),
                font: &layer.card_font,
                font_size: 12.0,
            },
            RichTextPart {
                text: "2".to_string(),
                font: &layer.action_count_font,
                font_size: 14.0,
            },
            RichTextPart {
                text: format!("{}", spell.level),
                font: &layer.card_font,
                font_size: 12.0,
            },
        ],
    };
    let layout = default_layout()
        .with_justify(JustifyContent::JustifyEven)
        .build(&text);
    draw_layouted(&mut layer.layer, layer.offset, height, &layout) + mm_to_pt(HEADER_DISTANCE)
}

fn draw_traits(layer: &mut PageRenderingContext, height: f32, spell: &Spell) -> f32 {
    let text = RichText {
        parts: spell
            .traits
            .iter()
            .map(|t| RichTextPart {
                text: t.clone(),
                font: &layer.card_font,
                font_size: 8.0,
            })
            .collect(),
    };
    let layout = default_layout()
        .with_line_spacing(mm_to_pt(TRAIT_Y_SPACING))
        .with_chunk_spacing(TRAIT_X_SPACING)
        .with_chunk_padding(mm_to_pt(TRAIT_PADDING))
        .build(&text);
    draw_trait_text(&mut layer.layer, layer.offset, height, &layout) + mm_to_pt(SECTION_DISTANCE)
}

fn draw_property(layer: &mut PageRenderingContext, height: f32, property: &Property) -> f32 {
    let text = RichText {
        parts: vec![
            RichTextPart {
                text: property.name.clone(),
                font: &layer.card_font_bold,
                font_size: 9.0,
            },
            RichTextPart {
                text: property.value.clone(),
                font: &layer.card_font,
                font_size: 9.0,
            },
        ],
    };
    let layout = default_layout()
        .with_chunk_spacing(PROPERTY_CHUNK_SPACING)
        .build(&text);
    draw_layouted(&mut layer.layer, layer.offset, height, &layout)
}

fn draw_separator_line(layer: &mut PageRenderingContext, mut height: f32) -> f32 {
    height += mm_to_pt(SECTION_DISTANCE);
    let points = vec![
        (
            text_coords_to_render(layer.offset, Vector2F::new(0.0, height)),
            false,
        ),
        (
            text_coords_to_render(
                layer.offset,
                Vector2F::new(mm_to_pt(CARD_WIDTH_INNER), height),
            ),
            false,
        ),
    ];
    let poly = Line {
        points,
        is_closed: false,
    };
    layer.layer.add_line(poly);
    mm_to_pt(SECTION_DISTANCE * 2.0)
}

fn draw_description(layer: &mut PageRenderingContext, height: f32, spell: &Spell) -> f32 {
    0.0
}

fn draw_heightened(layer: &mut PageRenderingContext, height: f32, spell: &Spell) -> f32 {
    0.0
}

fn default_layout<'a>() -> RichTextLayoutBuilder<'a> {
    RichTextLayoutBuilder::new(mm_to_pt(CARD_WIDTH_INNER)).with_chunk_spacing(5.0)
}

fn draw_layouted(
    layer: &mut PdfLayerReference,
    offset: Point,
    height: f32,
    layout: &Layout,
) -> f32 {
    for line in &layout.lines {
        for chunk in line {
            draw_text(layer, offset, height, chunk);
        }
    }

    layout.height
}

fn draw_trait_text(
    layer: &mut PdfLayerReference,
    offset: Point,
    height: f32,
    layout: &Layout,
) -> f32 {
    for line in &layout.lines {
        for chunk in line {
            draw_text(layer, offset, height, chunk);
            let rect = RectF::new(
                chunk.rect.origin() + Vector2F::new(0.0, height),
                chunk.rect.size(),
            );
            draw_bounding_box(layer, offset, rect.dilate(mm_to_pt(TRAIT_PADDING)));
        }
    }

    layout.height
}

fn draw_bounding_box(layer: &mut PdfLayerReference, offset: Point, rect: RectF) {
    let bottom_left = text_coords_to_render(offset, rect.lower_left());
    let top_right = text_coords_to_render(offset, rect.upper_right());
    let bottom_right = Point {
        x: top_right.x,
        y: bottom_left.y,
    };
    let top_left = Point {
        y: top_right.y,
        x: bottom_left.x,
    };

    let points = [bottom_left, bottom_right, top_right, top_left];
    let points = points.into_iter().map(|x| (x, false)).collect();
    let poly = Polygon {
        rings: vec![points],
        mode: PaintMode::Stroke,
        winding_order: WindingOrder::NonZero,
    };
    layer.add_polygon(poly);
}

fn draw_text(layer: &mut PdfLayerReference, offset: Point, height: f32, text: &TextChunk) {
    let origin = text_coords_to_render(offset, text.rect.lower_left() + Vector2F::new(0.0, height));
    layer.use_text(
        text.text,
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

fn draw_border(layer: &mut PdfLayerReference, offset: &Point) {
    let points = [
        offset.clone(),
        Point::new(Mm::from(offset.x) + Mm(CARD_WIDTH), Mm::from(offset.y)),
        Point::new(
            Mm::from(offset.x) + Mm(CARD_WIDTH),
            Mm::from(offset.y) + Mm(CARD_HEIGHT),
        ),
        Point::new(Mm::from(offset.x), Mm::from(offset.y) + Mm(CARD_HEIGHT)),
    ];
    let points = points.into_iter().map(|x| (x, false)).collect();
    let poly = Polygon {
        rings: vec![points],
        mode: PaintMode::Stroke,
        winding_order: WindingOrder::NonZero,
    };
    layer.add_polygon(poly);
}

fn mm_to_pt(x: f32) -> f32 {
    Pt::from(Mm(x)).0
}
