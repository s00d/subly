[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-blue?style=for-the-badge)](https://github.com/s00d/subly)
[![License](https://img.shields.io/badge/license-MIT-green?style=for-the-badge)](https://github.com/s00d/subly/blob/main/LICENSE)
[![GitHub release](https://img.shields.io/github/v/release/s00d/subly?style=for-the-badge)](https://github.com/s00d/subly/releases)
[![GitHub downloads](https://img.shields.io/github/downloads/s00d/subly/total?style=for-the-badge)](https://github.com/s00d/subly/releases)
[![GitHub issues](https://img.shields.io/badge/github-issues-orange?style=for-the-badge)](https://github.com/s00d/subly/issues)
[![GitHub stars](https://img.shields.io/badge/github-stars-yellow?style=for-the-badge)](https://github.com/s00d/subly/stargazers)
[![Donate](https://img.shields.io/badge/Donate-Donationalerts-ff4081?style=for-the-badge)](https://www.donationalerts.com/r/s00d88)

# Subly — Subscription & Expense Tracker

<p align="center">
  <img src="app-icon.svg" alt="Subly" width="128" height="128" />
</p>

<p align="center">
  <strong>Desktop app for tracking subscriptions and expenses</strong><br>
  Tauri v2 · Vue 3 · TypeScript · Tailwind CSS v4
</p>

---

## Features

### Subscriptions
- Add, edit, and delete subscriptions (daily, weekly, monthly, yearly cycles)
- Favorites — pin important subscriptions to the top
- Tags and categories with custom icons
- Group by category or payment method
- Compact and expanded list views
- Batch actions (delete, change category, add tags)
- Detailed subscription card with payment history
- Auto-renewal and overdue payment tracking

### Expenses
- Track one-time expenses with categories, tags, and notes
- Filter by period, category, and tags
- Export expenses to CSV

### Dashboard
- Summary stats: monthly/yearly spending, active subscription count
- Charts: category breakdown (Doughnut), spending trend (Bar)
- Spending forecast for the next month/quarter
- Lifetime cost of subscriptions
- Average cost per category
- Customizable widget order and visibility
- Overdue payment alerts

### Calendar
- Visualize upcoming payments on a calendar grid
- Monthly stats: payment count and total amount
- Click to navigate to subscription details

### Notifications
- System notifications for upcoming and overdue payments
- Recurring reminders every payment cycle
- Notification schedule: morning, evening, or custom time
- Customizable notification titles and messages
- Telegram bot integration for sending alerts
- In-app alerts with copy and dismiss actions

### Currencies
- 34 currencies out of the box with the ability to add custom ones
- Automatic rate updates via Fixer.io (on a schedule)
- Conversion to selected target currencies

### Settings
- **Appearance**: 15+ color themes, responsive interface
- **Categories**: custom icons, ordering, default category
- **Payment methods**: icons, editing, default method
- **Tags**: favorites, editing, sorting
- **Currencies**: main currency, manual/automatic rate updates
- **Household members**: track expenses per family member
- **Budget**: monthly budget with progress bar
- **Data**: export/import `.subly`, JSON, CSV; import from CSV

### System Tray
- Tray icon showing upcoming payments
- Click on a payment to open the app and show details
- Minimize to tray on window close
- Auto-start on system boot

### Localization
12 languages: English, Русский, 中文, Español, Français, Deutsch, Português, 日本語, 한국어, العربية, हिन्दी, Türkçe

---

## Tech Stack

| Layer | Stack |
|-------|-------|
| **Framework** | [Tauri v2](https://v2.tauri.app/) |
| **Frontend** | [Vue 3](https://vuejs.org/) (Composition API, `<script setup>`) |
| **Styling** | [Tailwind CSS v4](https://tailwindcss.com/) |
| **Type Safety** | TypeScript + [Zod v4](https://zod.dev/) |
| **Charts** | [Chart.js](https://www.chartjs.org/) + [vue-chartjs](https://vue-chartjs.org/) |
| **Icons** | [Lucide](https://lucide.dev/) |
| **Routing** | [Vue Router](https://router.vuejs.org/) |
| **Bundler** | [Vite](https://vitejs.dev/) |
| **Package Manager** | [pnpm](https://pnpm.io/) |

### Tauri Plugins
- `@tauri-apps/plugin-store` — local data storage
- `@tauri-apps/plugin-notification` — system notifications
- `@tauri-apps/plugin-dialog` — file picker dialogs
- `@tauri-apps/plugin-fs` — file system access
- `@tauri-apps/plugin-clipboard-manager` — clipboard
- `@tauri-apps/plugin-autostart` — launch on boot
- `@tauri-apps/plugin-opener` — open URLs

---

## Project Structure

```
app/subly/
├── src/
│   ├── components/
│   │   ├── calendar/        # Calendar components
│   │   ├── dashboard/       # Dashboard widgets
│   │   ├── expenses/        # Expense form
│   │   ├── layout/          # AppLayout, Header, Sidebar, MobileTabBar
│   │   ├── settings/        # Settings sections
│   │   ├── subscriptions/   # Form, detail, payment history
│   │   └── ui/              # Reusable UI components
│   ├── composables/         # useAlerts, useToast, useCurrencyFormat, etc.
│   ├── i18n/                # 12 language files
│   ├── pages/               # Dashboard, Subscriptions, Expenses, Calendar, Settings
│   ├── router/              # Vue Router
│   ├── schemas/             # Zod data schemas
│   ├── services/            # Business logic, notifications, export, tray
│   └── stores/              # Global store (appStore)
├── src-tauri/
│   ├── src/                 # Rust: main.rs, lib.rs (tray, menu, windows)
│   ├── capabilities/        # Tauri permissions
│   ├── icons/               # App icons
│   └── tauri.conf.json      # Tauri configuration
├── public/assets/           # SVG service logos
├── .github/workflows/       # GitHub Actions for automated builds
├── index.html
├── package.json
├── vite.config.ts
└── tsconfig.json
```

---

## Getting Started

### Prerequisites
- [Node.js](https://nodejs.org/) >= 20
- [pnpm](https://pnpm.io/) >= 9
- [Rust](https://www.rust-lang.org/tools/install) (stable)
- Tauri system dependencies — see [docs](https://v2.tauri.app/start/prerequisites/)

### Build Steps

```bash
# Clone the repository
git clone https://github.com/s00d/subly.git
cd subly/app/subly

# Install dependencies
pnpm install

# Development mode
pnpm tauri dev

# Build for production
pnpm tauri build
```

Built binaries will be in `src-tauri/target/release/bundle/`.

### Generate Icons

```bash
pnpm tauri icon app-icon.svg
```

### Platform-Specific Notes

#### macOS

```bash
# If you encounter signing issues after installing
chmod +x /Applications/Subly.app
xattr -cr /Applications/Subly.app
codesign --force --deep --sign - /Applications/Subly.app
```

> **Note:** macOS builds are unsigned. On first launch you may need to right-click the app → Open, or run the commands above.

#### Windows

```bash
# Install Rust (https://rustup.rs) and Node.js (https://nodejs.org)
# Then follow the build steps above
```

#### Linux

```bash
# Install system dependencies
sudo apt-get update
sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf libssl-dev

# Then follow the build steps above
```

---

## Screenshots

> _TODO: add screenshots_

---

## CI/CD

A GitHub Actions workflow is configured in `.github/workflows/build.yml` for automated builds on macOS (Universal), Linux, and Windows. Code signing is disabled.

Releases are created automatically when a `v*` tag is pushed.

---

## License

MIT
