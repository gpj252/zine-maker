// Mirrors src-tauri/src/{imposition,project}.rs — keep these in sync by hand, there's no
// codegen wired up yet.

export type BookletSide = "Duplex" | "Single";

export type Format =
  | { kind: "OnePageZine" }
  | { kind: "Booklet"; page_count: number; side: BookletSide };

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
