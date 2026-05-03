import { useState } from "react";
import { useTranslation } from "react-i18next";
import { useStore } from "../store";
import { ipc } from "../ipc";
import type { Config, Language } from "../types";
import { ask } from "@tauri-apps/plugin-dialog";

const USER_COUNT_OPTIONS = [1, 2, 3, 4, 5] as const;

function clampUserCount(value: number) {
  return Math.min(5, Math.max(1, Math.trunc(value) || 1));
}

function normalizeCountSettings(cfg: Config): Config {
  return {
    ...cfg,
    maxPlaying: clampUserCount(cfg.maxPlaying),
    maxWaiting: clampUserCount(cfg.maxWaiting),
  };
}

function CountRadioGroup({
  label,
  name,
  value,
  onChange,
}: {
  label: string;
  name: string;
  value: number;
  onChange: (value: number) => void;
}) {
  return (
    <fieldset className="count-radio-field">
      <legend>{label}</legend>
      <div className="count-radio-group">
        {USER_COUNT_OPTIONS.map((option) => (
          <label
            key={option}
            className={option === value ? "count-radio is-selected" : "count-radio"}
          >
            <input
              type="radio"
              name={name}
              value={option}
              checked={option === value}
              onChange={() => onChange(option)}
            />
            <span>{option}</span>
          </label>
        ))}
      </div>
    </fieldset>
  );
}

export function SettingsModal({ onClose }: { onClose: () => void }) {
  const { t } = useTranslation();
  const current = useStore((s) => s.config);
  const setConfigInStore = useStore((s) => s.setConfig);
  const [cfg, setCfg] = useState<Config | null>(() =>
    current ? normalizeCountSettings(current) : current
  );

  if (!cfg) return null;

  const save = async () => {
    await setConfigInStore(normalizeCountSettings(cfg));
    onClose();
  };

  const reset = async () => {
    const confirmed = await ask(t("settings.resetWarning"), {
      title: t("settings.resetTitle"),
      kind: "warning",
      okLabel: t("settings.resetConfirm"),
      cancelLabel: t("settings.cancel"),
    });
    if (confirmed) {
      await ipc.resetCounts();
      onClose();
    }
  };

  return (
    <div className="modal-backdrop">
      <div className="modal">
        <h2>{t("settings.title")}</h2>
        <label>
          {t("settings.keyword")}
          <input
            value={cfg.firstTimeKeyword}
            placeholder={t("settings.keywordPlaceholder")}
            onChange={(e) => setCfg({ ...cfg, firstTimeKeyword: e.target.value })}
          />
          <span className="field-help">{t("settings.keywordHelp")}</span>
        </label>
        <CountRadioGroup
          label={t("settings.maxPlaying")}
          name="max-playing"
          value={cfg.maxPlaying}
          onChange={(maxPlaying) => setCfg({ ...cfg, maxPlaying })}
        />
        <CountRadioGroup
          label={t("settings.maxWaiting")}
          name="max-waiting"
          value={cfg.maxWaiting}
          onChange={(maxWaiting) => setCfg({ ...cfg, maxWaiting })}
        />
        <label>
          <span>
            <input
              type="checkbox"
              checked={cfg.prioritizeFirstTimers}
              onChange={(e) => setCfg({ ...cfg, prioritizeFirstTimers: e.target.checked })}
            />
            {t("settings.prioritizeFirstTimers")}
          </span>
        </label>
        <label>
          {t("settings.language")}
          <select value={cfg.language} onChange={(e) => setCfg({ ...cfg, language: e.target.value as Language })}>
            <option value="ja">日本語</option>
            <option value="en">English</option>
            <option value="ko">한국어</option>
          </select>
        </label>
        <div className="modal-actions">
          <div className="reset-action">
            <button onClick={reset} className="danger">{t("settings.reset")}</button>
            <span
              className="help-chip"
              tabIndex={0}
              aria-label={t("settings.resetHelp")}
            >
              ?
              <span className="help-tooltip">{t("settings.resetHelp")}</span>
            </span>
          </div>
          <span style={{ flex: 1 }} />
          <button onClick={onClose}>{t("settings.cancel")}</button>
          <button onClick={save} className="primary">{t("settings.save")}</button>
        </div>
      </div>
    </div>
  );
}
