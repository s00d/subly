import type { SiteContent } from "./types";
import { buildLocaleSwitcherOptions } from "./locales";

const sharedUrls = {
  repoUrl: "https://github.com/s00d/subly",
  releasesUrl: "https://github.com/s00d/subly/releases/latest",
  privacyUrl: "https://s00d.github.io/subly/privacy",
  changelogUrl: "https://s00d.github.io/subly/changelog",
  appStoreUrl: "https://apps.apple.com/app/subly-money-tracker/id6767137924",
  donateUrl: "https://www.donationalerts.com/r/s00d88",
  videoDesktopUrl:
    "https://github.com/s00d/subly/raw/main/store-assets/video/store-walkthrough-desktop-with-audio.mp4",
  videoMobileUrl:
    "https://github.com/s00d/subly/raw/main/store-assets/video/store-walkthrough-mobile-with-audio.mp4",
  ogImage: "https://s00d.github.io/subly/screenshots/01_dashboard_overview.png",
} as const;

export const siteMeta = {
  title: "Subly — Subscription & Expense Tracker",
  description:
    "Open-source subscription and expense tracker. Local-first, no analytics or tracking, optional cloud sync.",
  siteUrl: "https://s00d.github.io/subly/",
  canonicalUrl: "https://s00d.github.io/subly/",
  locale: "en_US",
  softwareVersion: "1.0.4",
  applicationCategory: "FinanceApplication",
  operatingSystems: ["macOS", "Windows", "Linux", "iOS", "Android"],
  ...sharedUrls,
} as const;

export const localeSwitcher = buildLocaleSwitcherOptions("Language");

export const headerNav = {
  download: "Download",
  menuToggle: "Toggle menu",
  moreLabel: "More",
  moreMenuAria: "More sections",
} as const;

export const navPrimaryLinks = [
  { id: "why", label: "Why Subly", icon: "heart-outline" },
  { id: "subscriptions", label: "Subscriptions", icon: "repeat" },
  { id: "dashboard", label: "Dashboard", icon: "view-dashboard-outline" },
  { id: "video", label: "Demo", icon: "play-circle-outline" },
] as const;

export const navMoreLinks = [
  { id: "ai", label: "AI", icon: "robot-outline" },
  { id: "sync", label: "Sync", icon: "cloud-sync-outline" },
  { id: "faq", label: "FAQ", icon: "help-circle-outline" },
] as const;

export const navLinks = [...navPrimaryLinks, ...navMoreLinks] as const;

export const hero = {
  headline: "Know what you pay — before it renews.",
  subline:
    "A fast subscription and expense tracker that stays on your device, ships as fully open MIT source, and never phones home with analytics or usage metrics.",
  ctaPrimary: "Download latest release",
  ctaSecondary: "View on GitHub",
  watchDemo: "Watch demo",
} as const;

export const whySection = {
  title: "Why people choose Subly",
  subtitle:
    "Clarity for recurring bills and everyday spending — open code, no tracking, and tools that respect your data.",
} as const;

export const videoSection = {
  eyebrow: "Product demo",
  title: "See Subly in action",
  subtitle: "Switch between desktop and phone recordings.",
  fallbackLabel: "Open video in a new tab if playback does not start",
  demos: [
    {
      id: "desktop",
      label: "Desktop",
      icon: "microsoft-windows",
      description:
        "Subscriptions, expenses, dashboard, and settings on macOS.",
      url: siteMeta.videoDesktopUrl,
      poster: "01_dashboard_overview.png",
      aspect: "video" as const,
    },
    {
      id: "phone",
      label: "Phone",
      icon: "cellphone",
      description:
        "The same flows on iPhone — lists, calendar, and settings.",
      url: siteMeta.videoMobileUrl,
      poster: "phone/01_dashboard_overview.png",
      aspect: "portrait" as const,
    },
  ],
} as const;

export const platforms = {
  title: "Available everywhere you work",
  items: [
    { name: "macOS", icon: "apple" },
    { name: "Windows", icon: "microsoft-windows" },
    { name: "Linux", icon: "linux" },
    { name: "iOS", icon: "apple-ios" },
    { name: "Android", icon: "android" },
  ] as const,
  tagline: "Local-first, fully open source, and free of analytics — no forced cloud account.",
} as const;

