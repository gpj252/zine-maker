import {
  DndContext,
  closestCenter,
  PointerSensor,
  useSensor,
  useSensors,
  type DragEndEvent,
} from "@dnd-kit/core";
import {
  SortableContext,
  arrayMove,
  rectSortingStrategy,
  useSortable,
} from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import type { PageEntry } from "../types";
import { assetUrl } from "../lib/tauri";

interface Props {
  pages: PageEntry[];
  onChange: (pages: PageEntry[]) => void;
}

export default function PageList({ pages, onChange }: Props) {
  const sensors = useSensors(useSensor(PointerSensor, { activationConstraint: { distance: 4 } }));

  function handleDragEnd(event: DragEndEvent) {
    const { active, over } = event;
    if (!over || active.id === over.id) return;
    const oldIndex = pages.findIndex((p) => p.id === active.id);
    const newIndex = pages.findIndex((p) => p.id === over.id);
    onChange(arrayMove(pages, oldIndex, newIndex));
  }

  function removePage(id: string) {
    onChange(pages.filter((p) => p.id !== id));
  }

  if (pages.length === 0) {
    return (
      <div className="flex h-40 items-center justify-center rounded-lg border border-dashed border-zinc-700 text-sm text-zinc-500">
        No pages yet — add some images or insert a blank page to start.
      </div>
    );
  }

  return (
    <DndContext sensors={sensors} collisionDetection={closestCenter} onDragEnd={handleDragEnd}>
      <SortableContext items={pages.map((p) => p.id)} strategy={rectSortingStrategy}>
        <div className="grid grid-cols-[repeat(auto-fill,minmax(96px,1fr))] gap-3">
          {pages.map((page, i) => (
            <SortablePage key={page.id} page={page} index={i} onRemove={() => removePage(page.id)} />
          ))}
        </div>
      </SortableContext>
    </DndContext>
  );
}

function SortablePage({
  page,
  index,
  onRemove,
}: {
  page: PageEntry;
  index: number;
  onRemove: () => void;
}) {
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({
    id: page.id,
  });

  return (
    <div
      ref={setNodeRef}
      style={{ transform: CSS.Transform.toString(transform), transition, opacity: isDragging ? 0.5 : 1 }}
      {...attributes}
      {...listeners}
      className="group relative aspect-[3/4] cursor-grab rounded-md border border-zinc-700 bg-zinc-800 active:cursor-grabbing"
    >
      {page.image_path ? (
        <img
          src={assetUrl(page.image_path)}
          alt={`page ${index + 1}`}
          className="h-full w-full rounded-md object-cover"
          draggable={false}
        />
      ) : (
        <div className="flex h-full w-full items-center justify-center rounded-md bg-zinc-800 text-xs italic text-zinc-500">
          blank
        </div>
      )}
      <span className="absolute left-1 top-1 rounded bg-black/60 px-1.5 py-0.5 text-[10px] font-medium text-white">
        {index + 1}
      </span>
      <button
        onClick={(e) => {
          e.stopPropagation();
          onRemove();
        }}
        className="absolute right-1 top-1 hidden h-5 w-5 items-center justify-center rounded-full bg-black/60 text-xs text-white hover:bg-red-500 group-hover:flex"
        title="Remove"
      >
        ×
      </button>
    </div>
  );
}
