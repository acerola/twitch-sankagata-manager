import { useRef } from "react";
import type { Zone } from "../types";
import { ipc } from "../ipc";

export function useDragDrop() {
  const dragged = useRef<{ id: string } | null>(null);

  const onDragStart = (userId: string) => (e: React.DragEvent) => {
    dragged.current = { id: userId };
    e.dataTransfer.effectAllowed = "move";
    e.dataTransfer.setData("text/plain", userId);
  };

  const onDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = "move";
  };

  const onDrop = (zone: Zone, index: number) => (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    const id = dragged.current?.id ?? e.dataTransfer.getData("text/plain");
    if (!id) return;
    ipc.moveUser(id, zone, index).catch((err) => console.error("move user failed", err));
    dragged.current = null;
  };

  return { onDragStart, onDragOver, onDrop };
}