export const openValuesSection = {
  eyebrow: "Open & honest",
  icon: "github",
  title: "Powerful app. Open code. Zero surveillance.",
  lead:
    "Most finance apps want your data on their servers — and a quiet stream of analytics on the side. Subly is the opposite: MIT-licensed source you can read and build yourself, local-first by default, and no tracking stack of any kind. Not “privacy mode.” Not “we aggregate anonymously.” Just no metrics, no ads, no hidden pixels.",
  pillars: [
    {
      icon: "github",
      title: "Fully open source",
      body: "Desktop, mobile, sync, and optional AI — the whole product is on GitHub under the MIT license. Audit it, fork it, ship your own build, or contribute a fix. Transparency is a feature.",
    },
    {
      icon: "eye-off-outline",
      title: "No analytics. Ever.",
      body: "No Google Analytics, Firebase, Mixpanel, Amplitude, Sentry session replay, or ad SDKs. Subly does not collect usage statistics or crash telemetry to our servers. How you use the app is your business alone.",
    },
    {
      icon: "shield-check-outline",
      title: "Trust you can verify",
      body: "Data lives on your device. Sync uses your cloud if you turn it on. API keys sit in the OS keychain. When the README says “no tracking,” you can grep the repo and see it.",
    },
  ],
  productHighlights: [
    "Subscriptions, expenses, dashboard, and renewal calendar in one place",
    "34+ currencies, 12 languages, CSV/JSON exports",
    "Optional AI assist — four capabilities, seven providers, off until you enable it",
    "Works offline; no Subly account required",
  ],
  ctaLabel: "Explore the source on GitHub",
} as const;

export const whyCards = [
  {
    icon: "github",
    title: "Open source, MIT license",
    body: "Every feature is in a public repository. No black box around your finances — inspect, build, and share on your terms.",
  },
  {
    icon: "eye-off-outline",
    title: "No metrics or tracking",
    body: "We do not measure screens, clicks, or crashes for ourselves. Your spending patterns never become someone else's dashboard.",
  },
  {
    icon: "cellphone-link-off",
    title: "Your data, your device",
    body: "Subscriptions, expenses, and settings live on your machine or phone. You stay in control without signing up for our servers.",
  },
  {
    icon: "cloud-sync-outline",
    title: "Sync when you choose",
    body: "Optional backup through iCloud, Google Drive, Dropbox, OneDrive, or WebDAV — only if you enable it in Settings.",
  },
  {
    icon: "translate",
    title: "12 languages",
    body: "English, Russian, Chinese, Spanish, French, German, Portuguese, Japanese, Korean, Arabic, Hindi, and Turkish.",
  },
  {
    icon: "view-dashboard-outline",
    title: "Built for real life",
    body: "Dashboards, a renewal calendar, multi-currency support, reminders, and exports — everything in one calm interface.",
  },
] as const;

export const featureSections = [
  {
    id: "subscriptions",
    eyebrow: "Subscriptions",
    icon: "repeat",
    title: "Every recurring charge in one place",
    body: "Add streaming, software, utilities, or any service. Use daily, weekly, monthly, or yearly billing — plus custom cycles where supported.",
    bullets: [
      "Pin favorites and switch between compact or expanded lists",
      "Group by category or payment method; batch actions for cleanup",
      "Full payment history, auto-renewal flags, overdue tracking, notes and links",
    ],
    screenshot: "02_subscriptions_list.png",
    alt: "Subly subscriptions list",
  },
  {
    id: "expenses",
    eyebrow: "Expenses",
    icon: "cash-multiple",
    title: "One-off spending without the spreadsheet",
    body: "Record purchases with amounts, categories, tags, notes, and optional links. Filter by period, category, or tags when you need the full picture.",
    bullets: [
      "Export expenses to CSV for spreadsheets",
      "Link expenses to subscriptions when it fits your workflow",
      "Same categories, tags, and payment methods as subscriptions",
    ],
    screenshot: "03_expenses_list.png",
    alt: "Subly expenses list",
  },
  {
    id: "dashboard",
    eyebrow: "Dashboard",
    icon: "view-dashboard-outline",
    title: "See the big picture at a glance",
    body: "Pick which blocks appear and how they are ordered. From overdue alerts to forecasts and lifetime subscription cost — your home screen adapts to you.",
    bullets: [
      "Overdue and upcoming payments, stats, monthly budget progress",
      "Spending trends, forecasts, lifetime costs, category averages",
      "Charts for breakdowns, top expenses, weekday patterns, tag views, and rate history",
    ],
    screenshot: "01_dashboard_overview.png",
    alt: "Subly dashboard overview",
  },
  {
    id: "calendar",
    eyebrow: "Calendar",
    icon: "calendar-month-outline",
    title: "Plan around paydays and bills",
    body: "Upcoming charges appear on a calendar grid. Review monthly totals and jump from a date straight into the related subscription.",
    bullets: [
      "Monthly payment count and total amount",
      "Clear view of what hits before your next paycheck",
      "Tap through to subscription details in one step",
    ],
    screenshot: "04_calendar_month.png",
    alt: "Subly calendar month view",
  },
  {
    id: "currencies",
    eyebrow: "Money & alerts",
    icon: "currency-usd",
    title: "Currencies and reminders that keep up",
    body: "Track spending in many currencies, refresh exchange rates on a schedule, and get notified before money leaves your account.",
    bullets: [
      "34+ currencies with room for custom entries",
      "Automatic rate updates from your chosen provider",
      "System notifications for upcoming and overdue payments",
      "Optional Telegram reminders; customizable titles and schedule",
    ],
    screenshot: "05_rates_screen.png",
    alt: "Subly currency rates screen",
  },
] as const;

