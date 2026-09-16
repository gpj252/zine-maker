import { useEffect, useState } from "react";
import type { ImpositionResult, Margins, PageEntry, Project } from "./types";
import { effectivePaperSize } from "./types";
import NewProjectForm from "./components/NewProjectForm";
import PageList from "./components/PageList";
import SheetPreview from "./components/SheetPreview";
import {
  assetUrl,
  exportProjectPdf,
  getImposition,
  pickImage,
  pickImages,
  pickSaveLocation,
} from "./lib/tauri";

function newId() {
  return crypto.randomUUID();
}

function formatLabel(project: Project): string {
  switch (project.format.kind) {
    case "OnePageZine":
      return "One-page zine (8 panels)";
    case "OnePageZinePosterBack":
      return "One-page zine + poster back";
    case "SixteenPageZine":
      return "16-page mini-zine (1 sheet)";
    case "Booklet": {
      const { page_count, side } = project.format;
      return `${page_count}-page booklet · ${side === "Duplex" ? "duplex" : "single-sided"}`;
    }
  }
}

export default function App() {
  const [project, setProject] = useState<Project | null>(null);
  const [imposition, setImposition] = useState<ImpositionResult | null>(null);
  const [exportState, setExportState] = useState<string | null>(null);

  // Recompute the imposition on every page/margin/format change — cheap pure computation on
  // the Rust side, no reason to debounce or special-case what changed.
  useEffect(() => {
    if (!project) {
      setImposition(null);
      return;
    }
    let cancelled = false;
    getImposition(project).then((result) => {
      if (!cancelled) setImposition(result);
    });
    return () => {
      cancelled = true;
    };
  }, [project]);

  if (!project) {
    return (
      <div className="min-h-screen bg-zinc-950">
        <NewProjectForm onCreate={setProject} />
      </div>
    );
  }

  function updatePages(pages: PageEntry[]) {
    setProject((p) => (p ? { ...p, pages } : p));
  }

  function updateMargins(margins: Margins) {
    setProject((p) => (p ? { ...p, margins } : p));
  }

  async function addImages() {
    const paths = await pickImages();
    if (!paths || paths.length === 0) return;
    updatePages([...project!.pages, ...paths.map((path) => ({ id: newId(), image_path: path }))]);
  }

  function addBlankPage() {
    updatePages([...project!.pages, { id: newId(), image_path: null }]);
  }

  async function pickPoster() {
    const path = await pickImage();
    if (!path) return;
    setProject((p) => (p ? { ...p, poster_image_path: path } : p));
  }

  function clearPoster() {
    setProject((p) => (p ? { ...p, poster_image_path: null } : p));
  }

  async function handleExport() {
    const outputPath = await pickSaveLocation("zine.pdf");
    if (!outputPath) return;
    setExportState("Exporting…");
    try {
      await exportProjectPdf(project!, outputPath);
      setExportState(`✅ Saved to ${outputPath}`);
    } catch (e) {
      setExportState(`❌ Export failed: ${e}`);
    }
  }

  // Mirrors Rust's effective_paper_size(): the one-sheet zine grids print on a landscape
  // sheet regardless of which paper preset (portrait by default) is selected.
  const effectivePaper = effectivePaperSize(project.format, project.paper);
  const paperAspect = effectivePaper.width_mm / effectivePaper.height_mm;

  return (
    <div className="flex h-screen bg-zinc-950 text-zinc-100">
      <div className="flex w-[420px] flex-none flex-col border-r border-zinc-800 p-5">
        <div className="mb-4">
          <button
            onClick={() => setProject(null)}
            className="text-xs text-zinc-500 hover:text-zinc-300"
          >
            ← New project
          </button>
          <h2 className="mt-1 text-lg font-semibold">{formatLabel(project)}</h2>
          <p className="text-xs text-zinc-500">
            {project.paper.width_mm}×{project.paper.height_mm}mm · {project.pages.length} page
            {project.pages.length === 1 ? "" : "s"} added
          </p>
        </div>

        <div className="mb-4 flex gap-2">
          <button
            onClick={addImages}
            className="flex-1 rounded-md bg-indigo-500 py-2 text-sm font-medium text-white hover:bg-indigo-400"
          >
            + Add images
          </button>
          <button
            onClick={addBlankPage}
            className="flex-1 rounded-md bg-zinc-800 py-2 text-sm font-medium text-zinc-200 hover:bg-zinc-700"
          >
            + Blank page
          </button>
        </div>

        <div className="mb-4 flex-1 overflow-y-auto">
          <PageList pages={project.pages} onChange={updatePages} />
        </div>

        {project.format.kind === "OnePageZinePosterBack" && (
          <div className="mb-4 rounded-md border border-zinc-700 bg-zinc-800/50 p-3">
            <div className="mb-2 text-sm font-medium text-zinc-300">Poster (back side)</div>
            {project.poster_image_path ? (
              <div className="flex items-center justify-between gap-2">
                <span className="truncate text-xs text-zinc-400">{project.poster_image_path}</span>
                <button
                  onClick={clearPoster}
                  className="flex-none text-xs text-zinc-500 hover:text-zinc-300"
                >
                  Remove
                </button>
              </div>
            ) : (
              <button
                onClick={pickPoster}
                className="w-full rounded-md bg-zinc-700 py-1.5 text-sm text-zinc-200 hover:bg-zinc-600"
              >
                + Choose poster image
              </button>
            )}
          </div>
        )}

        <MarginsEditor margins={project.margins} onChange={updateMargins} />

        <button
          onClick={handleExport}
          className="mt-4 w-full rounded-md bg-emerald-500 py-2.5 font-medium text-white hover:bg-emerald-400"
        >
          Export print-ready PDF
        </button>
        {exportState && <p className="mt-2 text-xs text-zinc-400">{exportState}</p>}
      </div>

      <div className="flex-1 overflow-y-auto p-6">
        <h3 className="mb-4 text-sm font-medium uppercase tracking-wide text-zinc-500">
          Print preview
        </h3>
        <SheetPreview imposition={imposition} pages={project.pages} paperAspect={paperAspect} />
        {project.format.kind === "OnePageZinePosterBack" && (
          <PosterPreview path={project.poster_image_path} paperAspect={paperAspect} />
        )}
      </div>
    </div>
  );
}

