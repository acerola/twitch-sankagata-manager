import { useState, useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";
import { emit } from "@tauri-apps/api/event";
import { useStore } from "../store";
import { applyTheme, getDefaultCustomColors } from "../theme-utils";
import type { Theme, CustomColors } from "../types";

const THEMES: { id: Theme; label: string; preview: string[] }[] = [
  {
    id: "twitch",
    label: "theme.twitch",
    preview: ["#1e1a2e", "#ff4d94", "#c4a8e0"],
  },
  {
    id: "midnight",
    label: "theme.midnight",
    preview: ["#0f0f18", "#16e9f3", "#ff7bc4"],
  },
  {
    id: "daylight",
    label: "theme.daylight",
    preview: ["#f0f0f5", "#0066cc", "#cc0066"],
  },
  {
    id: "sakura",
    label: "theme.sakura",
    preview: ["#1a0f1a", "#ff66b2", "#cc99ff"],
  },
  {
    id: "forest",
    label: "theme.forest",
    preview: ["#0f1a12", "#00cc88", "#88cc00"],
  },
  {
    id: "contrast",
    label: "theme.contrast",
    preview: ["#000000", "#00ffff", "#ff00ff"],
  },
  {
    id: "custom",
    label: "theme.custom",
    preview: ["#333333", "#ff6600", "#00ff66"],
  },
];

export function ThemeModal({ onClose }: { onClose: () => void }) {
  const { t } = useTranslation();
  const config = useStore((s) => s.config);
  const setConfigInStore = useStore((s) => s.setConfig);
  const [selected, setSelected] = useState<Theme>(config?.theme ?? "twitch");
  const [customColors, setCustomColors] = useState<CustomColors>(
    config?.customColors ?? getDefaultCustomColors()
  );
  const savedThemeRef = useRef<Theme>(config?.theme ?? "twitch");
  const savedColorsRef = useRef<CustomColors>(
    config?.customColors ?? getDefaultCustomColors()
  );

  // Preview: apply selected theme immediately to body and emit to other windows
  useEffect(() => {
    applyTheme(selected, selected === "custom" ? customColors : undefined);
    emit("theme-preview", selected).catch(() => {});
  }, [selected, customColors]);

  // On unmount without applying, revert to saved theme
  useEffect(() => {
    return () => {
      applyTheme(savedThemeRef.current, savedColorsRef.current);
      emit("theme-preview", savedThemeRef.current).catch(() => {});
    };
  }, []);

  const handleApply = async () => {
    if (config) {
      await setConfigInStore({
        ...config,
        theme: selected,
        customColors: selected === "custom" ? customColors : undefined,
      });
      savedThemeRef.current = selected;
      savedColorsRef.current = customColors;
    }
    onClose();
  };

  const handleCancel = () => {
    applyTheme(savedThemeRef.current, savedColorsRef.current);
    emit("theme-preview", savedThemeRef.current).catch(() => {});
    onClose();
  };

  // Close on Escape key
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") handleCancel();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  const handleColorChange = (key: keyof CustomColors, value: string) => {
    setCustomColors((prev) => ({ ...prev, [key]: value }));
  };

  return (
    <div
      className="modal-backdrop"
      onClick={(e) => {
        if (e.target === e.currentTarget) handleCancel();
      }}
    >
      <div
        className="modal theme-modal"
      >
        <h2>{t("theme.title")}</h2>

        <div className="theme-modal-body">
        <div className="theme-grid">
          {THEMES.map((theme) => (
            <button
              key={theme.id}
              onClick={() => setSelected(theme.id)}
              className={selected === theme.id ? "theme-option is-selected" : "theme-option"}
            >
              <div className="theme-swatches">
                {theme.preview.map((color) => (
                  <div
                    key={color}
                    className="theme-swatch"
                    style={{ background: color }}
                  />
                ))}
              </div>
              <span>{t(theme.label)}</span>
            </button>
          ))}
        </div>

        {selected === "custom" && (
          <div className="custom-colors">
            <h3>{t("theme.customColors")}</h3>
            <div className="custom-color-grid">
              {(
                [
                  { key: "bg", label: t("theme.bgColor") },
                  { key: "primary", label: t("theme.primaryColor") },
                  { key: "secondary", label: t("theme.secondaryColor") },
                  { key: "tertiary", label: t("theme.tertiaryColor") },
                  { key: "text", label: t("theme.textColor") },
                ] as { key: keyof CustomColors; label: string }[]
              ).map(({ key, label }) => (
                <label
                  key={key}
                  className="custom-color-row"
                >
                  <span>{label}</span>
                  <div className="custom-color-controls">
                    <input
                      type="color"
                      value={customColors[key]}
                      onChange={(e) => handleColorChange(key, e.target.value)}
                    />
                    <input
                      type="text"
                      value={customColors[key]}
                      onChange={(e) => handleColorChange(key, e.target.value)}
                    />
                  </div>
                </label>
              ))}
            </div>
          </div>
        )}

        </div>

        <div className="modal-actions">
          <span style={{ flex: 1 }} />
          <button onClick={handleCancel}>{t("theme.cancel")}</button>
          <button onClick={handleApply} className="primary">
            {t("theme.apply")}
          </button>
        </div>
      </div>
    </div>
  );
}