export const dashboardWidgets = [
  "Overdue & upcoming payments",
  "Stats & budget progress",
  "Spending trends & forecasts",
  "Lifetime subscription cost",
  "Category breakdown charts",
  "Top expenses & month comparison",
  "Expenses by tags or weekday",
] as const;

export const aiSection = {
  eyebrow: "AI assist",
  icon: "robot-outline",
  title: "Optional AI — four ways to add data faster",
  capabilitiesTitle: "Capabilities",
  capabilitiesIntro:
    "One assist entry point on subscriptions and expenses — enable each feature separately in Settings.",
  inputMethodsTitle: "How you can send input",
  lead:
    "Subly’s AI layer is entirely opt-in: disabled until you flip the master switch in Settings. You choose the provider, store your API key in the OS keychain, and turn on only the capabilities you want. Nothing is sent to a model until you open the assist dialog and submit.",
  principles: [
    {
      icon: "toggle-switch-off-outline",
      title: "Off by default",
      body: "No background calls, no silent uploads. Enable AI only when you need it.",
    },
    {
      icon: "key",
      title: "Bring your own key",
      body: "Paid cloud APIs or a local Ollama / LM Studio endpoint — credentials never sit in the database.",
    },
    {
      icon: "clipboard-check-outline",
      title: "Preview before save",
      body: "Every subscription, expense, or imported row is shown for review. Nothing lands in your ledger without confirmation.",
    },
  ],
  features: [
    {
      icon: "repeat",
      title: "Quick-add subscriptions",
      body: "Describe a service in plain language — price, cycle, category, payment method, and tags are prefilled using your existing catalog.",
      example: "“Telegram Premium 300 ₽ monthly, card”",
    },
    {
      icon: "cash-multiple",
      title: "Quick-add expenses",
      body: "Same dialog for one-off spending: amount, date, merchant, categories, and tags parsed from a short sentence.",
      example: "“Coffee at Starbucks 5.40 EUR yesterday”",
    },
    {
      icon: "receipt",
      title: "Receipt & document scan",
      body: "Point a vision-capable model at a photo or PDF. Line items, totals, currency, merchant, and date come back as a draft expense.",
      example: "PNG, JPEG, WebP, HEIC, or PDF receipts",
    },
    {
      icon: "file-document-multiple-outline",
      title: "Bank statement import",
      body: "Bulk-import CSV, XLSX, PDF, or JSON statements. Heuristic parsing plus AI fallback, with a per-row table before saving.",
      example: "Card and bank exports in common formats",
    },
  ],
  inputMethods: [
    { icon: "keyboard-outline", label: "Type a description", detail: "⌘/Ctrl + Enter to send" },
    { icon: "image-plus-outline", label: "Drop or pick files", detail: "Drag-and-drop on desktop" },
    { icon: "content-paste", label: "Paste screenshots", detail: "Into the text field" },
  ],
  providersTitle: "Supported providers",
  providers: [
    "OpenAI",
    "OpenRouter",
    "Google Gemini",
    "Groq",
    "DeepSeek",
    "Mistral",
    "OpenAI-compatible (Ollama, LM Studio, vLLM)",
  ],
  settingsBullets: [
    "Master toggle plus individual switches for each capability",
    "Recommended model presets and custom model name",
    "Test connection before you rely on it",
    "Vision badge when the model supports receipt OCR",
  ],
  privacyNote:
    "Requests go only to the provider you configured. Subly does not operate its own LLM or log your prompts on our servers.",
  screenshot: "06_settings_overview.png",
  alt: "Subly AI settings panel",
} as const;

