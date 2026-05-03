import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import type { UpdateInfo, UpdateProgress } from "../update";
import { downloadUpdate, installAndRelaunch, openReleasePage } from "../update";

type Phase = "ready" | "downloading" | "downloaded" | "installing" | "error";

function formatBytes(bytes: number) {
  if (!Number.isFinite(bytes) || bytes <= 0) return "0 B";
  const units = ["B", "KB", "MB", "GB"];
  let value = bytes;
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit += 1;
  }
  return `${value >= 10 || unit === 0 ? Math.round(value) : value.toFixed(1)} ${units[unit]}`;
}

export function UpdateModal({
  info,
  onClose,
}: {
  info: UpdateInfo;
  onClose: () => void;
}) {
  const { t } = useTranslation();
  const [phase, setPhase] = useState<Phase>("ready");
  const [progress, setProgress] = useState<UpdateProgress>({
    downloaded: 0,
    done: false,
  });
  const [error, setError] = useState<string | null>(null);
  const isBusy = phase === "downloading" || phase === "installing";
  const isPortable = info.installMode.portable;
  const percentLabel = progress.percent == null ? null : `${Math.round(progress.percent)}%`;
  const sizeLabel = useMemo(() => {
    if (!progress.total) return null;
    return `${formatBytes(progress.downloaded)} / ${formatBytes(progress.total)}`;
  }, [progress.downloaded, progress.total]);

  const startDownload = async () => {
    setError(null);
    setProgress({ downloaded: 0, done: false });
    setPhase("downloading");
    try {
      await downloadUpdate(info.update, setProgress);
      setPhase("downloaded");
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      setPhase("error");
    }
  };

  const install = async () => {
    setError(null);
    setPhase("installing");
    try {
      await installAndRelaunch(info.update);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      setPhase("error");
    }
  };

  const openDownloads = async () => {
    setError(null);
    try {
      await openReleasePage(info.releaseUrl);
      onClose();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      setPhase("error");
    }
  };

  return (
    <div className="modal-backdrop">
      <div className="modal update-modal" role="dialog" aria-modal="true">
        <h2>{t("updates.title")}</h2>
        <p className="update-summary">
          {t("updates.summary", {
            version: info.version,
            currentVersion: info.currentVersion,
          })}
        </p>
        <p className="update-body">
          {isPortable ? t("updates.portableBody") : t("updates.installedBody")}
        </p>

        {info.body && (
          <details className="update-notes">
            <summary>{t("updates.releaseNotes")}</summary>
            <p>{info.body}</p>
          </details>
        )}

        {(phase === "downloading" || phase === "downloaded" || phase === "installing") && (
          <div className="update-progress" aria-live="polite">
            <div className="update-progress-row">
              <span>
                {phase === "downloaded"
                  ? t("updates.downloaded")
                  : phase === "installing"
                    ? t("updates.installing")
                    : t("updates.downloading")}
              </span>
              <span>{percentLabel ?? sizeLabel ?? t("updates.unknownSize")}</span>
            </div>
            <div className="update-progress-track">
              <div
                className={
                  progress.total ? "update-progress-bar" : "update-progress-bar is-indeterminate"
                }
                style={progress.percent == null ? undefined : { width: `${progress.percent}%` }}
              />
            </div>
            {sizeLabel && <div className="update-progress-size">{sizeLabel}</div>}
          </div>
        )}

        {error && (
          <p className="error-text" role="alert">
            {t("updates.errorPrefix", { error })}
          </p>
        )}

        <div className="modal-actions">
          <span style={{ flex: 1 }} />
          <button onClick={onClose} disabled={isBusy}>
            {t("updates.later")}
          </button>
          {isPortable ? (
            <button className="primary" onClick={openDownloads} disabled={isBusy}>
              {t("updates.openRelease")}
            </button>
          ) : phase === "downloaded" ? (
            <button className="primary" onClick={install} disabled={isBusy}>
              {t("updates.installRestart")}
            </button>
          ) : (
            <button className="primary" onClick={startDownload} disabled={isBusy}>
              {phase === "error" ? t("updates.retry") : t("updates.download")}
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
