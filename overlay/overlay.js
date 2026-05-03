(function () {
  const root = document.getElementById("root");
  const moreTmpl = { ja: "+ {n} 人待機", en: "+ {n} more waiting", ko: "+ {n} 명 대기" };
  const badgeTmpl = { ja: "初回", en: "NEW", ko: "처음" };

  function badge(u, lang) {
    return u.firstTimeToday ? `<span class="badge">${escape(badgeTmpl[lang] || badgeTmpl.ja)}</span>` : '';
  }

  function render(snap) {
    const lang = snap.language || "ja";
    if (snap.theme) {
      document.body.setAttribute("data-theme", snap.theme);
    }
    const parts = [];
    for (const u of snap.playing) {
      parts.push(`<div class="row playing"><span class="name">${escape(u.displayName)}</span>${badge(u, lang)}</div>`);
    }
    const visible = snap.waiting.slice(0, snap.maxWaiting || 3);
    for (const u of visible) {
      parts.push(`<div class="row waiting"><span class="name">${escape(u.displayName)}</span>${badge(u, lang)}</div>`);
    }
    const hidden = snap.waitingTotal - visible.length;
    if (hidden > 0) {
      parts.push(`<div class="more">${(moreTmpl[lang] || moreTmpl.ja).replace("{n}", hidden)}</div>`);
    }
    root.innerHTML = parts.join("");
  }

  function escape(s) { return String(s).replace(/[&<>"']/g, c => ({ "&":"&amp;","<":"&lt;",">":"&gt;",'"':"&quot;","'":"&#39;" }[c])); }

  function connect() {
    const ws = new WebSocket(`ws://${location.host}/ws`);
    ws.onmessage = (e) => {
      try { render(JSON.parse(e.data)); } catch (err) { console.error(err); }
    };
    ws.onclose = () => setTimeout(connect, 1000);
  }
  connect();
})();