export const syncSection = {
  eyebrow: "Sync & backup",
  icon: "backup-restore",
  title: "Your backup, your cloud",
  body: "When you need multiple devices, enable the provider you already use. Export and import full backups anytime.",
  providers: [
    { id: "icloud", label: "iCloud", icon: "apple-icloud", brandIcon: null },
    { id: "google-drive", label: "Google Drive", icon: "google-drive", brandIcon: "google-drive.svg" },
    { id: "dropbox", label: "Dropbox", icon: "dropbox", brandIcon: "dropbox.svg" },
    { id: "onedrive", label: "OneDrive", icon: "microsoft-onedrive", brandIcon: "onedrive.svg" },
    { id: "webdav", label: "WebDAV", icon: "web", brandIcon: null },
  ],
  bullets: [
    "Encrypted sync payloads between your devices",
    "Export or import `.subly` backups, plus JSON and CSV",
    "Household members, categories, and tags travel with your data",
  ],
} as const;

export const privacySection = {
  eyebrow: "Privacy",
  icon: "shield-lock-outline",
  title: "Private by default",
  body: "No analytics, no ad SDKs, no “anonymous usage” pipelines — and an open codebase so you do not have to take our word for it.",
  bullets: [
    "Core data stored locally on each device",
    "No in-app metrics, crash reporters, or third-party tracking to Subly servers",
    "API keys, sync passwords, and Telegram tokens live in the OS keychain — not in plain text in the database",
    "AI and cloud features are opt-in; read our Privacy Policy for full details",
  ],
  policyLinkLabel: "Privacy Policy",
} as const;

export const customizeSection = {
  eyebrow: "Make it yours",
  icon: "palette-outline",
  title: "Themes, tags, and household",
  body: "Fifteen-plus color themes, custom category icons, payment methods, tags, monthly budget progress, and system tray shortcuts on desktop.",
  bullets: [
    "Household members to attribute spending correctly",
    "System tray with upcoming payments and quick open",
    "Minimize to tray, autostart on boot (desktop)",
  ],
  screenshot: "06_settings_overview.png",
  alt: "Subly appearance and settings",
} as const;

export const deviceGalleryMeta = {
  title: "Inside the app",
  subtitle: "The same core flows on desktop, phone, and tablet.",
} as const;

const desktopShots = [
  { file: "01_dashboard_overview.png", label: "Dashboard" },
  { file: "02_subscriptions_list.png", label: "Subscriptions" },
  { file: "03_expenses_list.png", label: "Expenses" },
  { file: "04_calendar_month.png", label: "Calendar" },
  { file: "05_rates_screen.png", label: "Currencies" },
  { file: "06_settings_overview.png", label: "Settings" },
] as const;

const mobileShots = [
  { file: "phone/01_dashboard_overview.png", label: "Dashboard" },
  { file: "phone/02_subscriptions_list.png", label: "Subscriptions" },
  { file: "phone/03_expenses_list.png", label: "Expenses" },
  { file: "phone/04_calendar_month.png", label: "Calendar" },
  { file: "phone/05_rates_screen.png", label: "Currencies" },
  { file: "phone/06_settings_overview.png", label: "Settings" },
] as const;

const tabletShots = [
  { file: "tablet/01_dashboard_overview.png", label: "Dashboard" },
  { file: "tablet/02_subscriptions_list.png", label: "Subscriptions" },
  { file: "tablet/03_expenses_list.png", label: "Expenses" },
  { file: "tablet/04_calendar_month.png", label: "Calendar" },
  { file: "tablet/05_rates_screen.png", label: "Currencies" },
  { file: "tablet/06_settings_overview.png", label: "Settings" },
] as const;

export const deviceGalleries = [
  {
    id: "desktop",
    label: "macOS",
    icon: "apple",
    description:
      "Full desktop experience with dashboard widgets, system tray shortcuts, and exports — built for macOS, Windows, and Linux.",
    shots: desktopShots,
  },
  {
    id: "phone",
    label: "iPhone",
    icon: "cellphone",
    description:
      "Track subscriptions and expenses on the go with the same categories, tags, and renewal calendar.",
    shots: mobileShots,
  },
  {
    id: "tablet",
    label: "Tablet",
    icon: "tablet",
    description:
      "More room for lists and the monthly calendar on iPad and Android tablets without losing local-first privacy.",
    shots: tabletShots,
  },
] as const;

