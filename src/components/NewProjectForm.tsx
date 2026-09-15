import { useState } from "react";
import type { BookletSide, Format, PaperSize, Project } from "../types";
import { PAPER_PRESETS, ZINE_BOOKLET_PRESETS, COMIC_PRESETS } from "../types";

type ProjectKind = "zine" | "comic";
type ZineStyle = "onepage" | "booklet";

interface Props {
  onCreate: (project: Project) => void;
}

export default function NewProjectForm({ onCreate }: Props) {
  const [kind, setKind] = useState<ProjectKind>("zine");
  const [zineStyle, setZineStyle] = useState<ZineStyle>("booklet");
  const [pageCount, setPageCount] = useState(ZINE_BOOKLET_PRESETS[0]);
  const [side, setSide] = useState<BookletSide>("Duplex");
  const [paperName, setPaperName] = useState<keyof typeof PAPER_PRESETS>("A4");

  const presets = kind === "zine" ? ZINE_BOOKLET_PRESETS : COMIC_PRESETS;
  const isOnePage = kind === "zine" && zineStyle === "onepage";

  function create() {
    const format: Format = isOnePage
      ? { kind: "OnePageZine" }
      : { kind: "Booklet", page_count: pageCount, side };
    const paper: PaperSize = PAPER_PRESETS[paperName];
    onCreate({
      format,
      paper,
      margins: { top_mm: 5, right_mm: 5, bottom_mm: 5, left_mm: 5 },
      pages: [],
    });
  }

  return (
    <div className="mx-auto max-w-xl p-8">
      <h1 className="mb-6 text-2xl font-semibold text-zinc-100">New project</h1>

      <fieldset className="mb-6">
        <legend className="mb-2 text-sm font-medium text-zinc-400">What are you making?</legend>
        <div className="grid grid-cols-2 gap-3">
          <PickerCard label="Zine" active={kind === "zine"} onClick={() => setKind("zine")} />
          <PickerCard label="Comic" active={kind === "comic"} onClick={() => setKind("comic")} />
        </div>
      </fieldset>

      {kind === "zine" && (
        <fieldset className="mb-6">
          <legend className="mb-2 text-sm font-medium text-zinc-400">Zine style</legend>
          <div className="grid grid-cols-2 gap-3">
            <PickerCard
              label="One-page (8 panels)"
              sub="Single sheet, no stapling — fold and cut"
              active={zineStyle === "onepage"}
              onClick={() => setZineStyle("onepage")}
            />
            <PickerCard
              label="Booklet"
              sub="Choose a page count, staple the spine"
              active={zineStyle === "booklet"}
              onClick={() => setZineStyle("booklet")}
            />
          </div>
        </fieldset>
      )}

      {!isOnePage && (
        <>
          <fieldset className="mb-6">
            <legend className="mb-2 text-sm font-medium text-zinc-400">Page count</legend>
            <div className="flex flex-wrap gap-2">
              {presets.map((n) => (
                <button
                  key={n}
                  onClick={() => setPageCount(n)}
                  className={`rounded-md px-3 py-1.5 text-sm ${
                    pageCount === n
                      ? "bg-indigo-500 text-white"
                      : "bg-zinc-800 text-zinc-300 hover:bg-zinc-700"
                  }`}
                >
                  {n}
                </button>
              ))}
              <input
                type="number"
                min={kind === "zine" ? 4 : 4}
                step={side === "Duplex" ? 4 : 2}
                value={pageCount}
                onChange={(e) => setPageCount(Number(e.target.value))}
                className="w-20 rounded-md bg-zinc-800 px-2 py-1.5 text-sm text-zinc-100"
              />
            </div>
          </fieldset>

          <fieldset className="mb-6">
            <legend className="mb-2 text-sm font-medium text-zinc-400">Printing</legend>
            <div className="grid grid-cols-2 gap-3">
              <PickerCard
                label="Duplex"
                sub="Real saddle-stitch booklet"
                active={side === "Duplex"}
                onClick={() => setSide("Duplex")}
              />
              <PickerCard
                label="Single-sided"
                sub="No duplex printer needed"
                active={side === "Single"}
                onClick={() => setSide("Single")}
              />
            </div>
          </fieldset>
        </>
      )}

      <fieldset className="mb-8">
        <legend className="mb-2 text-sm font-medium text-zinc-400">Paper size</legend>
        <div className="grid grid-cols-2 gap-3">
          {Object.keys(PAPER_PRESETS).map((name) => (
            <PickerCard
              key={name}
              label={name}
              active={paperName === name}
              onClick={() => setPaperName(name as keyof typeof PAPER_PRESETS)}
            />
          ))}
        </div>
      </fieldset>

      <button
        onClick={create}
        className="w-full rounded-md bg-indigo-500 py-2.5 font-medium text-white hover:bg-indigo-400"
      >
        Start arranging pages
      </button>
    </div>
  );
}

function PickerCard({
  label,
  sub,
  active,
  onClick,
}: {
  label: string;
  sub?: string;
  active: boolean;
  onClick: () => void;
}) {
  return (
    <button
      onClick={onClick}
      className={`rounded-lg border px-4 py-3 text-left transition ${
        active
          ? "border-indigo-400 bg-indigo-500/10"
          : "border-zinc-700 bg-zinc-800/50 hover:border-zinc-600"
      }`}
    >
      <div className="font-medium text-zinc-100">{label}</div>
      {sub && <div className="mt-0.5 text-xs text-zinc-400">{sub}</div>}
    </button>
  );
}
