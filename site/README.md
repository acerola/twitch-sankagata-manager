# site/

Landing page for **Twitch Sankagata Manager**. Static HTML/CSS/JS, no build step. Served via GitHub Pages.

## Local preview

```bash
cd site && python -m http.server 8080
# or: bunx serve .
```

Open http://localhost:8080

## Files

- `index.html` — sections: nav, hero (with mascot + drifting stars), features, live preview (mock stream + app window), languages, download, changelog, footer
- `styles.css` — neon kawaii dark theme. Tokens at `:root`.
- `app.js` — drifts star particles, wires the language pill, auto-detects user OS, fetches latest GitHub Releases (download cards + changelog), tiny markdown renderer.
- `assets/mascot.png`, `assets/mascot-128.png` — mascot (chibi catgirl) used in hero and footer.
- `assets/icon.png`, `assets/icon-256.png` — legacy app-icon copies (not currently referenced; kept for re-use).

## Configuration

`app.js` first line:

```js
const REPO = "acerola/twitch-sankagata-manager";
```

Once a tagged release ships with `.exe` / `.dmg` assets, the download cards and changelog populate automatically — no static `latest.json` needed.

## Deploy

`.github/workflows/pages.yml` redeploys on any push to `main` that touches `site/**`.

GitHub repo → **Settings → Pages → Source: GitHub Actions**.
