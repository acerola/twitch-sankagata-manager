import { act, fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import i18n from "../../i18n";
import { useStore } from "../../store";
import type { Config, Snapshot } from "../../types";
import { Header } from "../Header";

const mocks = vi.hoisted(() => {
  const overlayWindow = {
    isVisible: vi.fn(async () => true),
    show: vi.fn(async () => {}),
    hide: vi.fn(async () => {}),
  };
  return {
    startDragging: vi.fn(async () => {}),
    isMaximized: vi.fn(async () => false),
    minimize: vi.fn(async () => {}),
    maximize: vi.fn(async () => {}),
    unmaximize: vi.fn(async () => {}),
    close: vi.fn(async () => {}),
    onResized: vi.fn(async () => () => {}),
    overlayWindow,
    emit: vi.fn(async (_event: string, _payload?: unknown) => {}),
  };
});

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    startDragging: mocks.startDragging,
    isMaximized: mocks.isMaximized,
    minimize: mocks.minimize,
    maximize: mocks.maximize,
    unmaximize: mocks.unmaximize,
    close: mocks.close,
    onResized: mocks.onResized,
  }),
}));

vi.mock("@tauri-apps/api/webviewWindow", () => ({
  WebviewWindow: {
    getByLabel: vi.fn(async () => mocks.overlayWindow),
  },
}));

vi.mock("@tauri-apps/plugin-clipboard-manager", () => ({
  writeText: vi.fn(async () => {}),
}));

vi.mock("@tauri-apps/api/event", () => ({
  emit: (event: string, payload?: unknown) => mocks.emit(event, payload),
  listen: vi.fn(async () => () => {}),
}));

vi.mock("../../ipc", () => ({
  ipc: {
    getAuthStatus: vi.fn(async () => ({ authenticated: true, loginName: null })),
    setEnabled: vi.fn(async () => {}),
    logout: vi.fn(async () => {}),
    getSessionId: vi.fn(async () => null),
    debugSeedUsers: vi.fn(async () => {}),
    debugSeedLongNames: vi.fn(async () => {}),
    debugClearQueues: vi.fn(async () => {}),
    debugRefundFirst: vi.fn(async () => {}),
    resetCounts: vi.fn(async () => {}),
  },
}));

const baseConfig: Config = {
  firstTimeKeyword: "参加券",
  maxPlaying: 4,
  maxWaiting: 3,
  prioritizeFirstTimers: true,
  enabled: true,
  language: "en",
  port: 24816,
  mockMode: false,
  theme: "midnight",
};

const baseSnap: Snapshot = {
  type: "state",
  playing: [],
  waiting: [],
  waitingTotal: 0,
  trash: [],
  enabled: true,
  language: "en",
  maxWaiting: 3,
  theme: "midnight",
};

function pointerDown(element: Element, button = 0) {
  fireEvent(element, new MouseEvent("pointerdown", { bubbles: true, cancelable: true, button }));
}

describe("Header window dragging", () => {
  beforeEach(async () => {
    vi.clearAllMocks();
    await i18n.changeLanguage("en");
    act(() => {
      useStore.setState({
        config: baseConfig,
        snap: baseSnap,
        _syncCount: 0,
      });
    });
  });

  it("starts native drag from the header background", () => {
    const { container } = render(<Header onOpenSettings={() => {}} onLoggedOut={() => {}} />);

    pointerDown(container.querySelector(".app-header")!);

    expect(mocks.startDragging).toHaveBeenCalledTimes(1);
  });

  it("does not start native drag when closing a header modal from the backdrop", async () => {
    const { container } = render(<Header onOpenSettings={() => {}} onLoggedOut={() => {}} />);

    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: "Theme" }));
    });
    const backdrop = container.querySelector(".modal-backdrop");
    expect(backdrop).not.toBeNull();

    mocks.startDragging.mockClear();
    pointerDown(backdrop!);
    await act(async () => {
      fireEvent.click(backdrop!);
    });

    expect(mocks.startDragging).not.toHaveBeenCalled();
  });
});
