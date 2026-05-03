import { useTranslation } from "react-i18next";
import type { User } from "../types";
import { ipc } from "../ipc";
import { ClearIcon, TrashIcon } from "./icons";

export function Row({
  user, zone, index, onDragStart,
}: {
  user: User;
  zone: "playing" | "waiting";
  index: number;
  onDragStart: (userId: string) => (e: React.DragEvent) => void;
}) {
  const { t } = useTranslation();
  const isFirst = user.firstTimeToday;
  return (
    <div
      className={`row ${zone}`}
      draggable
      onDragStart={onDragStart(user.id)}
      data-index={index}
    >
      <span className="name" title={user.displayName}>{user.displayName}</span>
      {isFirst && <span className="badge" title={t("row.badgeTitle")}>初</span>}
      {zone === "playing" && (
        <button
          type="button"
          className="clear-btn"
          title={t("row.clearFromPlaying")}
          aria-label={`${t("row.clearFromPlaying")} ${user.displayName}`}
          onClick={() => ipc.clearPlayingUser(user.id)}
        >
          <ClearIcon />
        </button>
      )}
      <button
        type="button"
        className="trash-btn"
        title={t("row.sendToTrash")}
        aria-label={`${t("row.sendToTrash")} ${user.displayName}`}
        onClick={() => ipc.trashUser(user.id)}
      >
        <TrashIcon />
      </button>
    </div>
  );
}
