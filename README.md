# Zine/Comic Maker

A desktop app (Tauri + React/TypeScript + Rust) that takes a folder of page images and turns
them into a print-ready, correctly-imposed PDF for a zine or comic — so the physical folding
and stapling actually comes out in the right page order without doing the math by hand.

## What it does

1. **Pick a format**
   - **Zine → One-page**: the classic single-sheet, 8-panel mini zine (fold into eighths,
     slit the center fold, no stapling).
   - **Zine → Booklet** or **Comic**: a saddle-stitch booklet — choose a page count preset (or
     a custom one) and duplex (real booklet, needs a duplex printer) or single-sided (needs
     none, but isn't a true nested signature — see the code comment in
     `imposition.rs::BookletSide::Single`).
2. **Upload pages** in reading order; drag to reorder, insert blank pages anywhere (the
   imposition recomputes automatically — there's no separate "reorganize" step, every edit
   just re-runs the same pure function).
3. **Set margins** per side, in mm.
4. **Preview** the actual imposed sheets — what will print, not just the reader-order pages.
5. **Export** a print-ready PDF, one page per physical sheet side.

## Project layout

- `src-tauri/src/imposition.rs` — the actual page-to-sheet-position math. Pure, unit-tested,
  no I/O. The one-page-zine panel grid is verified against
  [pdfimpose](https://framagit.org/spalax/pdfimpose)'s reference output, not derived from
  scratch — see the doc comment for the source.
- `src-tauri/src/project.rs` — the project data model and `compute_imposition`, which pads
  the page list to whatever multiple the chosen format needs and calls into `imposition.rs`.
- `src-tauri/src/pdf_export.rs` — renders imposed sheets to a PDF via `printpdf` + `image`.
  **Not yet visually verified** — see the warning at the top of the file.
- `src/` — the React UI: `NewProjectForm`, `PageList` (drag-reorder via `@dnd-kit`),
  `SheetPreview`.

## Status

Early scaffold — the imposition math has unit tests (`cargo test`) and is the part most
worth trusting right now. The PDF export path compiles against printpdf's documented API but
hasn't been run against a real printer/fold yet. Do that before trusting a real print job.

## Developing

```bash
npm install
npm run tauri dev
```

Needs a Rust toolchain and Tauri's Linux prerequisites (webkit2gtk, etc. — see
https://tauri.app/start/prerequisites/) if building on Linux.
