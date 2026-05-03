export type Zone = "playing" | "waiting" | "trash";

export type User = {
  id: string;
  name: string;
  displayName: string;
  joinCount: number;
  lastJoinAt: number | null;
  enqueuedAt: number;
  manualOrder: number | null;
  firstTimeToday: boolean;
};

export type Language = "ja" | "en" | "ko";
export type Theme = "twitch" | "midnight" | "daylight" | "sakura" | "forest" | "contrast" | "custom";

export type CustomColors = {
  bg: string;
  primary: string;
  secondary: string;
  tertiary: string;
  text: string;
};

export type Config = {
  firstTimeKeyword: string;
  maxPlaying: number;
  maxWaiting: number;
  prioritizeFirstTimers: boolean;
  enabled: boolean;
  language: Language;
  port: number;
  mockMode: boolean;
  theme: Theme;
  customColors?: CustomColors;
};

export type Snapshot = {
  type: "state";
  playing: User[];
  waiting: User[];
  waitingTotal: number;
  trash: User[];
  enabled: boolean;
  language: Language;
  maxWaiting: number;
  theme: Theme;
};
