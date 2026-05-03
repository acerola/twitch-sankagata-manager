import { useEffect } from "react";
import { create } from "zustand";
import { listen, type Event, type UnlistenFn } from "@tauri-apps/api/event";
import type { Config, Snapshot } from "./types";
import { ipc } from "./ipc";

type Store = {
  snap: Snapshot | null;
  config: Config | null;
  /** Counter incremented on each state-changed event — used by overlay's
   *  dev-mode diagnostic badge to prove IPC event delivery end-to-end. */
  _syncCount: number;
  hydrate: () => Promise<void>;
  setConfig: (cfg: Config) => Promise<void>;
};

export const useStore = create<Store>((set) => ({
  snap: null,
  config: null,
  _syncCount: 0,
  hydrate: async () => {
    const [snap, config] = await Promise.all([ipc.getSnapshot(), ipc.getConfig()]);
    set({ snap, config });
    (await import("./i18n")).default.changeLanguage(config.language);
  },
  setConfig: async (cfg) => {
    const snap = await ipc.setConfig(cfg);
    set({ snap, config: cfg });
    (await import("./i18n")).default.changeLanguage(cfg.language);
  },
}));

/**
 * Subscribe to Tauri `state-changed` events for the lifetime of the caller
 * component. Every Tauri window (main, overlay) must call this once —
 * a module-level `listen()` race can drop the first event in a fresh window.
 */
export function useStateSync() {
  useEffect(() => {
    let cancelled = false;
    let unlisten: UnlistenFn | null = null;
    if (import.meta.env.DEV) console.debug("[store] registering state-changed listener");
    listen<Snapshot>(
      "state-changed",
      (e: Event<Snapshot>) => {
        if (import.meta.env.DEV) {
          const s = e.payload;
          console.debug(
            `[store] state-changed received: playing=${s.playing.length}`
              + ` waiting=${s.waiting.length} trash=${s.trash.length}`
          );
        }
        const prev = useStore.getState();
        useStore.setState({
          snap: e.payload,
          _syncCount: (prev._syncCount ?? 0) + 1,
        });
      },
      { target: { kind: "Any" } },
    )
      .then((fn) => {
        if (cancelled) fn();
        else unlisten = fn;
        if (import.meta.env.DEV) console.debug("[store] listener registered");
      })
      .catch((err) => console.error("[store] listen failed", err));
    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);
}
