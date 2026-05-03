import { useTranslation } from "react-i18next";
import { useStore } from "../store";
import { Row } from "./Row";
import { useDragDrop } from "../hooks/useDragDrop";
import { ipc } from "../ipc";
import { ClearIcon, PlayIcon } from "./icons";

export function PlayingPane() {
  const { t } = useTranslation();
  const snap = useStore((s) => s.snap);
  const config = useStore((s) => s.config);
  const { onDragStart, onDragOver, onDrop } = useDragDrop();
  if (!snap || !config) return null;
  const slots = Array.from({ length: config.maxPlaying }, (_, i) => snap.playing[i]);
  return (
    <div className="pane" data-zone="playing" onDragOver={onDragOver} onDrop={onDrop("playing", snap.playing.length)}>
      <h3 className="pane-heading">
        <span className="pane-title">
          <PlayIcon size={14} /> {t("panes.playing")} <span className="count">{snap.playing.length}/{config.maxPlaying}</span>
        </span>
        <button
          type="button"
          className="clear-match-btn"
          title={t("panes.clearMatchTitle")}
          disabled={snap.playing.length === 0}
          onClick={() => ipc.clearPlaying()}
        >
          <ClearIcon size={13} /> {t("panes.clearMatch")}
        </button>
      </h3>
      {slots.map((u, i) => u
        ? <div key={u.id} onDrop={onDrop("playing", i)} onDragOver={onDragOver}>
            <Row user={u} zone="playing" index={i} onDragStart={onDragStart} />
          </div>
        : <div key={`empty-${i}`} className="row empty" onDrop={onDrop("playing", i)} onDragOver={onDragOver}>
            {t("panes.emptySlot")}
          </div>
      )}
    </div>
  );
}
