//! Page imposition: mapping reader-order page numbers onto the physical sheet layout that,
//! once printed, folded, and (for the one-page zine) cut, reproduces the pages in the right
//! order and orientation.
//!
//! Two schemes are supported:
//! - [`impose_one_page_zine`] — the classic single-sheet, single-sided 8-panel mini zine.
//! - [`impose_booklet`] — a saddle-stitch style booklet (nested, folded-in-half sheets),
//!   either duplex (the real thing) or a simpler single-sided variant for anyone without
//!   access to a duplex printer.

use serde::{Deserialize, Serialize};

/// One page-sized cell on a sheet side. `page` is the 1-indexed reader page number, or `None`
/// for a blank panel that isn't part of the page sequence at all (only ever appears on the
/// one-page zine's fixed 8-panel grid if fewer than 8 images end up in the project — the
/// booklet scheme always pads with real blank *pages* first, via [`pad_to_multiple`], so its
/// slots are always `Some`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Slot {
    pub page: Option<usize>,
    pub rotated: bool,
}

/// One printable side of one physical sheet: a row-major grid of [`Slot`]s.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetSide {
    pub rows: usize,
    pub cols: usize,
    pub slots: Vec<Slot>,
}

/// One physical sheet. `back` is `None` for a single-sided sheet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sheet {
    pub front: SheetSide,
    pub back: Option<SheetSide>,
}

/// Rounds `page_count` up to the next multiple of `multiple` (unchanged if already a
/// multiple). Used to work out how many blank pages a booklet needs appended before it can be
/// imposed — a saddle-stitch signature only works out evenly in multiples of 4.
pub fn pad_to_multiple(page_count: usize, multiple: usize) -> usize {
    let rem = page_count % multiple;
    if rem == 0 {
        page_count
    } else {
        page_count + (multiple - rem)
    }
}

/// The classic single-sheet, single-sided, 8-panel "one-page zine": fold the sheet into
/// eighths, slit the center fold between the two middle columns, and it opens into an 8-page
/// booklet with no stapling. Always exactly one sheet, always exactly 8 pages (pad the
/// project's page list to 8 with blanks first if it has fewer).
///
/// Grid (2 rows x 4 columns) verified against pdfimpose's reference "onepagezine" scheme
/// output (a mature, widely-used open-source imposition tool —
/// framagit.org/spalax/pdfimpose), not derived from scratch: the top row is upside down
/// relative to the bottom row on the printed sheet, which is correct — the fold sequence
/// brings it right-side up in the assembled booklet.
///
/// ```text
///              col1  col2  col3  col4
/// row1 (180°):   5     4     3     2
/// row2 (0°):     6     7     8     1
/// ```
pub fn impose_one_page_zine() -> Sheet {
    Sheet { front: eight_panel_side(&[5, 4, 3, 2], true, &[6, 7, 8, 1], false), back: None }
}

/// The classic 16-page mini-zine: the same one-sheet fold as [`impose_one_page_zine`], but
/// printed on BOTH sides — 8 physical panel positions x 2 faces = 16 reader pages, no
/// stapling, cut + fold only. Grid verified against a real, working reference implementation
/// — github.com/colarusso/mini-zine's index.html/zine_text.js, which hardcodes this exact
/// front/back page numbering (and confirms every one of the 4 attempts I made deriving this
/// from the fold mechanics alone was wrong — this is sourced, not derived).
/// Print instructions from that source: portrait content, two-sided, long-edge binding, on a
/// **landscape** sheet (11in x 8.5in / ~297x210mm) — not portrait paper like the 8-page
/// version defaults to. See [`crate::project::Format::sheet_orientation`].
///
/// ```text
/// FRONT            col1  col2  col3  col4      BACK             col1  col2  col3  col4
/// row1 (0°):         16    1     4    13        row1 (0°):         10    7     6    11
/// row2 (180°):        9    8     5    12        row2 (180°):       15    2     3    14
/// ```
pub fn impose_sixteen_page_zine() -> Sheet {
    Sheet {
        front: eight_panel_side(&[16, 1, 4, 13], false, &[9, 8, 5, 12], true),
        back: Some(eight_panel_side(&[10, 7, 6, 11], false, &[15, 2, 3, 14], true)),
    }
}

