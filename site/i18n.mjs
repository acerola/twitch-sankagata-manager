/* === Twitch Sankagata Manager · landing page i18n dictionary ===
 *
 * Single source of truth, imported by both:
 *   - site/build.mjs (build-time prerender → dist/<lang>/index.html)
 *   - site/app.js    (runtime — currently only reads keys for runtime
 *                     features like changelog markers; per-page text is
 *                     baked at build time so crawlers see real lang HTML)
 *
 * Keys map to elements via [data-i18n] (textContent) or
 * [data-i18n-html] (innerHTML, used where the source has nested markup
 * like the hero h1's <span class="accent">).
 *
 * meta.title / meta.description are not bound to DOM nodes — the build
 * script consumes them directly to generate per-lang <title>, og:*,
 * and twitter:* tags.
 */

export const I18N = {
  en: {
    "meta.title":
      "Twitch Sankagata Manager — Twitch viewer-participation queue + OBS overlay",
    "meta.description":
      "Real-time channel-point queue manager that pushes a transparent overlay to OBS. For Twitch streamers running viewer-participation games.",

    "nav.features": "Features",
    "nav.preview": "Live preview",
    "nav.howto": "How to use",
    "nav.download": "Download",
    "nav.faq": "FAQ",
    "nav.changelog": "Changelog",
    "nav.dl": "Download",

    "hero.eyebrow": "For Twitch viewer-participation streamers",
    "hero.h1":
      'Run your viewer queue<br>without leaving the <span class="accent">stream</span>.',
    "hero.lead":
      '<span class="nobr">Twitch Sankagata Manager</span> listens to channel-point redemptions in real time, sorts viewers into <span class="nobr">Playing / Waiting / Trash</span>, and pushes the queue to OBS as a browser-source overlay — all from one native desktop app.',
    "hero.cta.download": "Download",
    "hero.cta.source": "Source on GitHub",
    "hero.meta.live": "Live · Twitch EventSub",
    "hero.meta.os": "Windows",
    "hero.meta.license": "MIT licensed",
    "hero.meta.notel": "No telemetry",

    "feat.eyebrow": "Core features",
    "feat.title": "From join rewards to OBS overlay, handled in one app.",
    "feat.sub":
      "Catch channel-point redemptions, organize the queue, and update the stream overlay without jumping between tools.",

    "card1.title": "Auto-detect redemptions",
    "card1.body":
      "EventSub picks up channel-point redemptions in real time. Even without a fixed reward selected, you can catch joins by keyword in the reward title. Refunded redemptions are removed from the queue automatically.",
    "card1.meta": "EVENTSUB · KEYWORD MATCH · REFUND-AWARE",
    "card2.title": "Route first-time and repeat joins separately",
    "card2.body":
      "Use different channel-point rewards for first-time viewers and people joining again. You can set only one reward and catch the other side by keywords in the reward name.",
    "card2.meta": "2 REWARDS · EITHER SIDE OPTIONAL · FLEXIBLE ROUTING",
    "card3.title": "Paste into OBS and go",
    "card3.body":
      "Copy the OBS URL and paste it into a Browser Source. The overlay comes up with a transparent background, with no extra server or complicated setup.",
    "card3.meta": "COPY URL · TRANSPARENT BG · NO EXTRA SERVER",
    "card4.title": "App and overlay stay in sync",
    "card4.body":
      "Reordering, trashing, and restoring in the app is reflected in the overlay immediately. Because both views read the same queue state, the stream view does not drift.",
    "card4.meta": "SHARED QUEUE · REAL-TIME PUSH · NO DRIFT",

    "preview.eyebrow": "See it running",
    "preview.title": "App on the side. Overlay on stream.",
    "preview.sub":
      "The desktop app and the OBS browser source share one queue. What you drag, viewers see.",

    "tab.queue": "Queue",
    "tab.rewards": "Rewards",
    "tab.overlay": "Overlay",
    "tab.settings": "Settings",
    "zone.playing": "Playing",
    "zone.waiting": "Waiting",
    "zone.trash": "Trash",

    "howto.eyebrow": "How to use",
    "howto.title": "From setup to OBS in five clear steps.",
    "howto.sub":
      "Sign in, choose rewards, set the visible queue, and paste the OBS URL. No server setup or command-line work.",
    "howto.step1.tag": "Start",
    "howto.step1.title": "Launch the app",
    "howto.step1.body":
      "Open the desktop app after install. First launch takes you straight to the Twitch sign-in flow.",
    "howto.step2.tag": "Auth",
    "howto.step2.title": "Sign in with Twitch",
    "howto.step2.body":
      "Authorize with the device code shown in your browser. No client secret or callback URL setup required.",
    "howto.step3.tag": "Rewards",
    "howto.step3.title": "Choose join rewards",
    "howto.step3.body":
      "Choose rewards for first-time viewers and repeat joins. If you only use one reward, the other side can be detected by keywords in the reward name.",
    "howto.step4.tag": "Queue",
    "howto.step4.title": "Set the visible queue",
    "howto.step4.body":
      "Choose max playing slots, waiting rows shown on stream, and whether first-time-today viewers get priority.",
    "howto.step5.tag": "OBS",
    "howto.step5.title": "Paste the OBS URL",
    "howto.step5.body":
      "Copy the OBS URL into a Browser Source. From there, app changes appear on the overlay immediately.",

    "dl.eyebrow": "Download",
    "dl.title": "Download for Windows or macOS. Source code is public.",
    "dl.signed": "Open source · MIT licensed · no telemetry",
    "dl.runtimeNote": "Pick the build that matches your OS. The macOS .dmg is a universal binary covering both Apple Silicon and Intel.",
    "dl.portable": "Portable .zip · no install",
    "dl.installer": "Or use the .exe installer",
    "dl.comingSoon": "Coming soon",
    "dl.notifyMe": "Notify me",

    "faq.eyebrow": "FAQ",
    "faq.title": "Frequently asked questions",
    "faq.sub":
      "Plain answers about how the app handles trust, your data, and what's coming.",
    "faq.q1.q":
      'Windows shows a "Windows protected your PC" warning when I launch it. Is it safe?',
    "faq.q1.a":
      "Yes — that's Windows SmartScreen, and it warns about any app without an Authenticode certificate (around USD 200/year, heavy for a solo project). Even buying the cert doesn't make the warning disappear immediately — SmartScreen only stops warning after the binary builds up enough install reputation. For a solo project, paying for the cert is often not worth it for that reason. Click 'More info' → 'Run anyway' to launch. The project is also fully open source (MIT) on GitHub, so you can read the source — or build it yourself — to verify the binary is safe.",
    "faq.q2.q": "Which platforms are supported?",
    "faq.q2.a":
      "Windows is the primary target and gets the most testing. macOS (universal — Apple Silicon + Intel) builds are published but have not been thoroughly tested yet; please file an issue if anything breaks on Mac. Linux is not planned.",
    "faq.q3.q": "Where do my Twitch tokens go?",
    "faq.q3.a":
      "Encrypted in your OS keychain — Credential Manager on Windows, Keychain on macOS. Tokens never leave your machine. The app talks to api.twitch.tv directly; there is no third-party server in the loop.",

    "log.eyebrow": "Changelog",
    "log.title": "What changed recently.",
    "log.sub": "Recent fixes and improvements are listed here.",
    "log.loading": "Loading…",
    "log.more": "See all releases →",

    "ft.tag": "Made by streamers, for streamers.",
    "ft.twitchDev": "Twitch dev portal",
    "ft.license": "License (MIT)",
    "ft.report": "Report an issue",
    "ft.bottomTagline": "NO TELEMETRY · NO COOKIES · NO TRACKING",
    "ft.builtWith": "BUILT WITH TAURI · 🐈‍⬛",
  },

  ja: {
    "meta.title":
      "Twitch Sankagata Manager — Twitch 視聴者参加型キュー + OBS オーバーレイ",
    "meta.description":
      "チャンネルポイント引換をリアルタイムに管理して、OBS のブラウザソースに透過オーバーレイで配信。視聴者参加型ゲームを行う Twitch 配信者向け。",

    "nav.features": "機能",
    "nav.preview": "プレビュー",
    "nav.howto": "使い方",
    "nav.download": "ダウンロード",
    "nav.faq": "FAQ",
    "nav.changelog": "更新履歴",
    "nav.dl": "ダウンロード",

    "hero.eyebrow": "視聴者参加型配信を、もっとスムーズに",
    "hero.h1": '視聴者参加型を、<br>もっと <span class="accent">楽に</span>。',
    "hero.lead":
      '<span class="nobr">Twitch Sankagata Manager</span> は、チャンネルポイント引換をリアルタイムに検知し、<span class="nobr">Playing / Waiting / Trash</span> に自動で振り分けます。そのまま OBS のブラウザソース用オーバーレイとして配信に流せて、ネイティブアプリ一本で完結。',
    "hero.cta.download": "ダウンロード",
    "hero.cta.source": "GitHub のソース",
    "hero.meta.live": "Live · Twitch EventSub",
    "hero.meta.os": "Windows",
    "hero.meta.license": "MIT ライセンス",
    "hero.meta.notel": "テレメトリなし",

    "feat.eyebrow": "主な機能",
    "feat.title": "参加受付から OBS 表示まで、ひとつのアプリで。",
    "feat.sub":
      "チャンネルポイントの引換を拾い、キューを整理し、OBS のオーバーレイへすぐ反映します。配信中にいくつもの画面を行き来する手間を減らせます。",

    "card1.title": "リワード引換を自動検知",
    "card1.body":
      "EventSub でチャンネルポイント引換をリアルタイムに検知します。対象リワードを固定しなくても、タイトルのキーワードで拾えます。返金された引換も自動でキューから外れます。",
    "card1.meta": "EVENTSUB · キーワード検出 · 返金対応",
    "card2.title": "初回と2回目以降を分けて受付",
    "card2.body":
      "初めて参加する人用と、2回目以降の人用で、使うリワードを分けられます。片方だけリワードを選び、もう片方はリワード名のキーワードで拾うこともできます。",
    "card2.meta": "2 リワード対応 · 片側設定も可 · 運用に合わせる",
    "card3.title": "OBS に貼るだけで使える",
    "card3.body":
      "OBS URL をコピーして、ブラウザソースに貼るだけ。透過背景のオーバーレイをすぐ出せるので、別サーバーや複雑な準備は要りません。",
    "card3.meta": "URL を貼るだけ · 透過背景 · 追加準備なし",
    "card4.title": "管理画面とオーバーレイがズレない",
    "card4.body":
      "管理画面で並び替えた順番や、ゴミ箱への移動と復元まで、そのままオーバーレイに反映されます。同じキューを見ているので、配信画面だけ古い表示になる心配がありません。",
    "card4.meta": "同じキューを共有 · リアルタイム反映 · ズレなし",

    "preview.eyebrow": "動作プレビュー",
    "preview.title": "管理した内容が、そのまま配信画面へ。",
    "preview.sub":
      "参加者の並び替えやゴミ箱への移動は、OBS のオーバーレイにすぐ反映されます。配信中に別画面を調整し直す必要はありません。",

    "tab.queue": "キュー",
    "tab.rewards": "報酬",
    "tab.overlay": "オーバーレイ",
    "tab.settings": "設定",
    "zone.playing": "プレイ中",
    "zone.waiting": "待機",
    "zone.trash": "ゴミ箱",

    "howto.eyebrow": "使い方",
    "howto.title": "OBS に表示するまでの流れ",
    "howto.sub":
      "Twitch ログイン、参加用リワードの設定、OBS への URL 追加まで。サーバーの準備やコマンド操作は不要です。",
    "howto.step1.tag": "起動",
    "howto.step1.title": "アプリを起動",
    "howto.step1.body":
      "インストールしたデスクトップアプリを開きます。初回起動時は、そのまま Twitch のログイン手順に進みます。",
    "howto.step2.tag": "認証",
    "howto.step2.title": "Twitch でログイン",
    "howto.step2.body":
      "ブラウザに表示されたコードで認証します。クライアントシークレットやコールバック URL の設定は必要ありません。",
    "howto.step3.tag": "リワード",
    "howto.step3.title": "参加用リワードを選ぶ",
    "howto.step3.body":
      "初めて参加する人用と、2回目以降の人用で、使うリワードを選びます。片方だけリワードを選び、もう片方はリワード名のキーワードで拾うこともできます。",
    "howto.step4.tag": "表示",
    "howto.step4.title": "表示人数を決める",
    "howto.step4.body":
      "プレイ人数と、オーバーレイに表示する待機人数を設定します。本日初参加者を優先する設定もここで選べます。",
    "howto.step5.tag": "OBS",
    "howto.step5.title": "OBS に URL を貼る",
    "howto.step5.body":
      "OBS URL をコピーし、ブラウザソースに貼り付けます。以降は管理画面の変更がオーバーレイにすぐ反映されます。",

    "dl.eyebrow": "ダウンロード",
    "dl.title": "Windows / macOS 版をダウンロード。ソースコードも公開しています。",
    "dl.signed": "オープンソース · MIT ライセンス · テレメトリなし",
    "dl.runtimeNote": "お使いの OS に合わせて選んでください。macOS 版は Apple Silicon と Intel 両対応のユニバーサルバイナリです。",
    "dl.portable": "ポータブル版 (.zip) · インストール不要",
    "dl.installer": "またはインストーラー (.exe) を使う",
    "dl.comingSoon": "近日公開",
    "dl.notifyMe": "通知を受け取る",

    "faq.eyebrow": "FAQ",
    "faq.title": "よくある質問",
    "faq.sub": "セキュリティやプライバシー、今後の予定について、よくある疑問にお答えします。",
    "faq.q1.q":
      "Windows で起動すると「PC を保護しました」と警告が出ます。安全ですか？",
    "faq.q1.a":
      "はい、安全です。表示されているのは Windows SmartScreen の警告で、Authenticode 証明書（年 200 USD 前後）が付いていない個人プロジェクト全般に出るものです。なお、証明書を購入しても警告がすぐに消えるわけではなく、十分なインストール数によって「評判」が積み上がってはじめて表示されなくなる仕組みです。そのため個人開発では、証明書を購入しても費用に見合った効果が出にくいのが現状です。「詳細情報」→「実行」で進めてください。本プロジェクトはオープンソース（MIT）として GitHub に公開されているので、ソースコードを直接確認したり、自分でビルドして安全性を検証することも可能です。",
    "faq.q2.q": "対応プラットフォームは？",
    "faq.q2.a":
      "Windows をメインに開発しており、テストも Windows でいちばん行っています。macOS 版（Apple Silicon と Intel 両対応のユニバーサルビルド）も配布していますが、まだ十分に検証できていません。Mac で不具合があれば Issue でお知らせください。Linux 版は予定していません。",
    "faq.q3.q": "Twitch のトークンはどこに保存されますか？",
    "faq.q3.a":
      "OS のキーチェーンに暗号化して保存されます（Windows は資格情報マネージャー、macOS は Keychain）。トークンがマシンの外に出ることはありません。アプリは api.twitch.tv に直接アクセスし、第三者のサーバーは経由しません。",

    "log.eyebrow": "更新履歴",
    "log.title": "最近のアップデート。",
    "log.sub": "最近の修正や改善内容を確認できます。",
    "log.loading": "読み込み中…",
    "log.more": "すべてのリリースを見る →",

    "ft.tag": "配信者がつくった、配信者のためのツール。",
    "ft.twitchDev": "Twitch 開発者ポータル",
    "ft.license": "ライセンス (MIT)",
    "ft.report": "不具合を報告",
    "ft.bottomTagline": "テレメトリなし · クッキーなし · トラッキングなし",
    "ft.builtWith": "TAURI で構築 · 🐈‍⬛",
  },

  ko: {
    "meta.title":
      "Twitch Sankagata Manager — Twitch 시청자 참가형 대기열 + OBS 오버레이",
    "meta.description":
      "채널 포인트 사용을 실시간으로 관리하고 OBS 브라우저 소스에 투명 오버레이로 송출. 시청자 참가형 게임을 진행하는 Twitch 스트리머를 위한 도구.",

    "nav.features": "기능",
    "nav.preview": "라이브 미리보기",
    "nav.howto": "사용법",
    "nav.download": "다운로드",
    "nav.faq": "FAQ",
    "nav.changelog": "업데이트",
    "nav.dl": "다운로드",

    "hero.eyebrow": "Twitch 시청자 참가형 스트리머용",
    "hero.h1": '시청자 참가형을,<br>더 <span class="accent">가볍게</span>.',
    "hero.lead":
      '<span class="nobr">Twitch Sankagata Manager</span> 는 채널 포인트 사용을 실시간으로 감지하고, <span class="nobr">Playing / Waiting / Trash</span> 로 자동 분류합니다. 그대로 OBS 브라우저 소스용 오버레이로 송출 — 네이티브 데스크톱 앱 하나로 끝.',
    "hero.cta.download": "다운로드",
    "hero.cta.source": "GitHub 소스",
    "hero.meta.live": "Live · Twitch EventSub",
    "hero.meta.os": "Windows",
    "hero.meta.license": "MIT 라이선스",
    "hero.meta.notel": "추적 없음",

    "feat.eyebrow": "주요 기능",
    "feat.title": "참가 접수부터 OBS 표시까지, 하나의 앱에서.",
    "feat.sub":
      "채널 포인트 보상을 감지하고, 대기열을 정리하고, OBS 오버레이에 바로 반영합니다. 방송 중 여러 화면을 오가는 번거로움을 줄일 수 있습니다.",

    "card1.title": "리워드 사용 자동 감지",
    "card1.body":
      "EventSub 가 채널 포인트 사용을 실시간으로 감지합니다. 대상 리워드를 고정하지 않아도, 리워드 제목의 키워드로 참가 요청을 받을 수 있습니다. 환불된 사용도 대기열에서 자동으로 빠집니다.",
    "card1.meta": "EVENTSUB · 키워드 감지 · 환불 대응",
    "card2.title": "첫 참가와 2회차 이후를 따로 접수",
    "card2.body":
      "처음 참가하는 사람과 다시 참가하는 사람에게 서로 다른 리워드를 쓸 수 있습니다. 한쪽만 리워드를 선택하고, 다른 쪽은 리워드 이름의 키워드로 받을 수도 있습니다.",
    "card2.meta": "2개 리워드 · 한쪽만 설정 가능 · 운영 방식에 맞춤",
    "card3.title": "OBS 에 붙여 넣기만 하면 끝",
    "card3.body":
      "OBS URL 을 복사해서 브라우저 소스에 붙여 넣으면 바로 쓸 수 있습니다. 투명 배경 오버레이가 뜨고, 별도 서버나 복잡한 준비는 필요 없습니다.",
    "card3.meta": "URL 붙여 넣기 · 투명 배경 · 추가 준비 없음",
    "card4.title": "관리 화면과 오버레이가 어긋나지 않음",
    "card4.body":
      "앱에서 순서를 바꾸거나 휴지통으로 보내고 복원한 내용이 오버레이에 바로 반영됩니다. 같은 대기열 상태를 보기 때문에 방송 화면만 따로 늦게 남지 않습니다.",
    "card4.meta": "같은 대기열 공유 · 실시간 반영 · 어긋남 없음",

    "preview.eyebrow": "실제 동작",
    "preview.title": "앱은 옆에. 오버레이는 방송에.",
    "preview.sub":
      "데스크톱 앱과 OBS 브라우저 소스는 하나의 대기열을 공유합니다. 당신이 옮긴 그대로 시청자에게 전달됩니다.",

    "tab.queue": "대기열",
    "tab.rewards": "리워드",
    "tab.overlay": "오버레이",
    "tab.settings": "설정",
    "zone.playing": "플레이 중",
    "zone.waiting": "대기",
    "zone.trash": "휴지통",

    "howto.eyebrow": "사용법",
    "howto.title": "방송 화면에 띄우기까지의 흐름",
    "howto.sub":
      "Twitch 에 로그인하고, 참가용 리워드를 고른 뒤, OBS 에 URL 을 붙여 넣으면 됩니다. 서버 준비나 명령어 작업은 필요 없습니다.",
    "howto.step1.tag": "시작",
    "howto.step1.title": "앱 실행",
    "howto.step1.body":
      "설치한 데스크톱 앱을 엽니다. 첫 실행 시 Twitch 로그인 흐름으로 바로 이어집니다.",
    "howto.step2.tag": "인증",
    "howto.step2.title": "Twitch 로그인",
    "howto.step2.body":
      "브라우저에 표시된 코드로 인증합니다. 클라이언트 시크릿이나 콜백 URL 설정은 필요 없습니다.",
    "howto.step3.tag": "리워드",
    "howto.step3.title": "참가용 리워드 선택",
    "howto.step3.body":
      "처음 참가하는 사람용과 2회차 이후 참가자용 리워드를 선택합니다. 한쪽만 리워드를 선택하고, 다른 쪽은 리워드 이름의 키워드로 받을 수도 있습니다.",
    "howto.step4.tag": "표시",
    "howto.step4.title": "표시 인원 설정",
    "howto.step4.body":
      "플레이 인원과 오버레이에 보일 대기 인원을 정합니다. 오늘 첫 참가자를 우선할지도 여기서 선택할 수 있습니다.",
    "howto.step5.tag": "OBS",
    "howto.step5.title": "OBS 에 URL 붙여 넣기",
    "howto.step5.body":
      "OBS URL 을 복사해 브라우저 소스에 붙여 넣습니다. 이후 관리 화면의 변경이 오버레이에 바로 반영됩니다.",

    "dl.eyebrow": "다운로드",
    "dl.title": "Windows / macOS 버전 다운로드. 소스 코드도 공개되어 있습니다.",
    "dl.signed": "오픈소스 · MIT 라이선스 · 추적 없음",
    "dl.runtimeNote": "사용 중인 OS 에 맞춰 선택하세요. macOS 빌드는 Apple Silicon 과 Intel 모두를 지원하는 유니버설 바이너리입니다.",
    "dl.portable": "포터블 (.zip) · 설치 불필요",
    "dl.installer": "또는 인스톨러 (.exe) 사용",
    "dl.comingSoon": "공개 예정",
    "dl.notifyMe": "알림 받기",

    "faq.eyebrow": "FAQ",
    "faq.title": "자주 묻는 질문",
    "faq.sub": "신뢰, 데이터 처리, 앞으로의 계획에 대한 솔직한 답변.",
    "faq.q1.q":
      "Windows 에서 실행하면 ‘PC 보호’ 경고가 뜹니다. 안전한가요?",
    "faq.q1.a":
      "네, 안전합니다. 표시되는 건 Windows SmartScreen 경고로, Authenticode 인증서(연 200 USD 정도)가 없는 개인 프로젝트에는 일반적으로 뜹니다. 다만 인증서를 구매하더라도 SmartScreen 경고가 곧바로 사라지지는 않으며, 충분한 설치 수로 ‘평판’이 쌓여야 비로소 표시되지 않습니다. 그래서 개인 프로젝트에서는 인증서 비용 대비 효과가 크지 않습니다. ‘추가 정보’ → ‘실행’을 눌러 진행하세요. 또한 본 프로젝트는 오픈소스(MIT)로 GitHub 에 공개되어 있어, 소스 코드를 직접 확인하거나 직접 빌드해 바이너리의 안전성을 검증할 수도 있습니다.",
    "faq.q2.q": "지원 플랫폼은 어떻게 되나요?",
    "faq.q2.a":
      "Windows 를 메인으로 개발하고 있으며 테스트도 Windows 에서 가장 많이 진행됩니다. macOS 빌드(Apple Silicon · Intel 모두 지원하는 유니버설 버전)도 배포 중이지만, 아직 충분히 검증되지 않았습니다. Mac 에서 문제가 있으면 Issue 로 알려 주세요. Linux 버전은 예정에 없습니다.",
    "faq.q3.q": "Twitch 토큰은 어디에 저장되나요?",
    "faq.q3.a":
      "OS 의 키체인에 암호화되어 저장됩니다 (Windows 는 자격 증명 관리자, macOS 는 Keychain). 토큰이 컴퓨터 밖으로 나가지 않습니다. 앱은 api.twitch.tv 에 직접 연결하고, 중간에 제 3 자 서버는 거치지 않습니다.",

    "log.eyebrow": "업데이트",
    "log.title": "최근 업데이트 내역",
    "log.sub": "최근 수정 사항과 개선 내용을 확인할 수 있습니다.",
    "log.loading": "불러오는 중…",
    "log.more": "모든 릴리스 보기 →",

    "ft.tag": "스트리머가 만든, 스트리머를 위한 도구.",
    "ft.twitchDev": "Twitch 개발자 포털",
    "ft.license": "라이선스 (MIT)",
    "ft.report": "문제 신고",
    "ft.bottomTagline": "추적 없음 · 쿠키 없음 · 분석 없음",
    "ft.builtWith": "TAURI 로 제작 · 🐈‍⬛",
  },
};

export const LANG_META = {
  en: { hreflang: "en", ogLocale: "en_US", pillLabel: "EN" },
  ja: { hreflang: "ja", ogLocale: "ja_JP", pillLabel: "JP" },
  ko: { hreflang: "ko", ogLocale: "ko_KR", pillLabel: "KO" },
};
