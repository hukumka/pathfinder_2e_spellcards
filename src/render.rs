use crate::spell::{Property, Spell};
use anyhow::{anyhow, Result};
use printpdf::{
    path::{PaintMode, WindingOrder},
    Color, IndirectFontRef, Mm, PdfDocument, PdfLayerReference, Point, Polygon, Rgb,
};
use std::fs::File;
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

/// Write document containing all spells into `output`
pub fn write_to_pdf<T: Write>(output: T, spells: &[Spell]) -> Result<()> {
    let (doc, page1, layer1) = PdfDocument::new("Spells", Mm(A4_WIDTH), Mm(A4_HEIGHT), "Layer1");

    let card_font = doc.add_builtin_font(printpdf::BuiltinFont::Helvetica)?;
    let action_count_font = doc.add_external_font(File::open("static/Pathfinder2eActions.ttf")?)?;

    let layer = doc.get_page(page1).get_layer(layer1);
    let chunks_size = GRID_HEIGHT * GRID_WIDTH;

    // Reuse card context
    let mut context = PageRenderingContext {
        card_font,
        action_count_font,
        layer,
        offset: Point::new(Mm(0.0), Mm(0.0)),
    };

    write_page(&mut context, &spells[..chunks_size])?;
    for chunk in spells.chunks(GRID_HEIGHT * GRID_WIDTH) {
        let (page_index, layer_index) = doc.add_page(Mm(A4_WIDTH), Mm(A4_HEIGHT), "Layer");
        context.layer = doc.get_page(page_index).get_layer(layer_index);
        write_page(&mut context, chunk)?;
    }

    doc.save(&mut BufWriter::new(output))?;
    Ok(())
}

/// Holds all nessesary references needed to draw single spell card.
struct PageRenderingContext {
    card_font: IndirectFontRef,
    action_count_font: IndirectFontRef,
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
    let mut height = draw_header(layer, spell);
    height += draw_traits(layer, spell, height);
    for property in &spell.properties {
        height += draw_property(layer, property, height);
    }
    height += draw_separator_line(layer, height);
    height += draw_description(layer, spell, height);
    if height >= CARD_HEIGHT_INNER {
        Err(anyhow!(
            "Spell `{spell_name}` does not fit card format!",
            spell_name = spell.name
        ))
    } else {
        Ok(())
    }
}

fn draw_header(layer: &mut PageRenderingContext, spell: &Spell) -> f32 {
    0.0
}

fn draw_traits(layer: &mut PageRenderingContext, spell: &Spell, y_offset: f32) -> f32 {
    0.0
}

fn draw_property(layer: &mut PageRenderingContext, property: &Property, y_offset: f32) -> f32 {
    0.0
}

fn draw_separator_line(layer: &mut PageRenderingContext, y_offset: f32) -> f32 {
    0.0
}

fn draw_description(layer: &mut PageRenderingContext, spell: &Spell, y_offset: f32) -> f32 {
    0.0
}

fn draw_heightened(layer: &mut PageRenderingContext, spell: &Spell, y_offset: f32) -> f32 {
    0.0
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

struct RichText {}

struct RichTextPart<'a> {
    font: &'a FontDescriptop,
}

struct FontDescriptor {
    font: IndirectFontRef,
}
