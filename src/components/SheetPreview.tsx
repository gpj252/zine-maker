import type { ImpositionResult, PageEntry, Sheet, SheetSide } from "../types";
import { assetUrl } from "../lib/tauri";

interface Props {
  imposition: ImpositionResult | null;
  pages: PageEntry[];
  paperAspect: number; // width_mm / height_mm
}

/** Resolves a reader page number to the page entry, accounting for the imposition's own
 *  trailing auto-blanks (which don't exist in `pages` at all). */
function resolvePage(pages: PageEntry[], pageNum: number | null): PageEntry | null {
  if (pageNum == null) return null;
  return pages[pageNum - 1] ?? { id: `auto-blank-${pageNum}`, image_path: null };
}

export default function SheetPreview({ imposition, pages, paperAspect }: Props) {
  if (!imposition) {
    return <div className="text-sm text-zinc-500">Add pages to see a preview.</div>;
  }

  return (
    <div className="space-y-6">
      {imposition.auto_blanks_added > 0 && (
        <div className="rounded-md bg-amber-500/10 px-3 py-2 text-sm text-amber-300">
          +{imposition.auto_blanks_added} blank page
          {imposition.auto_blanks_added === 1 ? "" : "s"} added automatically at the end to
          complete the layout.
        </div>
      )}
      {imposition.sheets.map((sheet, i) => (
        <SheetCard key={i} sheet={sheet} index={i} pages={pages} paperAspect={paperAspect} />
      ))}
    </div>
  );
}

function SheetCard({
  sheet,
  index,
  pages,
  paperAspect,
}: {
  sheet: Sheet;
  index: number;
  pages: PageEntry[];
  paperAspect: number;
}) {
  return (
    <div>
      <div className="mb-2 text-xs font-medium uppercase tracking-wide text-zinc-500">
        Sheet {index + 1}
      </div>
      <div className={`grid gap-4 ${sheet.back ? "grid-cols-2" : "grid-cols-1"}`}>
        <SideCard label="Front" side={sheet.front} pages={pages} paperAspect={paperAspect} />
        {sheet.back && <SideCard label="Back" side={sheet.back} pages={pages} paperAspect={paperAspect} />}
      </div>
    </div>
  );
}

function SideCard({
  label,
  side,
  pages,
  paperAspect,
}: {
  label: string;
  side: SheetSide;
  pages: PageEntry[];
  paperAspect: number;
}) {
  return (
    <div>
      <div className="mb-1 text-[11px] text-zinc-500">{label}</div>
      <div
        className="grid gap-px overflow-hidden rounded-md border border-zinc-700 bg-zinc-700"
        style={{
          aspectRatio: paperAspect,
          gridTemplateColumns: `repeat(${side.cols}, 1fr)`,
          gridTemplateRows: `repeat(${side.rows}, 1fr)`,
        }}
      >
        {side.slots.map((slot, i) => {
          const entry = resolvePage(pages, slot.page);
          return (
            <div key={i} className="relative flex items-center justify-center overflow-hidden bg-zinc-900">
              {entry?.image_path ? (
                <img
                  src={assetUrl(entry.image_path)}
                  alt=""
                  className="h-full w-full object-contain"
                  style={{ transform: slot.rotated ? "rotate(180deg)" : undefined }}
                />
              ) : (
                <span className="text-[10px] text-zinc-600">
                  {slot.page != null ? `p${slot.page}` : ""}
                </span>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}
