import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { ipc, type DeviceStart } from "../ipc";
import { openUrl } from "@tauri-apps/plugin-opener";

type Phase = "idle" | "starting" | "awaiting_user" | "failed";

export function AuthPrompt({ onAuthed }: { onAuthed: () => void }) {
  const { t } = useTranslation();
  const [phase, setPhase] = useState<Phase>("idle");
  const [pending, setPending] = useState<DeviceStart | null>(null);
  const [error, setError] = useState<string>("");
  const timerRef = useRef<ReturnType<typeof setInterval> | null>(null);

  useEffect(() => () => {
    if (timerRef.current) clearInterval(timerRef.current);
  }, []);

  const start = async () => {
    setError("");
    setPhase("starting");
    try {
      const info = await ipc.startAuth();
      setPending(info);
      setPhase("awaiting_user");
      openUrl(info.verificationUri).catch(() => {});
      timerRef.current = setInterval(async () => {
        try {
          const s = await ipc.getAuthStatus();
          if (s.authenticated) {
            if (timerRef.current) clearInterval(timerRef.current);
            onAuthed();
          }
        } catch (e) {
          console.error("poll status", e);
        }
      }, 2500);
    } catch (e) {
      setError(String(e));
      setPhase("failed");
    }
  };

  return (
    <div className="auth-prompt">
      <h2>{t("header.login")}</h2>

      {phase === "idle" && (
        <button className="primary" onClick={start}>{t("header.login")}</button>
      )}

      {phase === "starting" && (
        <div className="auth-status">
          <span className="spinner" aria-hidden="true" />
          <span>{t("auth.starting")}</span>
        </div>
      )}

      {phase === "awaiting_user" && pending && (
        <div>
          <p>{t("auth.openBrowser")}</p>
          <p>
            <a href={pending.verificationUri} target="_blank" rel="noreferrer">
              {pending.verificationUri}
            </a>
          </p>
          <p>{t("auth.enterCode")}</p>
          <div className="device-code">{pending.userCode}</div>
          <div className="auth-status">
            <span className="spinner" aria-hidden="true" />
            <span>{t("auth.waiting")}</span>
          </div>
        </div>
      )}

      {phase === "failed" && (
        <div>
          <p className="error-text">{t("auth.failed", { error })}</p>
          <button className="primary" onClick={start}>{t("auth.retry")}</button>
        </div>
      )}
    </div>
  );
}
