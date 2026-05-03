import { render, act, waitFor } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import type { Snapshot } from "../types";

type Handler = (e: { payload: Snapshot }) => void;
const handlers: Handler[] = [];

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(async (_evt: string, cb: Handler) => {
    handlers.push(cb);
    return () => {
      const i = handlers.indexOf(cb);
      if (i !== -1) handlers.splice(i, 1);
    };
  }),
}));

vi.mock("../ipc", () => ({
  ipc: {
    getSnapshot: vi.fn(async (): Promise<Snapshot> => ({
      type: "state",
      playing: [],
      waiting: [],
      waitingTotal: 0,
      trash: [],
      enabled: true,
      language: "ja",
      maxWaiting: 3,
      theme: "midnight",
    })),
    getConfig: vi.fn(async () => ({
      firstTimeKeyword: "参加券", maxPlaying: 4, maxWaiting: 3,
      prioritizeFirstTimers: true, enabled: true, language: "ja" as const, port: 24816,
      mockMode: false, theme: "twitch" as const,
    })),
  },
}));

const freshSnap = (playingIds: string[], waitingIds: string[]): Snapshot => ({
  type: "state",
  playing: playingIds.map((id) => ({
    id, name: id, displayName: id, joinCount: 1, lastJoinAt: Date.now(),
    enqueuedAt: 0, manualOrder: null, firstTimeToday: false,
  })),
  waiting: waitingIds.map((id) => ({
    id, name: id, displayName: id, joinCount: 0, lastJoinAt: null,
    enqueuedAt: 0, manualOrder: null, firstTimeToday: true,
  })),
  waitingTotal: waitingIds.length,
  trash: [],
  enabled: true,
  language: "ja",
  maxWaiting: 3,
  theme: "midnight",
});

describe("useStateSync event plumbing", () => {
  beforeEach(() => { handlers.length = 0; });
  afterEach(() => { handlers.length = 0; });

  it("mounting a component that calls useStateSync registers a listener", async () => {
    const { useStateSync, useStore } = await import("../store");
    function Probe() { useStateSync(); return null; }
    render(<Probe />);
    await waitFor(() => expect(handlers.length).toBeGreaterThan(0));

    act(() => {
      handlers[0]({ payload: freshSnap(["alpha"], ["beta"]) });
    });
    expect(useStore.getState().snap?.playing[0]?.id).toBe("alpha");
    expect(useStore.getState().snap?.waiting[0]?.id).toBe("beta");
  });

  it("multiple mounted components (main + overlay simulation) both reflect store updates", async () => {
    const { useStateSync, useStore } = await import("../store");
    function Window({ label }: { label: string }) {
      useStateSync();
      const snap = useStore((s) => s.snap);
      return <div data-testid={label}>{snap?.playing.map((u) => u.id).join(",") ?? "-"}</div>;
    }
    const { getByTestId } = render(
      <>
        <Window label="main" />
        <Window label="overlay" />
      </>
    );
    await waitFor(() => expect(handlers.length).toBeGreaterThanOrEqual(2));

    act(() => {
      const snap = freshSnap(["u1", "u2"], []);
      handlers.forEach((h) => h({ payload: snap }));
    });
    expect(getByTestId("main").textContent).toBe("u1,u2");
    expect(getByTestId("overlay").textContent).toBe("u1,u2");

    act(() => {
      const snap = freshSnap(["u2"], []); // u1 deleted — overlay must drop u1
      handlers.forEach((h) => h({ payload: snap }));
    });
    expect(getByTestId("main").textContent).toBe("u2");
    expect(getByTestId("overlay").textContent).toBe("u2");
  });

  it("unmount removes the listener so stale windows stop updating", async () => {
    const { useStateSync } = await import("../store");
    function Probe() { useStateSync(); return null; }
    const { unmount } = render(<Probe />);
    await waitFor(() => expect(handlers.length).toBe(1));
    unmount();
    await waitFor(() => expect(handlers.length).toBe(0));
  });
});