function PosterPreview({ path, paperAspect }: { path: string | null; paperAspect: number }) {
  return (
    <div className="mt-6">
      <div className="mb-2 text-xs font-medium uppercase tracking-wide text-zinc-500">
        Poster (back side)
      </div>
      <div className="mx-auto w-full max-w-[420px]">
        <div
          className="flex items-center justify-center overflow-hidden rounded-md border border-zinc-700 bg-zinc-900"
          style={{ aspectRatio: paperAspect }}
        >
          {path ? (
            <img src={assetUrl(path)} alt="" className="h-full w-full object-contain" />
          ) : (
            <span className="text-xs text-zinc-600">No poster image chosen</span>
          )}
        </div>
      </div>
    </div>
  );
}

function MarginsEditor({
  margins,
  onChange,
}: {
  margins: Margins;
  onChange: (m: Margins) => void;
}) {
  function set(key: keyof Margins, value: number) {
    onChange({ ...margins, [key]: value });
  }

  return (
    <fieldset>
      <legend className="mb-2 text-sm font-medium text-zinc-400">Margins (mm)</legend>
      <div className="grid grid-cols-4 gap-2">
        <MarginInput label="Top" value={margins.top_mm} onChange={(v) => set("top_mm", v)} />
        <MarginInput label="Right" value={margins.right_mm} onChange={(v) => set("right_mm", v)} />
        <MarginInput label="Bottom" value={margins.bottom_mm} onChange={(v) => set("bottom_mm", v)} />
        <MarginInput label="Left" value={margins.left_mm} onChange={(v) => set("left_mm", v)} />
      </div>
    </fieldset>
  );
}

function MarginInput({
  label,
  value,
  onChange,
}: {
  label: string;
  value: number;
  onChange: (v: number) => void;
}) {
  return (
    <label className="flex flex-col gap-1">
      <span className="text-[10px] text-zinc-500">{label}</span>
      <input
        type="number"
        min={0}
        step={0.5}
        value={value}
        onChange={(e) => onChange(Number(e.target.value))}
        className="w-full rounded-md bg-zinc-800 px-2 py-1 text-sm text-zinc-100"
      />
    </label>
  );
}
