import type { CustomColors, Theme } from "./types";

const CUSTOM_STYLE_ID = "custom-theme-variables";

function hexToRgba(hex: string, alpha: number): string {
  const r = parseInt(hex.slice(1, 3), 16);
  const g = parseInt(hex.slice(3, 5), 16);
  const b = parseInt(hex.slice(5, 7), 16);
  return `rgba(${r}, ${g}, ${b}, ${alpha})`;
}

function generateCustomCss(colors: CustomColors): string {
  return `
    :root,
    body[data-theme="custom"] {
      --bg: ${colors.bg};
      --surface: ${adjustBrightness(colors.bg, 10)};
      --surface-2: ${adjustBrightness(colors.bg, 15)};
      --surface-3: ${adjustBrightness(colors.bg, 8)};
      --elevated: ${adjustBrightness(colors.bg, 20)};
      --elevated-2: ${adjustBrightness(colors.bg, 18)};
      --accent-primary: ${colors.primary};
      --accent-secondary: ${colors.secondary};
      --accent-tertiary: ${colors.tertiary};
      --warning: #ffd83f;
      --text: ${colors.text};
      --text-muted: ${adjustBrightness(colors.text, -30)};
      --text-dim: ${adjustBrightness(colors.text, -50)};
      --text-inverse: ${colors.bg};
      --border: ${adjustBrightness(colors.bg, 25)};
      --border-hover: ${adjustBrightness(colors.bg, 35)};
      --border-light: ${adjustBrightness(colors.bg, 30)};
      --shadow-glow-primary: ${hexToRgba(colors.primary, 0.4)};
      --shadow-glow-secondary: ${hexToRgba(colors.secondary, 0.4)};
      --shadow-glow-warning: rgba(255, 216, 63, 0.3);
      --overlay-bg: ${hexToRgba(colors.bg, 0.35)};
      --overlay-bg-playing: ${hexToRgba(colors.primary, 0.15)};
      --overlay-bg-waiting: ${hexToRgba(colors.bg, 0.3)};
      --row-playing-bg: ${hexToRgba(colors.primary, 0.08)};
      --row-waiting-bg: ${hexToRgba(colors.tertiary, 0.08)};
    }
  `;
}

function adjustBrightness(hex: string, percent: number): string {
  const num = parseInt(hex.replace("#", ""), 16);
  const amt = Math.round(2.55 * percent);
  const R = Math.min(255, Math.max(0, (num >> 16) + amt));
  const G = Math.min(255, Math.max(0, ((num >> 8) & 0x00ff) + amt));
  const B = Math.min(255, Math.max(0, (num & 0x0000ff) + amt));
  return "#" + (0x1000000 + R * 0x10000 + G * 0x100 + B).toString(16).slice(1);
}

export function applyTheme(theme: Theme, customColors?: CustomColors) {
  // Remove any existing custom theme style
  const existing = document.getElementById(CUSTOM_STYLE_ID);
  if (existing) {
    existing.remove();
  }

  if (theme === "custom" && customColors) {
    // Inject custom theme CSS
    const style = document.createElement("style");
    style.id = CUSTOM_STYLE_ID;
    style.textContent = generateCustomCss(customColors);
    document.head.appendChild(style);
  }

  // Apply theme attribute
  document.body.setAttribute("data-theme", theme);
}

export function getDefaultCustomColors(): CustomColors {
  return {
    bg: "#0f0f18",
    primary: "#16e9f3",
    secondary: "#ff7bc4",
    tertiary: "#ff4fa8",
    text: "#e8e8f0",
  };
}
