import { openUrl } from "@tauri-apps/plugin-opener";
import { relaunch } from "@tauri-apps/plugin-process";
import { check, type DownloadEvent, type Update } from "@tauri-apps/plugin-updater";
import { ipc, type InstallMode } from "./ipc";

export const RELEASE_URL = "https://github.com/acerola/twitch-sankagata-manager/releases/latest";

export type UpdateProgress = {
  downloaded: number;
  total?: number;
  percent?: number;
  done: boolean;
};

export type UpdateInfo = {
  update: Update;
  installMode: InstallMode;
  currentVersion: string;
  version: string;
  body?: string;
  date?: string;
  releaseUrl: string;
};

function nextProgress(progress: UpdateProgress, event: DownloadEvent): UpdateProgress {
  if (event.event === "Started") {
    return {
      downloaded: 0,
      total: event.data.contentLength,
      percent: event.data.contentLength ? 0 : undefined,
      done: false,
    };
  }

  if (event.event === "Progress") {
    const downloaded = progress.downloaded + event.data.chunkLength;
    return {
      ...progress,
      downloaded,
      percent: progress.total ? Math.min(100, (downloaded / progress.total) * 100) : undefined,
    };
  }

  return {
    ...progress,
    percent: progress.total ? 100 : progress.percent,
    done: true,
  };
}

export async function checkForUpdate(): Promise<UpdateInfo | null> {
  try {
    const update = await check();
    if (!update) return null;

    const installMode = await ipc.getInstallMode().catch((): InstallMode => ({
      kind: "unknown",
      portable: false,
      detail: "install mode check failed",
    }));

    return {
      update,
      installMode,
      currentVersion: update.currentVersion,
      version: update.version,
      body: update.body,
      date: update.date,
      releaseUrl: RELEASE_URL,
    };
  } catch (e) {
    console.error("update check failed", e);
    return null;
  }
}

export async function downloadUpdate(
  update: Update,
  onProgress: (progress: UpdateProgress) => void,
) {
  let progress: UpdateProgress = { downloaded: 0, done: false };
  await update.download((event) => {
    progress = nextProgress(progress, event);
    onProgress(progress);
  });
  if (!progress.done) {
    progress = { ...progress, percent: progress.total ? 100 : progress.percent, done: true };
    onProgress(progress);
  }
}

export async function installAndRelaunch(update: Update) {
  await update.install();
  await relaunch();
}

export async function openReleasePage(url = RELEASE_URL) {
  await openUrl(url);
}
