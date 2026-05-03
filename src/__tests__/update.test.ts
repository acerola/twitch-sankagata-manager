import { beforeEach, describe, expect, it, vi } from "vitest";
import { openUrl } from "@tauri-apps/plugin-opener";
import { relaunch } from "@tauri-apps/plugin-process";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { ipc } from "../ipc";
import {
  RELEASE_URL,
  checkForUpdate,
  downloadUpdate,
  installAndRelaunch,
  openReleasePage,
} from "../update";

vi.mock("@tauri-apps/plugin-updater", () => ({
  check: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-opener", () => ({
  openUrl: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-process", () => ({
  relaunch: vi.fn(),
}));

vi.mock("../ipc", () => ({
  ipc: {
    getInstallMode: vi.fn(),
  },
}));

const mockCheck = vi.mocked(check);
const mockOpenUrl = vi.mocked(openUrl);
const mockRelaunch = vi.mocked(relaunch);
const mockGetInstallMode = vi.mocked(ipc.getInstallMode);

describe("checkForUpdate", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockGetInstallMode.mockResolvedValue({
      kind: "installed",
      portable: false,
      detail: "test",
    });
  });

  it("does nothing when no update is available", async () => {
    mockCheck.mockResolvedValue(null);

    const update = await checkForUpdate();

    expect(update).toBeNull();
    expect(mockCheck).toHaveBeenCalledOnce();
    expect(mockGetInstallMode).not.toHaveBeenCalled();
    expect(mockRelaunch).not.toHaveBeenCalled();
  });

  it("returns update info with install mode for the app modal", async () => {
    mockCheck.mockResolvedValue({
      currentVersion: "0.2.0",
      version: "0.2.1",
      body: "notes",
      date: "2026-05-02",
    } as Update);

    const info = await checkForUpdate();

    expect(info).toMatchObject({
      currentVersion: "0.2.0",
      version: "0.2.1",
      body: "notes",
      date: "2026-05-02",
      releaseUrl: RELEASE_URL,
      installMode: {
        kind: "installed",
        portable: false,
        detail: "test",
      },
    });
  });

  it("treats install mode check failures as unknown installed mode", async () => {
    mockGetInstallMode.mockRejectedValue(new Error("unavailable"));
    mockCheck.mockResolvedValue({
      currentVersion: "0.2.0",
      version: "0.2.1",
    } as Update);

    const info = await checkForUpdate();

    expect(info?.installMode).toEqual({
      kind: "unknown",
      portable: false,
      detail: "install mode check failed",
    });
  });

  it("downloads with progress without installing automatically", async () => {
    const progress = vi.fn();
    const download = vi.fn(async (onEvent) => {
      onEvent({ event: "Started", data: { contentLength: 100 } });
      onEvent({ event: "Progress", data: { chunkLength: 25 } });
      onEvent({ event: "Progress", data: { chunkLength: 75 } });
      onEvent({ event: "Finished" });
    });

    await downloadUpdate({ download } as unknown as Update, progress);

    expect(download).toHaveBeenCalledOnce();
    expect(progress).toHaveBeenLastCalledWith({
      downloaded: 100,
      total: 100,
      percent: 100,
      done: true,
    });
  });

  it("installs and relaunches only when requested", async () => {
    const install = vi.fn(async () => {});
    mockRelaunch.mockResolvedValue(undefined);

    await installAndRelaunch({ install } as unknown as Update);

    expect(install).toHaveBeenCalledOnce();
    expect(mockRelaunch).toHaveBeenCalledOnce();
  });

  it("opens the release page for portable updates", async () => {
    mockOpenUrl.mockResolvedValue(undefined);

    await openReleasePage();

    expect(mockOpenUrl).toHaveBeenCalledWith(RELEASE_URL);
  });

  it("logs updater check failures without throwing", async () => {
    const error = new Error("offline");
    const consoleError = vi.spyOn(console, "error").mockImplementation(() => {});
    mockCheck.mockRejectedValue(error);

    await expect(checkForUpdate()).resolves.toBeNull();

    expect(consoleError).toHaveBeenCalledWith("update check failed", error);
    consoleError.mockRestore();
  });
});
