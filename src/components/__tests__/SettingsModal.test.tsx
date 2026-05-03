import { act, fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { SettingsModal } from "../SettingsModal";
import { useStore } from "../../store";
import type { Config } from "../../types";

vi.mock("@tauri-apps/plugin-dialog", () => ({
  ask: vi.fn(),
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

describe("SettingsModal", () => {
  it("saves the reward keyword using the backend config field", async () => {
    const setConfig = vi.fn<(cfg: Config) => Promise<void>>(async () => {});
    const onClose = vi.fn();

    act(() => {
      useStore.setState({ config: baseConfig, setConfig });
    });

    render(<SettingsModal onClose={onClose} />);

    fireEvent.change(screen.getByDisplayValue("参加券"), {
      target: { value: "custom-ticket" },
    });

    await act(async () => {
      fireEvent.click(screen.getByText("settings.save"));
    });

    expect(setConfig).toHaveBeenCalledWith({
      ...baseConfig,
      firstTimeKeyword: "custom-ticket",
    });
    expect(setConfig.mock.calls[0][0]).not.toHaveProperty("keyword");
    expect(onClose).toHaveBeenCalled();
  });
});
