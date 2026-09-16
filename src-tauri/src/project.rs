//! The project data model — what the frontend sends over `invoke()` and what gets persisted.

use crate::imposition::{
    impose_booklet, impose_one_page_zine, impose_sixteen_page_zine, pad_to_multiple, BookletSide,
    Sheet,
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

// Paper size presets (A4, Letter) live only on the frontend (src/types.ts) — the project just
// carries plain width_mm/height_mm here, no reason to duplicate the preset list on both sides
// and risk them drifting apart.

/// The imposition scheme a project uses. `OnePageZine`/`OnePageZinePosterBack`/
/// `SixteenPageZine` are all fixed-length, single-sheet formats (8, 8, and 16 pages
/// respectively — see imposition.rs for why the first two share a page count but not a
/// physical assembly). `Booklet` covers both the "Zine" (booklet-style, user-chosen page
/// count) and "Comic" project types — they're the same underlying imposition, just offered
/// with different preset page counts in the UI (see the frontend's format picker).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Format {
    OnePageZine,
    /// Same 8-panel front as `OnePageZine`, but the back of the sheet is one full-bleed
    /// poster image instead of 8 more panels — pdfimpose's actual "onepagezine" scheme
    /// (see imposition.rs's doc comment), not something this project invented.
    OnePageZinePosterBack,
    SixteenPageZine,
    Booklet { page_count: usize, side: BookletSide },
}

impl Format {
    /// The two one-sheet zine grids (8-panel and 16-page) are 4 columns x 2 rows — much wider
    /// than tall — and were verified against sources that print them on a landscape sheet
    /// (11x8.5in), not the portrait paper (8.5x11in) this app's paper-size presets otherwise
    /// assume. Swapping width/height for just these formats, rather than asking the paper
    /// picker to know about it, keeps "pick A4 or Letter" simple regardless of format.
    pub fn wants_landscape_sheet(&self) -> bool {
        matches!(self, Format::OnePageZine | Format::OnePageZinePosterBack | Format::SixteenPageZine)
    }

    /// Whether this format uses a full-bleed poster image (see `OnePageZinePosterBack`)
    /// rather than (or in addition to) a numbered page grid on the back of the sheet.
    pub fn has_poster_back(&self) -> bool {
        matches!(self, Format::OnePageZinePosterBack)
    }
}

/// Returns `paper` with width/height swapped if `format` wants a landscape sheet and `paper`
/// is currently taller than it is wide — a no-op if it's already landscape, or if the format
/// doesn't care. See [`Format::wants_landscape_sheet`].
pub fn effective_paper_size(format: &Format, paper: PaperSize) -> PaperSize {
    if format.wants_landscape_sheet() && paper.height_mm > paper.width_mm {
        PaperSize { width_mm: paper.height_mm, height_mm: paper.width_mm }
    } else {
        paper
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub format: Format,
    pub paper: PaperSize,
    pub margins: Margins,
    /// Reader order, including any blanks the user explicitly inserted. NOT padded to the
    /// format's required multiple — see [compute_imposition], which pads on the fly.
    pub pages: Vec<PageEntry>,
    /// Only meaningful for `Format::OnePageZinePosterBack` — the image drawn full-bleed on
    /// the sheet's back side. `None` there just leaves that side blank at export time.
    pub poster_image_path: Option<String>,
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
        // Fixed-length, single-sheet formats: the target is always exactly 8/16, never
        // rounded further up — pages beyond that (the UI is expected to cap uploads at the
        // format's limit, but this stays correct even if it doesn't) just aren't referenced
        // by the fixed slot grid, same as any other blank.
        Format::OnePageZine | Format::OnePageZinePosterBack => ImpositionResult {
            sheets: vec![impose_one_page_zine()],
            auto_blanks_added: 8usize.saturating_sub(project.pages.len()),
        },
        Format::SixteenPageZine => ImpositionResult {
            sheets: vec![impose_sixteen_page_zine()],
            auto_blanks_added: 16usize.saturating_sub(project.pages.len()),
        },
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
