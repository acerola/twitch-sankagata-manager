import { useEffect, useState } from "react";
import { Header } from "./components/Header";
import { PlayingPane } from "./components/PlayingPane";
import { WaitingPane } from "./components/WaitingPane";
import { SettingsModal } from "./components/SettingsModal";
import { AuthPrompt } from "./components/AuthPrompt";
import { useStore, useStateSync } from "./store";
import { ipc, type AuthStatus } from "./ipc";
import { applyTheme } from "./theme-utils";
import { UpdateModal } from "./components/UpdateModal";
import { checkForUpdate, type UpdateInfo } from "./update";
import "./styles.css";
import "./i18n";

export default function App() {
  const hydrate = useStore((s) => s.hydrate);
  const config = useStore((s) => s.config);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [authStatus, setAuthStatus] = useState<AuthStatus | null>(null);
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);
  useStateSync();

  useEffect(() => {
    hydrate();
    ipc.getAuthStatus()
      .then(setAuthStatus)
      .catch(() => setAuthStatus({ authenticated: false, loginName: null }));
  }, [hydrate]);

  useEffect(() => {
    if (config?.theme) {
      applyTheme(config.theme, config.customColors);
    }
  }, [config?.theme, config?.customColors]);

  useEffect(() => {
    let cancelled = false;
    let timer = 0;
    if (import.meta.env.PROD) {
      timer = window.setTimeout(() => {
        checkForUpdate().then((info) => {
          if (!info) return;
          if (cancelled) {
            info.update.close().catch(() => {});
          } else {
            setUpdateInfo(info);
          }
        });
      }, 3000);
    }
    return () => {
      cancelled = true;
      window.clearTimeout(timer);
    };
  }, []);

  if (authStatus?.authenticated === false) {
    return (
      <div className="app">
        <main className="panes" style={{ gridTemplateColumns: "1fr", alignItems: "center", justifyContent: "center" }}>
          <AuthPrompt
            onAuthed={() => {
              ipc.getAuthStatus()
                .then(setAuthStatus)
                .catch(() => setAuthStatus({ authenticated: true, loginName: null }));
            }}
          />
        </main>
      </div>
    );
  }

  return (
    <div className="app">
      <Header
        loginName={authStatus?.loginName ?? null}
        onOpenSettings={() => setSettingsOpen(true)}
        onLoggedOut={() => setAuthStatus({ authenticated: false, loginName: null })}
      />
      <main className="panes">
        <PlayingPane />
        <WaitingPane />
      </main>
      {settingsOpen && <SettingsModal onClose={() => setSettingsOpen(false)} />}
      {updateInfo && (
        <UpdateModal
          info={updateInfo}
          onClose={() => {
            updateInfo.update.close().catch(() => {});
            setUpdateInfo(null);
          }}
        />
      )}
    </div>
  );
}
