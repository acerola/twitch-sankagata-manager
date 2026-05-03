/* === Build script: prerender per-language static HTML for SEO ===
 *
 * Reads:  site/index.html (template)
 *         site/i18n.mjs   (dictionary + lang metadata)
 *
 * Writes: site/dist/index.html        (en)
 *         site/dist/<lang>/index.html (every other lang)
 *         site/dist/{styles.css,app.js,assets/}
 *
 * Why pre-render: search engines and social-card scrapers read raw HTML.
 * Runtime JS i18n meant only the English string ever made it into Google's
 * index, OG previews, etc. With per-lang HTML, JP/KO content is real and
 * crawlable. Pill becomes <a> nav links between the static pages.
 *
 * Env:
 *   SITE_BASE — URL prefix for assets + cross-page links. Default "/".
 *               GitHub Pages on a project repo wants "/<repo-name>/".
 *   SITE_URL  — Absolute origin used for canonical and og:url.
 *               Default "https://acerola.github.io${SITE_BASE}".
 */

import {
  cpSync,
  existsSync,
  mkdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { parseHTML } from "linkedom";
import { I18N, LANG_META } from "./i18n.mjs";

const SITE = dirname(fileURLToPath(import.meta.url));
const DIST = join(SITE, "dist");

let BASE = process.env.SITE_BASE || "/";
if (!BASE.endsWith("/")) BASE += "/";
const SITE_URL = process.env.SITE_URL || `https://acerola.github.io${BASE}`;

const LANGS = Object.keys(I18N);
// Display order in the lang-pill (independent from build-output order).
const PILL_ORDER = ["ja", "en", "ko"];
const langDir = (lang) => (lang === "en" ? "" : `${lang}/`);

function clean() {
  if (existsSync(DIST)) rmSync(DIST, { recursive: true, force: true });
  mkdirSync(DIST, { recursive: true });
}

function copyAssets() {
  for (const f of ["styles.css", "app.js"]) {
    cpSync(join(SITE, f), join(DIST, f));
  }
  cpSync(join(SITE, "assets"), join(DIST, "assets"), { recursive: true });
}

/**
 * Rewrite relative asset references in a parsed document so both
 * `/index.html` and `/ja/index.html` resolve to the same `/styles.css`,
 * `/app.js`, and `/assets/*`. Without this, the JA page would try to
 * load `/ja/styles.css` and 404.
 */
function absolutizeAssetUrls(document) {
  const rewrites = [
    ['link[rel="stylesheet"][href]', "href"],
    ['link[rel="icon"][href]', "href"],
    ["script[src]", "src"],
    ["img[src]", "src"],
  ];
  for (const [sel, attr] of rewrites) {
    document.querySelectorAll(sel).forEach((el) => {
      const v = el.getAttribute(attr);
      if (
        !v ||
        v.startsWith("http://") ||
        v.startsWith("https://") ||
        v.startsWith("//") ||
        v.startsWith("/") ||
        v.startsWith("#") ||
        v.startsWith("data:")
      )
        return;
      el.setAttribute(attr, BASE + v);
    });
  }
}

/**
 * Replace lang-pill <button>s (which would only work via JS) with proper
 * <a> elements that crawl-able and let users open another lang in a new
 * tab. Marks the current lang's link as is-active + aria-current.
 */
function rebuildLangPill(document, currentLang) {
  const pill = document.querySelector(".lang-pill");
  if (!pill) return;
  pill.innerHTML = "";
  pill.removeAttribute("role");
  pill.removeAttribute("aria-label");
  pill.setAttribute("aria-label", "Language");
  for (const lang of PILL_ORDER) {
    const a = document.createElement("a");
    a.setAttribute("href", `${BASE}${langDir(lang)}`);
    a.setAttribute("data-lang", lang);
    a.setAttribute("hreflang", LANG_META[lang].hreflang);
    if (lang === currentLang) {
      a.setAttribute("class", "is-active");
      a.setAttribute("aria-current", "page");
    }
    a.textContent = LANG_META[lang].pillLabel;
    pill.appendChild(a);
  }
}

function appendMeta(document, attrName, attrValue, content) {
  const m = document.createElement("meta");
  m.setAttribute(attrName, attrValue);
  m.setAttribute("content", content);
  document.head.appendChild(m);
}

function appendLink(document, attrs) {
  const l = document.createElement("link");
  for (const [k, v] of Object.entries(attrs)) l.setAttribute(k, v);
  document.head.appendChild(l);
}

function applyHeadSeo(document, lang, dict) {
  // <title>
  let title = document.querySelector("title");
  if (!title) {
    title = document.createElement("title");
    document.head.appendChild(title);
  }
  title.textContent = dict["meta.title"];

  // <meta name="description">
  let desc = document.querySelector('meta[name="description"]');
  if (!desc) {
    desc = document.createElement("meta");
    desc.setAttribute("name", "description");
    document.head.appendChild(desc);
  }
  desc.setAttribute("content", dict["meta.description"]);

  // canonical
  appendLink(document, {
    rel: "canonical",
    href: `${SITE_URL}${langDir(lang)}`,
  });

  // hreflang alternates per lang + x-default → en
  for (const l of LANGS) {
    appendLink(document, {
      rel: "alternate",
      hreflang: LANG_META[l].hreflang,
      href: `${SITE_URL}${langDir(l)}`,
    });
  }
  appendLink(document, {
    rel: "alternate",
    hreflang: "x-default",
    href: SITE_URL,
  });

  // Open Graph
  appendMeta(document, "property", "og:type", "website");
  appendMeta(document, "property", "og:locale", LANG_META[lang].ogLocale);
  appendMeta(document, "property", "og:title", dict["meta.title"]);
  appendMeta(document, "property", "og:description", dict["meta.description"]);
  appendMeta(
    document,
    "property",
    "og:url",
    `${SITE_URL}${langDir(lang)}`,
  );
  appendMeta(
    document,
    "property",
    "og:image",
    `${SITE_URL}assets/icon-256.png`,
  );

  // Twitter
  appendMeta(document, "name", "twitter:card", "summary_large_image");
  appendMeta(document, "name", "twitter:title", dict["meta.title"]);
  appendMeta(document, "name", "twitter:description", dict["meta.description"]);
  appendMeta(
    document,
    "name",
    "twitter:image",
    `${SITE_URL}assets/icon-256.png`,
  );
}

function substituteText(document, dict) {
  document.querySelectorAll("[data-i18n]").forEach((el) => {
    const k = el.getAttribute("data-i18n");
    if (dict[k] != null) el.textContent = dict[k];
  });
  document.querySelectorAll("[data-i18n-html]").forEach((el) => {
    const k = el.getAttribute("data-i18n-html");
    if (dict[k] != null) el.innerHTML = dict[k];
  });
}

function buildLang(lang, templateHtml) {
  const dict = I18N[lang];
  const { document } = parseHTML(templateHtml);

  document.documentElement.setAttribute("lang", lang);

  applyHeadSeo(document, lang, dict);
  absolutizeAssetUrls(document);
  rebuildLangPill(document, lang);
  substituteText(document, dict);

  return "<!doctype html>\n" + document.documentElement.outerHTML;
}

function build() {
  console.log(`[build] base=${BASE}`);
  console.log(`[build] site_url=${SITE_URL}`);
  clean();
  const template = readFileSync(join(SITE, "index.html"), "utf8");
  for (const lang of LANGS) {
    const html = buildLang(lang, template);
    const outDir = join(DIST, langDir(lang));
    mkdirSync(outDir, { recursive: true });
    writeFileSync(join(outDir, "index.html"), html);
    console.log(`[build] wrote ${langDir(lang)}index.html`);
  }
  copyAssets();
  console.log(`[build] copied styles.css, app.js, assets/`);
  console.log(`[build] done → ${DIST}`);
}

build();
