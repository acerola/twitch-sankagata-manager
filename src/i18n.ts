import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import ja from "../locales/ja.json";
import en from "../locales/en.json";
import ko from "../locales/ko.json";

i18n.use(initReactI18next).init({
  resources: { ja: { translation: ja }, en: { translation: en }, ko: { translation: ko } },
  lng: (navigator.language.startsWith("ko") ? "ko" : navigator.language.startsWith("en") ? "en" : "ja"),
  fallbackLng: "ja",
  interpolation: { escapeValue: false, prefix: "{", suffix: "}" },
});

export default i18n;
