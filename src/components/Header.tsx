import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useStore } from "../store";
import { ipc } from "../ipc";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { ConfirmModal } from "./ConfirmModal";
import { DebugPanel } from "./DebugPanel";
import { ThemeModal } from "./ThemeModal";
import {
  PaletteIcon,
  SettingsIcon,
  DebugIcon,
  PlayIcon,
  PauseIcon,
  MinimizeIcon,
  MaximizeIcon,
  RestoreIcon,
  CloseIcon,
  LogoutIcon,
  EyeIcon,
  EyeOffIcon,
} from "./icons";

const appIconUrl = new URL("../../src-tauri/icons/32x32.png", import.meta.url).href;
const APP_VERSION = import.meta.env.VITE_APP_VERSION;

export function Header({
  loginName = null,
  onOpenSettings,
  onLoggedOut,
}: {
  loginName?: string | null;
  onOpenSettings: () => void;
  onLoggedOut: () => void;
}) {
  const { t } = useTranslation();
  const snap = useStore((s) => s.snap);
  const config = useStore((s) => s.config);
  const enabled = snap?.enabled ?? false;
  const [copied, setCopied] = useState(false);
  const [debugOpen, setDebugOpen] = useState(false);
  const [themeOpen, setThemeOpen] = useState(false);
  const [isMaximized, setIsMaximized] = useState(false);
  const [overlayVisible, setOverlayVisible] = useState(true);

  useEffect(() => {
    const win = getCurrentWindow();
    let cancelled = false;
    let unlistenResized: (() => void) | null = null;
    const updateMaximized = () => {
      win.isMaximized().then((v) => {
        if (!cancelled) setIsMaximized(v);
      }).catch(() => {});
    };
    const updateOverlayVisible = async () => {
      try {
        const ov = await WebviewWindow.getByLabel("overlay");
        if (!ov) return;
        const v = await ov.isVisible();
        if (!cancelled) setOverlayVisible(v);
      } catch {}
    };
    updateMaximized();
    updateOverlayVisible();
    win.onResized(() => updateMaximized())
      .then((fn) => {
        if (cancelled) fn();
        else unlistenResized = fn;
      })
      .catch(() => {});
    return () => {
      cancelled = true;
      unlistenResized?.();
    };
  }, []);

  const toggle = async () => {
    if (enabled) {
      setShowDisableConfirm(true);
      return;
    }
    await ipc.setEnabled(!enabled);
  };

  const [showDisableConfirm, setShowDisableConfirm] = useState(false);

  const handleDisableConfirm = async () => {
    setShowDisableConfirm(false);
    await ipc.setEnabled(false);
  };
  const copyUrl = async () => {
    try {
      await writeText(`http://localhost:${config?.port ?? 24816}/overlay`);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (e) {
      console.error("copy failed", e);
    }
  };
  const toggleOverlay = async () => {
    try {
      const ov = await WebviewWindow.getByLabel("overlay");
      if (!ov) return;
      const visible = await ov.isVisible();
      if (visible) {
        await ov.hide();
        setOverlayVisible(false);
      } else {
        await ov.show();
        setOverlayVisible(true);
      }
    } catch (e) {
      console.error("toggle overlay failed", e);
    }
  };
  const logout = async () => {
    await ipc.logout();
    onLoggedOut();
  };

  const win = getCurrentWindow();

  const handleMinimize = () => {
    win.minimize().catch((err) => console.error("minimize failed", err));
  };

  const handleMaximizeToggle = () => {
    if (isMaximized) {
      win.unmaximize()
        .then(() => setIsMaximized(false))
        .catch((err) => console.error("unmaximize failed", err));
    } else {
      win.maximize()
        .then(() => setIsMaximized(true))
        .catch((err) => console.error("maximize failed", err));
    }
  };

  const handleClose = () => {
    win.close().catch((err) => console.error("close failed", err));
  };

  const startTitleDrag = (e: React.PointerEvent<HTMLDivElement>) => {
    if (e.button !== 0) return;
    if (debugOpen || themeOpen || showDisableConfirm) return;
    const target = e.target as HTMLElement;
    if (
      target.closest("button") ||
      target.closest("a") ||
      target.closest("input") ||
      target.closest("select") ||
      target.closest("textarea") ||
      target.closest(".modal-backdrop") ||
      target.closest(".modal") ||
      target.closest(".confirm-backdrop") ||
      target.closest(".confirm-modal")
    ) {
      return;
    }
    getCurrentWindow().startDragging().catch((err) => console.error("title drag failed", err));
  };

  return (
    <header className="app-header" onPointerDown={startTitleDrag}>
      <div className="header-left">
        <div className="brand">
          <img className="brand-icon" src={appIconUrl} alt="" draggable={false} />
          <span>Twitch Sankagata Manager</span>
          <span className="version">v{APP_VERSION}</span>
        </div>
        {loginName && (
          <>
            <span className="user-pill" title={t("header.loggedInAs", { name: loginName })}>
              <span className="dot" />
              {loginName}
            </span>
            <button
              className="icon-only logout-btn"
              onClick={logout}
              title={t("header.logout")}
              aria-label={t("header.logout")}
            >
              <LogoutIcon />
            </button>
          </>
        )}
      </div>

      <div className="header-right">
        <button
          className={`toggle ${enabled ? "on" : "off"}`}
          onClick={toggle}
          title={enabled ? t("header.enabled") : t("header.disabled")}
          aria-label={enabled ? t("header.enabled") : t("header.disabled")}
        >
          {enabled ? <PauseIcon size={14} /> : <PlayIcon size={14} />}
          <span>{enabled ? t("header.enabled") : t("header.disabled")}</span>
        </button>
        <button
          className="icon-only"
          onClick={toggleOverlay}
          title={overlayVisible ? t("header.hideOverlay") : t("header.showOverlay")}
          aria-label={overlayVisible ? t("header.hideOverlay") : t("header.showOverlay")}
        >
          {overlayVisible ? <EyeIcon /> : <EyeOffIcon />}
        </button>
        <div className="copy-wrap">
          <button onClick={copyUrl}>{t("header.copyUrl")}</button>
          {copied && <span className="toast">{t("header.copied")}</span>}
          <span
            className="help-chip"
            style={{ marginLeft: "6px" }}
            tabIndex={0}
            aria-label={t("header.obsSizeTip")}
          >
            ?
            <span className="help-tooltip bottom">{t("header.obsSizeTip")}</span>
          </span>
        </div>
        <button
          className="icon-only"
          onClick={() => setThemeOpen(true)}
          title={t("theme.title")}
          aria-label={t("theme.title")}
        >
          <PaletteIcon />
        </button>
        <button
          className="icon-only"
          onClick={onOpenSettings}
          title={t("header.settings")}
          aria-label={t("header.settings")}
        >
          <SettingsIcon />
        </button>
        {import.meta.env.DEV && (
          <button
            className="icon-only"
            title="Debug tools (dev only)"
            aria-label="Debug tools"
            onClick={() => setDebugOpen(true)}
            style={{ borderStyle: "dashed", opacity: 0.75 }}
          >
            <DebugIcon />
          </button>
        )}

        <div className="window-controls">
          <button
            className="window-btn minimize"
            onClick={handleMinimize}
            title={t("header.minimize")}
            aria-label={t("header.minimize")}
          >
            <MinimizeIcon />
          </button>
          <button
            className="window-btn maximize"
            onClick={handleMaximizeToggle}
            title={isMaximized ? t("header.restore") : t("header.maximize")}
            aria-label={isMaximized ? t("header.restore") : t("header.maximize")}
          >
            {isMaximized ? <RestoreIcon /> : <MaximizeIcon />}
          </button>
          <button
            className="window-btn close"
            onClick={handleClose}
            title={t("header.close")}
            aria-label={t("header.close")}
          >
            <CloseIcon />
          </button>
        </div>
      </div>

      {debugOpen && <DebugPanel onClose={() => setDebugOpen(false)} />}
      {themeOpen && <ThemeModal onClose={() => setThemeOpen(false)} />}
      {showDisableConfirm && (
        <ConfirmModal
          title={t("header.disableTitle")}
          message={t("header.disableConfirm")}
          confirmLabel={t("header.disableConfirmOk")}
          cancelLabel={t("settings.cancel")}
          onConfirm={handleDisableConfirm}
          onCancel={() => setShowDisableConfirm(false)}
        />
      )}
    </header>
  );
}
