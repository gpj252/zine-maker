// Mirrors src-tauri/src/{imposition,project}.rs — keep these in sync by hand, there's no
// codegen wired up yet.

export type BookletSide = "Duplex" | "Single";

export type Format =
  | { kind: "OnePageZine" }
  | { kind: "OnePageZinePosterBack" }
  | { kind: "SixteenPageZine" }
  | { kind: "Booklet"; page_count: number; side: BookletSide };

/// Mirrors Format::wants_landscape_sheet() in project.rs — the one-sheet zine grids (8-panel
/// and 16-page) are 4 columns x 2 rows and were verified against sources that print them on a
/// landscape sheet, not the portrait orientation the paper-size presets otherwise assume.
export function formatWantsLandscapeSheet(format: Format): boolean {
  return format.kind === "OnePageZine" || format.kind === "OnePageZinePosterBack" || format.kind === "SixteenPageZine";
}

/// Mirrors effective_paper_size() in project.rs.
export function effectivePaperSize(format: Format, paper: PaperSize): PaperSize {
  if (formatWantsLandscapeSheet(format) && paper.height_mm > paper.width_mm) {
    return { width_mm: paper.height_mm, height_mm: paper.width_mm };
  }
  return paper;
}

export interface PaperSize {
  width_mm: number;
  height_mm: number;
}

export const PAPER_PRESETS: Record<string, PaperSize> = {
  A4: { width_mm: 210, height_mm: 297 },
  Letter: { width_mm: 215.9, height_mm: 279.4 },
};

export interface Margins {
  top_mm: number;
  right_mm: number;
  bottom_mm: number;
  left_mm: number;
}

export interface PageEntry {
  id: string;
  image_path: string | null;
}

export interface Project {
  format: Format;
  paper: PaperSize;
  margins: Margins;
  pages: PageEntry[];
  /// Only meaningful for `{ kind: "OnePageZinePosterBack" }` — the image drawn full-bleed on
  /// the sheet's back side.
  poster_image_path: string | null;
}

export interface Slot {
  page: number | null;
  rotated: boolean;
}

export interface SheetSide {
  rows: number;
  cols: number;
  slots: Slot[];
}

export interface Sheet {
  front: SheetSide;
  back: SheetSide | null;
}

export interface ImpositionResult {
  sheets: Sheet[];
  auto_blanks_added: number;
}

/// Zine booklet page-count presets — must be multiples of 4 (the duplex requirement; single-
/// sided only needs even numbers, but offering one shared preset list per format keeps the
/// picker simple, and every one of these already satisfies both).
export const ZINE_BOOKLET_PRESETS = [8, 12, 16, 24, 32];

/// Typical American comic-book saddle-stitch lengths.
export const COMIC_PRESETS = [20, 24, 28, 32, 36];
