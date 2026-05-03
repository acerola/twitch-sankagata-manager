import { render, screen, fireEvent, act } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { ThemeModal } from "../ThemeModal";
import { useStore } from "../../store";
import type { Config } from "../../types";

const mockEmit = vi.fn(async (_event: string, _payload?: unknown) => {});
const mockListen = vi.fn(async (_event: string, _handler: unknown) => () => {});

vi.mock("@tauri-apps/api/event", () => ({
  emit: (event: string, payload?: unknown) => mockEmit(event, payload),
  listen: (event: string, handler: unknown) => mockListen(event, handler),
}));

const baseConfig: Config = {
  firstTimeKeyword: "参加券",
  maxPlaying: 4,
  maxWaiting: 3,
  prioritizeFirstTimers: true,
  enabled: true,
  language: "ja",
  port: 24816,
  mockMode: false,
  theme: "midnight",
};

describe("ThemeModal", () => {
  beforeEach(() => {
    mockEmit.mockClear();
    mockListen.mockClear();
    act(() => {
      useStore.setState({
        config: baseConfig,
        snap: null,
      });
    });
  });

  afterEach(() => {
    document.body.removeAttribute("data-theme");
  });

  it("renders all 7 theme options", () => {
    render(<ThemeModal onClose={() => {}} />);
    
    expect(screen.getByText("theme.twitch")).toBeInTheDocument();
    expect(screen.getByText("theme.midnight")).toBeInTheDocument();
    expect(screen.getByText("theme.daylight")).toBeInTheDocument();
    expect(screen.getByText("theme.sakura")).toBeInTheDocument();
    expect(screen.getByText("theme.forest")).toBeInTheDocument();
    expect(screen.getByText("theme.contrast")).toBeInTheDocument();
    expect(screen.getByText("theme.custom")).toBeInTheDocument();
  });

  it("applies theme preview to document body on click", () => {
    render(<ThemeModal onClose={() => {}} />);
    
    const daylightBtn = screen.getByText("theme.daylight").closest("button");
    fireEvent.click(daylightBtn!);
    
    expect(document.body.getAttribute("data-theme")).toBe("daylight");
  });

  it("emits theme-preview event when selecting a theme", () => {
    render(<ThemeModal onClose={() => {}} />);
    
    const forestBtn = screen.getByText("theme.forest").closest("button");
    fireEvent.click(forestBtn!);
    
    expect(mockEmit).toHaveBeenCalledWith("theme-preview", "forest");
  });

  it("reverts theme on cancel and emits revert event", () => {
    render(<ThemeModal onClose={() => {}} />);
    
    // Select a different theme
    const sakuraBtn = screen.getByText("theme.sakura").closest("button");
    fireEvent.click(sakuraBtn!);
    expect(document.body.getAttribute("data-theme")).toBe("sakura");
    
    // Click cancel
    const cancelBtn = screen.getByText("theme.cancel");
    fireEvent.click(cancelBtn);
    
    // Should revert to midnight (the original theme)
    expect(document.body.getAttribute("data-theme")).toBe("midnight");
    expect(mockEmit).toHaveBeenLastCalledWith("theme-preview", "midnight");
  });

  it("reverts theme on unmount without applying", () => {
    const { unmount } = render(<ThemeModal onClose={() => {}} />);
    
    // Select a different theme
    const contrastBtn = screen.getByText("theme.contrast").closest("button");
    fireEvent.click(contrastBtn!);
    expect(document.body.getAttribute("data-theme")).toBe("contrast");
    
    // Unmount without applying
    unmount();
    
    // Should revert to midnight
    expect(document.body.getAttribute("data-theme")).toBe("midnight");
    expect(mockEmit).toHaveBeenLastCalledWith("theme-preview", "midnight");
  });

  it("calls setConfig with new theme on apply", async () => {
    const setConfigSpy = vi.fn();
    act(() => {
      useStore.setState({
        config: baseConfig,
        setConfig: setConfigSpy,
      });
    });
    
    const onClose = vi.fn();
    render(<ThemeModal onClose={onClose} />);
    
    // Select daylight theme
    const daylightBtn = screen.getByText("theme.daylight").closest("button");
    fireEvent.click(daylightBtn!);
    
    // Click apply
    const applyBtn = screen.getByText("theme.apply");
    await act(async () => {
      fireEvent.click(applyBtn);
    });
    
    expect(setConfigSpy).toHaveBeenCalledWith({
      ...baseConfig,
      theme: "daylight",
    });
    expect(onClose).toHaveBeenCalled();
  });

  it("closes modal on Escape key", () => {
    const onClose = vi.fn();
    render(<ThemeModal onClose={onClose} />);
    
    fireEvent.keyDown(window, { key: "Escape" });
    
    expect(onClose).toHaveBeenCalled();
  });

  it("closes modal on backdrop click", () => {
    const onClose = vi.fn();
    const { container } = render(<ThemeModal onClose={onClose} />);
    
    const backdrop = container.querySelector(".modal-backdrop");
    fireEvent.click(backdrop!);
    
    expect(onClose).toHaveBeenCalled();
  });
});