export const appStore = {
  url: siteMeta.appStoreUrl,
  title: "Also on the App Store",
  body: "Subly is available on Mac and iPhone/iPad through the App Store. It is a paid download — your purchase helps support ongoing development. Desktop builds for macOS, Windows, and Linux remain free on GitHub Releases.",
  platforms: ["macOS", "iOS", "iPadOS"],
  ctaLabel: "View on the App Store",
} as const;

export const faqSection = {
  title: "FAQ",
} as const;

export const faqItems = [
  {
    q: "Is Subly free?",
    a: "Subly is open source under the MIT license. Download builds from GitHub Releases at no cost. Store listings may apply platform fees where applicable.",
  },
  {
    q: "Does it work offline?",
    a: "Yes. Subscriptions, expenses, dashboard, and calendar work fully offline. Sync, rate updates, AI, and Telegram need network when you use those features.",
  },
  {
    q: "Do I need AI?",
    a: "No. AI is optional and off by default. When enabled, you can quick-add subscriptions or expenses, scan receipts, or import statements — each capability has its own toggle, and you pick from seven supported providers with your own API key.",
  },
  {
    q: "Is cloud sync safe?",
    a: "Sync uses your own cloud account (iCloud, Drive, Dropbox, OneDrive, or WebDAV). Credentials are stored in the system keychain. Enable sync only if you want multi-device backup.",
  },
  {
    q: "Which platforms are supported?",
    a: "macOS (Universal), Windows, Linux, iOS, and Android. See GitHub Releases for desktop installers; mobile builds follow store distribution where published.",
  },
  {
    q: "How do I back up my data?",
    a: "Use Settings → Data to export a `.subly` file, JSON, or CSV. Import the same formats when moving to a new device.",
  },
  {
    q: "Where is my data stored?",
    a: "On your device by default. We do not run a central Subly cloud that holds your subscriptions.",
  },
  {
    q: "Does Subly use analytics or tracking?",
    a: "No. The app includes no analytics SDKs, ad networks, or usage metrics sent to us. Optional features (sync, AI, Telegram, rate updates) only talk to services you configure.",
  },
  {
    q: "Can I read the source code?",
    a: "Yes. Subly is fully open source on GitHub under the MIT license — frontend, backend, and mobile builds. You can verify privacy claims yourself.",
  },
  {
    q: "Can I try it before installing?",
    a: "Watch the desktop or phone demo videos on this page, or browse the screenshot gallery below.",
  },
] as const;

export const downloadSection = {
  title: "Ready to take control?",
  body: "Free desktop builds from GitHub Releases — pick your platform below. Prefer the App Store on Apple devices? That option supports development.",
  ctaPrimary: "Download latest release",
  githubHeading: "Free on GitHub",
  ctaSecondary: "Star on GitHub",
  ctaDonate: "Support development",
  ctaVideo: "Watch demo",
  changelogLabel: "Changelog",
} as const;

export const pageTitles = {
  privacy: "Privacy Policy — Subly",
  changelog: "Changelog — Subly",
} as const;

export const footer = {
  tagline: "Subly — subscription & expense tracker",
  license: "MIT License",
  donateLabel: "Donate",
  links: [
    { label: "GitHub", href: siteMeta.repoUrl, icon: "github", external: true },
    { label: "Releases", href: siteMeta.releasesUrl, icon: "download", external: true },
    { label: "Privacy", to: "/privacy", icon: "shield-check-outline", external: false },
    { label: "Changelog", to: "/changelog", icon: "history", external: false },
  ],
} as const;

export const enContent: SiteContent = {
  siteMeta,
  localeSwitcher,
  headerNav,
  navPrimaryLinks,
  navMoreLinks,
  navLinks,
  hero,
  videoSection,
  platforms,
  openValuesSection,
  whySection,
  whyCards,
  featureSections,
  dashboardWidgets,
  aiSection,
  syncSection,
  privacySection,
  customizeSection,
  deviceGalleryMeta,
  deviceGalleries,
  appStore,
  faqSection,
  faqItems,
  downloadSection,
  footer,
  pageTitles,
};
