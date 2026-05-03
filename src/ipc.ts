import { invoke } from "@tauri-apps/api/core";
import type { Config, Snapshot, Zone } from "./types";

export type AuthStatus = { authenticated: boolean; loginName: string | null };
export type DeviceStart = { deviceCode: string; userCode: string; verificationUri: string };
export type InstallMode = {
  kind: "installed" | "portable" | "unknown";
  portable: boolean;
  detail: string;
};

export const ipc = {
  getSnapshot: () => invoke<Snapshot>("get_snapshot"),
  getConfig: () => invoke<Config>("get_config"),
  getSessionId: () => invoke<string | null>("get_session_id"),
  setConfig: (cfg: Config) => invoke<Snapshot>("set_config", { cfg }),
  setEnabled: (enabled: boolean) => invoke<Snapshot>("set_enabled", { enabled }),
  trashUser: (userId: string) => invoke<Snapshot>("trash_user", { userId }),
  restoreUser: (userId: string) => invoke<Snapshot>("restore_user", { userId }),
  clearTrash: () => invoke<Snapshot>("clear_trash"),
  clearPlaying: () => invoke<Snapshot>("clear_playing"),
  clearPlayingUser: (userId: string) => invoke<Snapshot>("clear_playing_user", { userId }),
  moveUser: (userId: string, zone: Zone, index: number) =>
    invoke<Snapshot>("move_user", { userId, zone, index }),
  resetCounts: () => invoke<Snapshot>("reset_counts"),
  listRewards: () => invoke<Array<{ id: string; title: string; cost: number }>>("list_rewards"),
  getInstallMode: () => invoke<InstallMode>("get_install_mode"),
  getAuthStatus: () => invoke<AuthStatus>("get_auth_status"),
  startAuth: () => invoke<DeviceStart>("start_auth"),
  logout: () => invoke<void>("logout"),
  debugSeedUsers: (count: number) => invoke<Snapshot>("debug_seed_users", { count }),
  debugSeedLongNames: () => invoke<Snapshot>("debug_seed_long_names"),
  debugClearQueues: () => invoke<Snapshot>("debug_clear_queues"),
  debugRefundFirst: () => invoke<Snapshot>("debug_refund_first"),
  debugTriggerMockRedemption: (title: string) =>
    invoke<Snapshot>("debug_trigger_mock_redemption", { title }),
};
