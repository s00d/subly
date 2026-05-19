export const siteMeta = {
  title: "Subly — Subscription & Expense Tracker",
  description:
    "Track subscriptions and everyday spending in one fast, private app. Local-first with optional cloud sync.",
  repoUrl: "https://github.com/s00d/subly",
  releasesUrl: "https://github.com/s00d/subly/releases/latest",
  privacyUrl: "https://github.com/s00d/subly/blob/main/PRIVACY.md",
  changelogUrl: "https://github.com/s00d/subly/blob/main/CHANGELOG.md",
  donateUrl: "https://www.donationalerts.com/r/s00d88",
  videoUrl:
    "https://github.com/s00d/subly/raw/main/store-assets/video/store-walkthrough-desktop-with-audio.mp4",
} as const;

export const navLinks = [
  { id: "why", label: "Why Subly" },
  { id: "subscriptions", label: "Subscriptions" },
  { id: "dashboard", label: "Dashboard" },
  { id: "ai", label: "AI assist" },
  { id: "sync", label: "Sync" },
  { id: "faq", label: "FAQ" },
] as const;

export const hero = {
  headline: "Know what you pay — before it renews.",
  subline:
    "Subly tracks subscriptions and everyday spending in one fast, private app. Your data stays on your device by default; turn on sync only when you want it.",
  ctaPrimary: "Download latest release",
  ctaSecondary: "View on GitHub",
} as const;

export const platforms = {
  title: "Available everywhere you work",
  items: ["macOS", "Windows", "Linux", "iOS", "Android"] as const,
  tagline: "Local-first by design — no forced cloud account to use the app.",
} as const;

export const whyCards = [
  {
    title: "Your data, your device",
    body: "Subscriptions, expenses, and settings live on your machine or phone. You stay in control without signing up for our servers.",
  },
  {
    title: "Sync when you choose",
    body: "Optional backup through iCloud, Google Drive, Dropbox, OneDrive, or WebDAV — only if you enable it in Settings.",
  },
  {
    title: "12 languages",
    body: "English, Russian, Chinese, Spanish, French, German, Portuguese, Japanese, Korean, Arabic, Hindi, and Turkish.",
  },
  {
    title: "Built for real life",
    body: "Dashboards, a renewal calendar, multi-currency support, reminders, and exports — everything in one calm interface.",
  },
] as const;

export const featureSections = [
  {
    id: "subscriptions",
    eyebrow: "Subscriptions",
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
  title: "Optional help when you want speed",
  body: "AI is off until you turn it on. Bring your own API key from a provider you trust — nothing is sent until you use the assist dialog.",
  bullets: [
    "Add a subscription or expense from a plain sentence",
    "Drop a receipt photo for line items, totals, and merchant details",
    "Import bank statements (CSV, XLSX, PDF, JSON) with per-row preview before saving",
    "Supports OpenAI, OpenRouter, Gemini, Groq, DeepSeek, Mistral, and OpenAI-compatible endpoints (Ollama, LM Studio)",
    "Reuses your categories, payment methods, and favorite tags to avoid duplicates",
  ],
  screenshot: "06_settings_overview.png",
  alt: "Subly settings overview",
} as const;

export const syncSection = {
  eyebrow: "Sync & backup",
  title: "Your backup, your cloud",
  body: "When you need multiple devices, enable the provider you already use. Export and import full backups anytime.",
  providers: [
    { id: "icloud", label: "iCloud", icon: null },
    { id: "google-drive", label: "Google Drive", icon: "google-drive.svg" },
    { id: "dropbox", label: "Dropbox", icon: "dropbox.svg" },
    { id: "onedrive", label: "OneDrive", icon: "onedrive.svg" },
    { id: "webdav", label: "WebDAV", icon: null },
  ],
  bullets: [
    "Encrypted sync payloads between your devices",
    "Export or import `.subly` backups, plus JSON and CSV",
    "Household members, categories, and tags travel with your data",
  ],
} as const;

export const privacySection = {
  eyebrow: "Privacy",
  title: "Private by default",
  body: "Subly is built for people who want clarity without giving up ownership of their finances.",
  bullets: [
    "Core data stored locally on each device",
    "API keys, sync passwords, and Telegram tokens live in the OS keychain — not in plain text in the database",
    "AI and cloud features are opt-in; read our Privacy Policy for full details",
  ],
} as const;

export const customizeSection = {
  eyebrow: "Make it yours",
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

export const gallery = {
  title: "Inside the app",
  subtitle: "Desktop screenshots from the macOS build.",
  shots: [
    { file: "01_dashboard_overview.png", label: "Dashboard" },
    { file: "02_subscriptions_list.png", label: "Subscriptions" },
    { file: "03_expenses_list.png", label: "Expenses" },
    { file: "04_calendar_month.png", label: "Calendar" },
    { file: "05_rates_screen.png", label: "Currencies" },
    { file: "06_settings_overview.png", label: "Settings" },
  ],
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
    a: "No. AI assist is optional and disabled until you add a provider and API key in Settings.",
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
    q: "Can I try it before installing?",
    a: "Watch the desktop walkthrough video linked below, or browse the screenshot gallery on this page.",
  },
] as const;

export const downloadSection = {
  title: "Ready to take control?",
  body: "Grab the latest release for your platform, star the repo, or read the changelog to see what shipped recently.",
  ctaPrimary: "Download from GitHub",
  ctaSecondary: "Star on GitHub",
  ctaDonate: "Support development",
  ctaVideo: "Watch walkthrough",
} as const;

export const footer = {
  tagline: "Subly — subscription & expense tracker",
  license: "MIT License",
  links: [
    { label: "GitHub", href: siteMeta.repoUrl },
    { label: "Releases", href: siteMeta.releasesUrl },
    { label: "Privacy", href: siteMeta.privacyUrl },
    { label: "Changelog", href: siteMeta.changelogUrl },
  ],
} as const;
