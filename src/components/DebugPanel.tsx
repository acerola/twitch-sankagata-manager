import { useState } from "react";
import { ipc } from "../ipc";
import { useStore } from "../store";
import type { Config } from "../types";

type Action = { label: string; run: () => Promise<unknown>; danger?: boolean };

export function DebugPanel({ onClose }: { onClose: () => void }) {
  const [running, setRunning] = useState<string | null>(null);
  const [lastResult, setLastResult] = useState<string>("");
  const config = useStore((s) => s.config);
  const setConfigInStore = useStore((s) => s.setConfig);

  const run = async (label: string, fn: () => Promise<unknown>) => {
    setRunning(label);
    setLastResult("");
    try {
      await fn();
      setLastResult(`✓ ${label}`);
    } catch (e) {
      setLastResult(`✗ ${label}: ${e}`);
      console.error(label, e);
    } finally {
      setRunning(null);
    }
  };

  const toggleMockMode = async () => {
    if (!config) return;
    const newConfig: Config = { ...config, mockMode: !config.mockMode };
    await setConfigInStore(newConfig);
    setLastResult(`Mock mode ${!config.mockMode ? 'ENABLED' : 'DISABLED'} - reconnecting...`);
  };

  const seedActions: Action[] = [
    { label: "Seed 5 mixed users", run: () => ipc.debugSeedUsers(5) },
    { label: "Seed 20 mixed users (priority stress)", run: () => ipc.debugSeedUsers(20) },
    { label: "Seed long-name users (ellipsis test)", run: () => ipc.debugSeedLongNames() },
    { label: "Refund first playing user", run: () => ipc.debugRefundFirst() },
    { label: "Clear all queues (keeps history)", run: () => ipc.debugClearQueues(), danger: true },
    { label: "Reset counts + manual order", run: () => ipc.resetCounts(), danger: true },
  ];

  const mockActions: Action[] = [
    {
      label: "⚡ Mock redemption (1回目)",
      run: async () => {
        if (!config?.mockMode) throw new Error("Enable Mock Mode first!");
        await ipc.debugTriggerMockRedemption("参加券(1回目)");
        throw new Error("Mock redemption injected!");
      },
    },
    {
      label: "⚡ Mock redemption (2回目以降)",
      run: async () => {
        if (!config?.mockMode) throw new Error("Enable Mock Mode first!");
        await ipc.debugTriggerMockRedemption("参加券(2回目以降)");
        throw new Error("Mock redemption injected!");
      },
    },
    {
      label: "🔄 Mock 5 redemptions",
      run: async () => {
        if (!config?.mockMode) throw new Error("Enable Mock Mode first!");
        for (let i = 0; i < 5; i++) {
          await ipc.debugTriggerMockRedemption("参加券(1回目)");
          await new Promise((r) => setTimeout(r, 50));
        }
        throw new Error("5 mock redemptions injected!");
      },
    },
    {
      label: "📋 Copy CLI command (1回目)",
      run: async () => {
        if (!config?.mockMode) throw new Error("Enable Mock Mode first!");
        const cmd = `twitch event trigger add-redemption \\\n  -T websocket \\\n  -t 123456789 -f 987654321 \\\n  -i reward-first -n "参加券(1回目)" \\\n  -S fulfilled`;
        await navigator.clipboard.writeText(cmd);
        throw new Error("CLI command copied! Paste in terminal.");
      },
    },
    {
      label: "📋 Copy CLI command (2回目以降)",
      run: async () => {
        if (!config?.mockMode) throw new Error("Enable Mock Mode first!");
        const cmd = `twitch event trigger add-redemption \\\n  -T websocket \\\n  -t 123456789 -f 987654322 \\\n  -i reward-return -n "参加券(2回目以降)" \\\n  -S fulfilled`;
        await navigator.clipboard.writeText(cmd);
        throw new Error("CLI command copied! Paste in terminal.");
      },
    },
  ];

  return (
    <div className="modal-backdrop" onClick={onClose}>
      <div className="modal debug-modal" onClick={(e) => e.stopPropagation()}>
        <h2>🧪 Debug (dev only)</h2>
        <div className="debug-modal-body">
          <p className="debug-note">
            These commands are disabled in release builds.
          </p>

          <h3>🎭 Twitch CLI Mock</h3>
          <div className="debug-toggle-row">
            <label>
              <input
                type="checkbox"
                checked={config?.mockMode ?? false}
                onChange={toggleMockMode}
                disabled={running !== null}
              />
              <span>Mock Mode</span>
            </label>
            <span className={config?.mockMode ? "debug-status is-on" : "debug-status"}>
              {config?.mockMode ? "✅ Using ws://localhost:8081" : "❌ Using production Twitch"}
            </span>
          </div>
          <div className="debug-actions">
            {mockActions.map((a) => (
              <button
                key={a.label}
                className={a.danger ? "danger" : ""}
                disabled={running !== null}
                onClick={() => run(a.label, a.run)}
              >
                {running === a.label ? "…" : a.label}
              </button>
            ))}
          </div>

          <h3>🧪 Seed Data</h3>
          <div className="debug-actions">
            {seedActions.map((a) => (
              <button
                key={a.label}
                className={a.danger ? "danger" : ""}
                disabled={running !== null}
                onClick={() => run(a.label, a.run)}
              >
                {running === a.label ? "…" : a.label}
              </button>
            ))}
          </div>

          {lastResult && <div className="debug-result">{lastResult}</div>}
        </div>

        <div className="modal-actions">
          <span style={{ flex: 1 }} />
          <button onClick={onClose}>Close</button>
        </div>
      </div>
    </div>
  );
}
