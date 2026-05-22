import type { SiteContent } from "./types";
import type { LocaleId } from "./locales";
import { buildLocaleSwitcherOptions, LOCALE_META } from "./locales";
import { sharedUrls, SOFTWARE_VERSION } from "./shared";

export interface SiteLocaleStrings {
  site: { title: string; description: string };
  localeSwitcherAria: string;
  headerNav: {
    download: string;
    menuToggle: string;
    moreLabel: string;
    moreMenuAria: string;
  };
  nav: {
    why: string;
    subscriptions: string;
    dashboard: string;
    video: string;
    ai: string;
    sync: string;
    faq: string;
  };
  hero: {
    headline: string;
    subline: string;
    ctaPrimary: string;
    ctaSecondary: string;
    watchDemo: string;
  };
  videoSection: {
    eyebrow: string;
    title: string;
    subtitle: string;
    fallbackLabel: string;
    desktopLabel: string;
    desktopDescription: string;
    phoneLabel: string;
    phoneDescription: string;
  };
  platforms: { title: string; tagline: string };
  openValuesSection: {
    eyebrow: string;
    title: string;
    lead: string;
    pillars: [string, string, string];
    pillarBodies: [string, string, string];
    productHighlights: readonly string[];
    ctaLabel: string;
  };
  whySection: { title: string; subtitle: string };
  whyCards: readonly { title: string; body: string }[];
  featureSections: readonly {
    eyebrow: string;
    title: string;
    body: string;
    bullets: readonly string[];
    alt: string;
  }[];
  dashboardWidgets: readonly string[];
  aiSection: {
    eyebrow: string;
    title: string;
    capabilitiesTitle: string;
    capabilitiesIntro: string;
    inputMethodsTitle: string;
    lead: string;
    principles: readonly { title: string; body: string }[];
    features: readonly { title: string; body: string; example: string }[];
    inputMethods: readonly { label: string; detail: string }[];
    providersTitle: string;
    settingsBullets: readonly string[];
    privacyNote: string;
    alt: string;
  };
  syncSection: {
    eyebrow: string;
    title: string;
    body: string;
    bullets: readonly string[];
  };
  privacySection: {
    eyebrow: string;
    title: string;
    body: string;
    bullets: readonly string[];
    policyLinkLabel: string;
  };
  customizeSection: {
    eyebrow: string;
    title: string;
    body: string;
    bullets: readonly string[];
    alt: string;
  };
  deviceGalleryMeta: { title: string; subtitle: string };
  shotLabels: {
    dashboard: string;
    subscriptions: string;
    expenses: string;
    calendar: string;
    currencies: string;
    settings: string;
  };
  deviceGalleries: readonly { description: string }[];
  appStore: { title: string; body: string; ctaLabel: string };
  faqSection: { title: string };
  faqItems: readonly { q: string; a: string }[];
  downloadSection: {
    title: string;
    body: string;
    ctaPrimary: string;
    githubHeading: string;
    ctaSecondary: string;
    ctaDonate: string;
    ctaVideo: string;
    changelogLabel: string;
  };
  footer: {
    tagline: string;
    license: string;
    donateLabel: string;
    releasesLabel: string;
  };
}

const FEATURE_IDS = [
  "subscriptions",
  "expenses",
  "dashboard",
  "calendar",
  "currencies",
] as const;

const FEATURE_ICONS = ["repeat", "cash-multiple", "view-dashboard-outline", "calendar-month-outline", "currency-usd"] as const;
const FEATURE_SCREENSHOTS = [
  "02_subscriptions_list.png",
  "03_expenses_list.png",
  "01_dashboard_overview.png",
  "04_calendar_month.png",
  "05_rates_screen.png",
] as const;

function shotList(labels: SiteLocaleStrings["shotLabels"]) {
  return [
    { file: "01_dashboard_overview.png", label: labels.dashboard },
    { file: "02_subscriptions_list.png", label: labels.subscriptions },
    { file: "03_expenses_list.png", label: labels.expenses },
    { file: "04_calendar_month.png", label: labels.calendar },
    { file: "05_rates_screen.png", label: labels.currencies },
    { file: "06_settings_overview.png", label: labels.settings },
  ] as const;
}

function phoneShots(labels: SiteLocaleStrings["shotLabels"]) {
  return shotList(labels).map((s) => ({
    file: `phone/${s.file}`,
    label: s.label,
  }));
}

function tabletShots(labels: SiteLocaleStrings["shotLabels"]) {
  return shotList(labels).map((s) => ({
    file: `tablet/${s.file}`,
    label: s.label,
  }));
}

