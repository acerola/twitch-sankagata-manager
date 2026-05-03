import { useState } from "react";
import { useTranslation } from "react-i18next";
import { useStore } from "../store";
import { Row } from "./Row";
import { useDragDrop } from "../hooks/useDragDrop";
import { ipc } from "../ipc";
import { WaitingIcon, TrashIcon } from "./icons";
import { ConfirmModal } from "./ConfirmModal";

export function WaitingPane() {
  const { t } = useTranslation();
  const snap = useStore((s) => s.snap);
  const { onDragStart, onDragOver, onDrop } = useDragDrop();
  const [showConfirm, setShowConfirm] = useState(false);
  if (!snap) return null;

  const handleClearTrash = () => {
    if (snap.trash.length === 0) return;
    setShowConfirm(true);
  };

  const handleConfirmClear = () => {
    ipc.clearTrash().catch((err) => console.error("clear trash failed", err));
    setShowConfirm(false);
  };

  return (
    <div className="pane waiting-pane">
      <section
        data-zone="waiting"
        onDragOver={onDragOver}
        onDrop={onDrop("waiting", snap.waiting.length)}
      >
        <h3><WaitingIcon size={14} /> {t("panes.waiting")} <span className="count">{snap.waitingTotal} total</span></h3>
        {snap.waiting.length === 0
          ? <div className="empty">{t("panes.noWaiting")}</div>
          : snap.waiting.map((u, i) => (
              <div key={u.id} onDrop={onDrop("waiting", i)} onDragOver={onDragOver}>
                <Row user={u} zone="waiting" index={i} onDragStart={onDragStart} />
              </div>
            ))
        }
      </section>
      <section className="trash-section" aria-label={t("panes.trash")}>
        <div className="trash-header">
          <h3>
            {t("panes.trash")}
            <span className="count">{snap.trash.length}</span>
          </h3>
          {snap.trash.length > 0 && (
            <button
              className="trash-btn icon-only clear-trash-btn"
              onClick={handleClearTrash}
              title={t("row.clearTrash")}
              aria-label={t("row.clearTrash")}
            >
              <TrashIcon size={14} />
            </button>
          )}
        </div>
        {snap.trash.length === 0
          ? <div className="empty trash-empty"></div>
          : snap.trash.map((u) => (
              <div key={u.id} className="row trash-row">
                <span className="name" title={u.displayName}>{u.displayName}</span>
                <button
                  className="restore-btn"
                  aria-label={`${t("row.restore")} ${u.displayName}`}
                  onClick={() => ipc.restoreUser(u.id)}
                >
                  {t("row.restore")}
                </button>
              </div>
            ))
        }
      </section>
      {showConfirm && (
        <ConfirmModal
          title={t("row.clearTrashTitle")}
          message={t("row.clearTrashConfirm")}
          confirmLabel={t("row.clearTrash")}
          cancelLabel={t("settings.cancel")}
          onConfirm={handleConfirmClear}
          onCancel={() => setShowConfirm(false)}
        />
      )}
    </div>
  );
}