/// Shared by both one-sheet zine schemes: a 2x4 grid built from a top and bottom row, each
/// independently rotated or not — [`impose_one_page_zine`] rotates its top row, while
/// [`impose_sixteen_page_zine`] rotates its bottom row instead (confirmed against each
/// scheme's own verified source, not assumed to match between the two).
fn eight_panel_side(top: &[usize; 4], top_rotated: bool, bottom: &[usize; 4], bottom_rotated: bool) -> SheetSide {
    let slots = top
        .iter()
        .map(|&p| Slot { page: Some(p), rotated: top_rotated })
        .chain(bottom.iter().map(|&p| Slot { page: Some(p), rotated: bottom_rotated }))
        .collect();
    SheetSide { rows: 2, cols: 4, slots }
}

/// How a multi-sheet booklet's sheets get printed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BookletSide {
    /// The real saddle-stitch booklet: each sheet printed front and back, folded in half, and
    /// nested inside the others before stapling through the spine. Needs a duplex printer (or
    /// careful manual re-feeding).
    Duplex,
    /// No duplex printer needed: each sheet printed on one side only, in plain sequential
    /// reading order (no nesting remap) — pages 1-2 on the first sheet, 3-4 on the second, and
    /// so on. Fold each one in half and gather them as a simple stapled stack rather than
    /// nesting — a different (simpler, but not "real" saddle-stitch) physical assembly, since
    /// a nested signature depends on having back-side content to fold the outer sheets around.
    /// Not specified further than "single side" in the request — this is the simplest
    /// interpretation, and the one that needs no special equipment at all.
    Single,
}

/// Imposes a booklet of `page_count` reader pages (already padded — see [`pad_to_multiple`])
/// as a saddle-stitch signature.
///
/// Duplex: standard nested-fold-and-staple imposition (the same scheme Adobe Acrobat's
/// "Booklet" print option and every other booklet-imposition tool use) — `page_count` must be
/// a multiple of 4. For sheet `k` (0-indexed, `k = 0` is the outermost sheet, i.e. the cover):
///
/// ```text
/// front-left = page_count - 2k     front-right = 1 + 2k
/// back-left  = 2 + 2k               back-right  = page_count - 1 - 2k
/// ```
///
/// Single: `page_count` must be even. Sheet `k` just holds reader pages `2k+1` and `2k+2`
/// side by side, printed once, no back side, no reordering.
pub fn impose_booklet(page_count: usize, side: BookletSide) -> Vec<Sheet> {
    match side {
        BookletSide::Duplex => {
            assert!(
                page_count % 4 == 0,
                "duplex booklet page count must be a multiple of 4 (pad_to_multiple first)"
            );
            let sheets = page_count / 4;
            (0..sheets)
                .map(|k| {
                    let front_left = page_count - 2 * k;
                    let front_right = 1 + 2 * k;
                    let back_left = 2 + 2 * k;
                    let back_right = page_count - 1 - 2 * k;
                    Sheet {
                        front: two_up(front_left, front_right),
                        back: Some(two_up(back_left, back_right)),
                    }
                })
                .collect()
        }
        BookletSide::Single => {
            assert!(
                page_count % 2 == 0,
                "single-sided booklet page count must be even (pad_to_multiple first)"
            );
            let sheets = page_count / 2;
            (0..sheets)
                .map(|k| Sheet {
                    front: two_up(2 * k + 1, 2 * k + 2),
                    back: None,
                })
                .collect()
        }
    }
}