export function buildSiteContent(locale: LocaleId, s: SiteLocaleStrings): SiteContent {
  const meta = LOCALE_META[locale];
  const shots = shotList(s.shotLabels);

  const navPrimaryLinks = [
    { id: "why", label: s.nav.why, icon: "heart-outline" },
    { id: "subscriptions", label: s.nav.subscriptions, icon: "repeat" },
    { id: "dashboard", label: s.nav.dashboard, icon: "view-dashboard-outline" },
    { id: "video", label: s.nav.video, icon: "play-circle-outline" },
  ] as const;

  const navMoreLinks = [
    { id: "ai", label: s.nav.ai, icon: "robot-outline" },
    { id: "sync", label: s.nav.sync, icon: "cloud-sync-outline" },
    { id: "faq", label: s.nav.faq, icon: "help-circle-outline" },
  ] as const;

  return {
    siteMeta: {
      title: s.site.title,
      description: s.site.description,
      siteUrl: meta.siteUrl,
      canonicalUrl: meta.canonicalUrl,
      locale: meta.ogLocale,
      softwareVersion: SOFTWARE_VERSION,
      applicationCategory: "FinanceApplication",
      operatingSystems: ["macOS", "Windows", "Linux", "iOS", "Android"],
      ...sharedUrls,
    },
    localeSwitcher: buildLocaleSwitcherOptions(s.localeSwitcherAria),
    headerNav: s.headerNav,
    navPrimaryLinks,
    navMoreLinks,
    navLinks: [...navPrimaryLinks, ...navMoreLinks],
    hero: s.hero,
    videoSection: {
      eyebrow: s.videoSection.eyebrow,
      title: s.videoSection.title,
      subtitle: s.videoSection.subtitle,
      fallbackLabel: s.videoSection.fallbackLabel,
      demos: [
        {
          id: "desktop",
          label: s.videoSection.desktopLabel,
          icon: "microsoft-windows",
          description: s.videoSection.desktopDescription,
          url: sharedUrls.videoDesktopUrl,
          poster: "01_dashboard_overview.png",
          aspect: "video",
        },
        {
          id: "phone",
          label: s.videoSection.phoneLabel,
          icon: "cellphone",
          description: s.videoSection.phoneDescription,
          url: sharedUrls.videoMobileUrl,
          poster: "phone/01_dashboard_overview.png",
          aspect: "portrait",
        },
      ],
    },
    platforms: {
      title: s.platforms.title,
      items: [
        { name: "macOS", icon: "apple" },
        { name: "Windows", icon: "microsoft-windows" },
        { name: "Linux", icon: "linux" },
        { name: "iOS", icon: "apple-ios" },
        { name: "Android", icon: "android" },
      ],
      tagline: s.platforms.tagline,
    },
    openValuesSection: {
      eyebrow: s.openValuesSection.eyebrow,
      icon: "github",
      title: s.openValuesSection.title,
      lead: s.openValuesSection.lead,
      pillars: [
        { icon: "github", title: s.openValuesSection.pillars[0], body: s.openValuesSection.pillarBodies[0] },
        { icon: "eye-off-outline", title: s.openValuesSection.pillars[1], body: s.openValuesSection.pillarBodies[1] },
        { icon: "shield-check-outline", title: s.openValuesSection.pillars[2], body: s.openValuesSection.pillarBodies[2] },
      ],
      productHighlights: s.openValuesSection.productHighlights,
      ctaLabel: s.openValuesSection.ctaLabel,
    },
    whySection: s.whySection,
    whyCards: [
      { icon: "github", title: s.whyCards[0].title, body: s.whyCards[0].body },
      { icon: "eye-off-outline", title: s.whyCards[1].title, body: s.whyCards[1].body },
      { icon: "cellphone-link-off", title: s.whyCards[2].title, body: s.whyCards[2].body },
      { icon: "cloud-sync-outline", title: s.whyCards[3].title, body: s.whyCards[3].body },
      { icon: "translate", title: s.whyCards[4].title, body: s.whyCards[4].body },
      { icon: "view-dashboard-outline", title: s.whyCards[5].title, body: s.whyCards[5].body },
    ],
    featureSections: FEATURE_IDS.map((id, i) => ({
      id,
      eyebrow: s.featureSections[i].eyebrow,
      icon: FEATURE_ICONS[i],
      title: s.featureSections[i].title,
      body: s.featureSections[i].body,
      bullets: s.featureSections[i].bullets,
      screenshot: FEATURE_SCREENSHOTS[i],
      alt: s.featureSections[i].alt,
    })),
    dashboardWidgets: [...s.dashboardWidgets],
    aiSection: {
      eyebrow: s.aiSection.eyebrow,
      icon: "robot-outline",
      title: s.aiSection.title,
      capabilitiesTitle: s.aiSection.capabilitiesTitle,
      capabilitiesIntro: s.aiSection.capabilitiesIntro,
      inputMethodsTitle: s.aiSection.inputMethodsTitle,
      lead: s.aiSection.lead,
      principles: s.aiSection.principles.map((p, idx) => ({
        icon: ["toggle-switch-off-outline", "key", "clipboard-check-outline"][idx],
        title: p.title,
        body: p.body,
      })),
      features: s.aiSection.features.map((f, idx) => ({
        icon: ["repeat", "cash-multiple", "receipt", "file-document-multiple-outline"][idx],
        title: f.title,
        body: f.body,
        example: f.example,
      })),
      inputMethods: s.aiSection.inputMethods.map((m, idx) => ({
        icon: ["keyboard-outline", "image-plus-outline", "content-paste"][idx],
        label: m.label,
        detail: m.detail,
      })),
      providersTitle: s.aiSection.providersTitle,
      providers: [
        "OpenAI",
        "OpenRouter",
        "Google Gemini",
        "Groq",
        "DeepSeek",
        "Mistral",
        "OpenAI-compatible (Ollama, LM Studio, vLLM)",
      ],
      settingsBullets: [...s.aiSection.settingsBullets],
      privacyNote: s.aiSection.privacyNote,
      screenshot: "06_settings_overview.png",
      alt: s.aiSection.alt,
    },
    syncSection: {
      eyebrow: s.syncSection.eyebrow,
      icon: "backup-restore",
      title: s.syncSection.title,
      body: s.syncSection.body,
      providers: [
        { id: "icloud", label: "iCloud", icon: "apple-icloud", brandIcon: null },
        { id: "google-drive", label: "Google Drive", icon: "google-drive", brandIcon: "google-drive.svg" },
        { id: "dropbox", label: "Dropbox", icon: "dropbox", brandIcon: "dropbox.svg" },
        { id: "onedrive", label: "OneDrive", icon: "microsoft-onedrive", brandIcon: "onedrive.svg" },
        { id: "webdav", label: "WebDAV", icon: "web", brandIcon: null },
      ],
      bullets: [...s.syncSection.bullets],
    },
    privacySection: {
      eyebrow: s.privacySection.eyebrow,
      icon: "shield-lock-outline",
      title: s.privacySection.title,
      body: s.privacySection.body,
      bullets: [...s.privacySection.bullets],
      policyLinkLabel: s.privacySection.policyLinkLabel,
    },
    customizeSection: {
      eyebrow: s.customizeSection.eyebrow,
      icon: "palette-outline",
      title: s.customizeSection.title,
      body: s.customizeSection.body,
      bullets: [...s.customizeSection.bullets],
      screenshot: "06_settings_overview.png",
      alt: s.customizeSection.alt,
    },
    deviceGalleryMeta: s.deviceGalleryMeta,
    deviceGalleries: [
      {
        id: "desktop",
        label: "macOS",
        icon: "apple",
        description: s.deviceGalleries[0].description,
        shots,
      },
      {
        id: "phone",
        label: "iPhone",
        icon: "cellphone",
        description: s.deviceGalleries[1].description,
        shots: phoneShots(s.shotLabels),
      },
      {
        id: "tablet",
        label: "Tablet",
        icon: "tablet",
        description: s.deviceGalleries[2].description,
        shots: tabletShots(s.shotLabels),
      },
    ],
    appStore: {
      url: sharedUrls.appStoreUrl,
      title: s.appStore.title,
      body: s.appStore.body,
      platforms: ["macOS", "iOS", "iPadOS"],
      ctaLabel: s.appStore.ctaLabel,
    },
    faqSection: s.faqSection,
    faqItems: [...s.faqItems],
    downloadSection: s.downloadSection,
    footer: {
      tagline: s.footer.tagline,
      license: s.footer.license,
      donateLabel: s.footer.donateLabel,
      links: [
        { label: "GitHub", href: sharedUrls.repoUrl, icon: "github", external: true },
        { label: s.footer.releasesLabel, href: sharedUrls.releasesUrl, icon: "download", external: true },
        { label: "Privacy", to: "/privacy", icon: "shield-check-outline", external: false },
        { label: "Changelog", to: "/changelog", icon: "history", external: false },
      ],
    },
    pageTitles: {
      privacy: "Privacy Policy — Subly",
      changelog: "Changelog — Subly",
    },
  };
}
