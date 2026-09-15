//! Renders an [`crate::imposition::Sheet`] list to a print-ready PDF — one PDF page per
//! physical sheet SIDE (so a duplex-imposed booklet's PDF is meant to be printed "flip on
//! short/long edge, duplex" straight from a PDF reader).
//!
//! NOTE: the exact positioning/rotation math below (translate vs. rotation pivot, in
//! particular) is written against printpdf 0.7's documented API but hasn't been visually
//! verified against a real rendered PDF yet — do that (export a one-page zine, check the 4
//! rotated panels land right-side-up in the right spot after folding) before trusting this
//! for an actual print job.

use crate::imposition::{Sheet, SheetSide};
use crate::project::PageEntry;
use printpdf::{
    Image, ImageRotation, ImageTransform, Mm, PdfDocument, PdfLayerReference, Px,
};
use std::fs::File;
use std::io::BufWriter;

const IMAGE_DPI: f32 = 300.0;

pub fn export_pdf(
    pages: &[PageEntry],
    sheets: &[Sheet],
    paper_width_mm: f64,
    paper_height_mm: f64,
    margins: (f64, f64, f64, f64), // top, right, bottom, left
    output_path: &str,
) -> Result<(), String> {
    if sheets.is_empty() {
        return Err("nothing to export — the imposition produced no sheets".into());
    }

    let (doc, first_page, first_layer) = PdfDocument::new(
        "Zine",
        Mm(paper_width_mm as f32),
        Mm(paper_height_mm as f32),
        "Layer",
    );

    // One PDF page per sheet side; the first side reuses the page PdfDocument::new already
    // created, every side after that needs its own add_page call.
    let total_sides: usize = sheets.iter().map(|s| if s.back.is_some() { 2 } else { 1 }).sum();
    let mut layers = vec![doc.get_page(first_page).get_layer(first_layer)];
    for _ in 1..total_sides {
        let (page, layer) =
            doc.add_page(Mm(paper_width_mm as f32), Mm(paper_height_mm as f32), "Layer");
        layers.push(doc.get_page(page).get_layer(layer));
    }

    let mut side_index = 0;
    for sheet in sheets {
        render_side(&layers[side_index], &sheet.front, pages, paper_width_mm, paper_height_mm, margins)?;
        side_index += 1;
        if let Some(back) = &sheet.back {
            render_side(&layers[side_index], back, pages, paper_width_mm, paper_height_mm, margins)?;
            side_index += 1;
        }
    }

    let file = File::create(output_path).map_err(|e| format!("couldn't create {output_path}: {e}"))?;
    doc.save(&mut BufWriter::new(file)).map_err(|e| format!("couldn't write PDF: {e}"))?;
    Ok(())
}

fn render_side(
    layer: &PdfLayerReference,
    side: &SheetSide,
    pages: &[PageEntry],
    paper_w: f64,
    paper_h: f64,
    (margin_top, margin_right, margin_bottom, margin_left): (f64, f64, f64, f64),
) -> Result<(), String> {
    let usable_w = paper_w - margin_left - margin_right;
    let usable_h = paper_h - margin_top - margin_bottom;
    let cell_w = usable_w / side.cols as f64;
    let cell_h = usable_h / side.rows as f64;

    for (i, slot) in side.slots.iter().enumerate() {
        let Some(page_num) = slot.page else { continue };
        let Some(entry) = pages.get(page_num - 1) else { continue };
        let Some(image_path) = &entry.image_path else { continue }; // a blank page: leave the cell empty

        let row = i / side.cols;
        let col = i % side.cols;
        let cell_x = margin_left + col as f64 * cell_w;
        // PDF's origin is bottom-left; grid row 0 is the sheet's TOP row.
        let cell_y = margin_bottom + usable_h - (row as f64 + 1.0) * cell_h;

        draw_fitted(layer, image_path, cell_x, cell_y, cell_w, cell_h, slot.rotated)?;
    }
    Ok(())
}

/// Draws `image_path` into the `cell_w` x `cell_h` box at `(cell_x, cell_y)` (bottom-left
/// corner, in mm), scaled to fit without cropping (letterboxed, not filled — the user's page
/// image is never cut off) and centered in the cell.
fn draw_fitted(
    layer: &PdfLayerReference,
    image_path: &str,
    cell_x: f64,
    cell_y: f64,
    cell_w: f64,
    cell_h: f64,
    rotated: bool,
) -> Result<(), String> {
    let dynamic_image =
        image::open(image_path).map_err(|e| format!("couldn't open {image_path}: {e}"))?;
    let px_w = dynamic_image.width();
    let px_h = dynamic_image.height();

    let aspect_img = px_w as f64 / px_h as f64;
    let aspect_cell = cell_w / cell_h;
    let (draw_w, draw_h) = if aspect_img > aspect_cell {
        (cell_w, cell_w / aspect_img)
    } else {
        (cell_h * aspect_img, cell_h)
    };
    let offset_x = cell_x + (cell_w - draw_w) / 2.0;
    let offset_y = cell_y + (cell_h - draw_h) / 2.0;

    // native size (mm) at IMAGE_DPI, i.e. the size printpdf draws the image at scale_x/y = 1.0
    let native_w_mm = px_w as f64 / IMAGE_DPI as f64 * 25.4;
    let native_h_mm = px_h as f64 / IMAGE_DPI as f64 * 25.4;
    let scale_x = (draw_w / native_w_mm) as f32;
    let scale_y = (draw_h / native_h_mm) as f32;

    let image = Image::from_dynamic_image(&dynamic_image);
    let transform = ImageTransform {
        translate_x: Some(Mm(offset_x as f32)),
        translate_y: Some(Mm(offset_y as f32)),
        rotate: if rotated {
            Some(ImageRotation {
                angle_ccw_degrees: 180.0,
                rotation_center_x: Px((px_w / 2) as usize),
                rotation_center_y: Px((px_h / 2) as usize),
            })
        } else {
            None
        },
        scale_x: Some(scale_x),
        scale_y: Some(scale_y),
        dpi: Some(IMAGE_DPI),
    };
    image.add_to_layer(layer.clone(), transform);
    Ok(())
}