fn two_up(left: usize, right: usize) -> SheetSide {
    SheetSide {
        rows: 1,
        cols: 2,
        slots: vec![
            Slot { page: Some(left), rotated: false },
            Slot { page: Some(right), rotated: false },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pad_to_multiple_already_aligned() {
        assert_eq!(pad_to_multiple(8, 4), 8);
        assert_eq!(pad_to_multiple(0, 4), 0);
    }

    #[test]
    fn pad_to_multiple_rounds_up() {
        assert_eq!(pad_to_multiple(9, 4), 12);
        assert_eq!(pad_to_multiple(1, 4), 4);
        assert_eq!(pad_to_multiple(13, 4), 16);
    }

    #[test]
    fn one_page_zine_matches_reference_grid() {
        let sheet = impose_one_page_zine();
        assert!(sheet.back.is_none());
        assert_eq!(sheet.front.rows, 2);
        assert_eq!(sheet.front.cols, 4);
        let pages: Vec<Option<usize>> = sheet.front.slots.iter().map(|s| s.page).collect();
        assert_eq!(
            pages,
            vec![
                Some(5), Some(4), Some(3), Some(2),
                Some(6), Some(7), Some(8), Some(1),
            ]
        );
        // Top row (indices 0-3) is rotated, bottom row (4-7) is not.
        assert!(sheet.front.slots[0..4].iter().all(|s| s.rotated));
        assert!(sheet.front.slots[4..8].iter().all(|s| !s.rotated));
        // Every reader page 1-8 appears exactly once.
        let mut seen: Vec<usize> = pages.into_iter().flatten().collect();
        seen.sort_unstable();
        assert_eq!(seen, (1..=8).collect::<Vec<_>>());
    }

    #[test]
    fn sixteen_page_zine_matches_reference_grid() {
        let sheet = impose_sixteen_page_zine();
        let front: Vec<Option<usize>> = sheet.front.slots.iter().map(|s| s.page).collect();
        assert_eq!(
            front,
            vec![
                Some(16), Some(1), Some(4), Some(13),
                Some(9), Some(8), Some(5), Some(12),
            ]
        );
        // Top row un-rotated, bottom row rotated — opposite of the 8-page scheme.
        assert!(sheet.front.slots[0..4].iter().all(|s| !s.rotated));
        assert!(sheet.front.slots[4..8].iter().all(|s| s.rotated));

        let back_side = sheet.back.expect("16-page zine is double-sided");
        let back: Vec<Option<usize>> = back_side.slots.iter().map(|s| s.page).collect();
        assert_eq!(
            back,
            vec![
                Some(10), Some(7), Some(6), Some(11),
                Some(15), Some(2), Some(3), Some(14),
            ]
        );
        assert!(back_side.slots[0..4].iter().all(|s| !s.rotated));
        assert!(back_side.slots[4..8].iter().all(|s| s.rotated));

        // Every reader page 1-16 appears exactly once across both sides.
        let mut seen: Vec<usize> = front.into_iter().chain(back).flatten().collect();
        seen.sort_unstable();
        assert_eq!(seen, (1..=16).collect::<Vec<_>>());
    }

    #[test]
    fn duplex_booklet_eight_pages_two_sheets() {
        let sheets = impose_booklet(8, BookletSide::Duplex);
        assert_eq!(sheets.len(), 2);

        // Outermost sheet (the cover): front shows the back cover (8) and front cover (1).
        let s0 = &sheets[0];
        assert_eq!(page_at(&s0.front, 0), 8);
        assert_eq!(page_at(&s0.front, 1), 1);
        let s0_back = s0.back.as_ref().unwrap();
        assert_eq!(page_at(s0_back, 0), 2);
        assert_eq!(page_at(s0_back, 1), 7);

        // Innermost sheet (the center spread, pages 4-5, ends up back-left/back-right).
        let s1 = &sheets[1];
        assert_eq!(page_at(&s1.front, 0), 6);
        assert_eq!(page_at(&s1.front, 1), 3);
        let s1_back = s1.back.as_ref().unwrap();
        assert_eq!(page_at(s1_back, 0), 4);
        assert_eq!(page_at(s1_back, 1), 5);
    }

    #[test]
    fn duplex_booklet_covers_every_page_exactly_once() {
        for &n in &[4usize, 8, 12, 16, 24, 32] {
            let sheets = impose_booklet(n, BookletSide::Duplex);
            assert_eq!(sheets.len(), n / 4);
            let mut seen = Vec::new();
            for sheet in &sheets {
                seen.extend(sheet.front.slots.iter().filter_map(|s| s.page));
                seen.extend(sheet.back.as_ref().unwrap().slots.iter().filter_map(|s| s.page));
            }
            seen.sort_unstable();
            assert_eq!(seen, (1..=n).collect::<Vec<_>>(), "page_count={n}");
        }
    }

    #[test]
    #[should_panic(expected = "multiple of 4")]
    fn duplex_booklet_rejects_non_multiple_of_four() {
        impose_booklet(6, BookletSide::Duplex);
    }

    #[test]
    fn single_sided_booklet_is_plain_sequential() {
        let sheets = impose_booklet(6, BookletSide::Single);
        assert_eq!(sheets.len(), 3);
        assert!(sheets.iter().all(|s| s.back.is_none()));
        let seen: Vec<usize> = sheets
            .iter()
            .flat_map(|s| s.front.slots.iter().filter_map(|slot| slot.page))
            .collect();
        assert_eq!(seen, vec![1, 2, 3, 4, 5, 6]);
    }

    fn page_at(side: &SheetSide, index: usize) -> usize {
        side.slots[index].page.expect("slot should be filled")
    }
}
