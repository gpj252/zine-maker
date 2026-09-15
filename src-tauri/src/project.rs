//! The project data model — what the frontend sends over `invoke()` and what gets persisted.

use crate::imposition::{
    impose_booklet, impose_one_page_zine, pad_to_multiple, BookletSide, Sheet,
};
use serde::{Deserialize, Serialize};

/// One entry in the reader-order page sequence. A blank page the user explicitly inserted
/// (not an auto-pad at export time — see [ImpositionResult::auto_blanks_added]) is just an
/// entry with `image_path: None`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageEntry {
    pub id: String,
    pub image_path: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Margins {
    pub top_mm: f64,
    pub right_mm: f64,
    pub bottom_mm: f64,
    pub left_mm: f64,
}

impl Default for Margins {
    fn default() -> Self {
        Margins { top_mm: 5.0, right_mm: 5.0, bottom_mm: 5.0, left_mm: 5.0 }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PaperSize {
    pub width_mm: f64,
    pub height_mm: f64,
}

impl PaperSize {
    pub const A4: PaperSize = PaperSize { width_mm: 210.0, height_mm: 297.0 };
    pub const LETTER: PaperSize = PaperSize { width_mm: 215.9, height_mm: 279.4 };
}

/// The imposition scheme a project uses. `OnePageZine` is always exactly 8 pages on one
/// single-sided sheet; `Booklet` covers both the "Zine" (booklet-style, user-chosen page
/// count) and "Comic" project types — they're the same underlying imposition, just offered
/// with different preset page counts in the UI (see the frontend's format picker).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Format {
    OnePageZine,
    Booklet { page_count: usize, side: BookletSide },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub format: Format,
    pub paper: PaperSize,
    pub margins: Margins,
    /// Reader order, including any blanks the user explicitly inserted. NOT padded to the
    /// format's required multiple — see [compute_imposition], which pads on the fly.
    pub pages: Vec<PageEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpositionResult {
    pub sheets: Vec<Sheet>,
    /// How many extra blank pages had to be appended (beyond whatever's already in
    /// `pages`) to reach a valid multiple for the format — surfaced in the UI ("+2 blank
    /// pages added to complete the booklet") rather than silently padding.
    pub auto_blanks_added: usize,
}

/// Runs the format's imposition over the project's current page list, auto-padding with
/// trailing blanks first if the count isn't already a valid multiple for the scheme. This is
/// meant to be cheap enough to call on every page-list edit (add/remove/reorder a page, insert
/// a blank) — there's no separate "recompute" step, the frontend just re-invokes this and
/// re-renders the preview.
pub fn compute_imposition(project: &Project) -> ImpositionResult {
    match &project.format {
        Format::OnePageZine => {
            let padded = pad_to_multiple(project.pages.len().max(1), 8).max(8);
            ImpositionResult {
                sheets: vec![impose_one_page_zine()],
                auto_blanks_added: padded - project.pages.len(),
            }
        }
        Format::Booklet { side, .. } => {
            let multiple = match side {
                BookletSide::Duplex => 4,
                BookletSide::Single => 2,
            };
            let padded = pad_to_multiple(project.pages.len(), multiple);
            ImpositionResult {
                sheets: impose_booklet(padded, *side),
                auto_blanks_added: padded - project.pages.len(),
            }
        }
    }
}
