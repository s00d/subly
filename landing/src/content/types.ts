import type { MdiIconName } from "@/mdi-icons";

export type LocaleId = "en" | "ru";

export interface LocaleOption {
  id: LocaleId;
  label: string;
}

export interface SiteContent {
  siteMeta: {
    title: string;
    description: string;
    siteUrl: string;
    canonicalUrl: string;
    ogImage: string;
    locale: string;
    softwareVersion: string;
    applicationCategory: string;
    operatingSystems: readonly string[];
    repoUrl: string;
    releasesUrl: string;
    privacyUrl: string;
    changelogUrl: string;
    appStoreUrl: string;
    donateUrl: string;
    videoDesktopUrl: string;
    videoMobileUrl: string;
  };
  localeSwitcher: {
    ariaLabel: string;
    options: readonly LocaleOption[];
  };
  headerNav: {
    download: string;
    menuToggle: string;
    moreLabel: string;
    moreMenuAria: string;
  };
  navPrimaryLinks: readonly { id: string; label: string; icon: string }[];
  navMoreLinks: readonly { id: string; label: string; icon: string }[];
  navLinks: readonly { id: string; label: string; icon: string }[];
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
    demos: readonly {
      id: string;
      label: string;
      icon: string;
      description: string;
      url: string;
      poster: string;
      aspect: "video" | "portrait";
    }[];
  };
  platforms: {
    title: string;
    items: readonly { name: string; icon: string }[];
    tagline: string;
  };
  openValuesSection: {
    eyebrow: string;
    icon: string;
    title: string;
    lead: string;
    pillars: readonly { icon: string; title: string; body: string }[];
    productHighlights: readonly string[];
    ctaLabel: string;
  };
  whySection: {
    title: string;
    subtitle: string;
  };
  whyCards: readonly { icon: string; title: string; body: string }[];
  featureSections: readonly {
    id: string;
    eyebrow: string;
    icon: string;
    title: string;
    body: string;
    bullets: readonly string[];
    screenshot: string;
    alt: string;
  }[];
  dashboardWidgets: readonly string[];
  aiSection: {
    eyebrow: string;
    icon: string;
    title: string;
    lead: string;
    capabilitiesTitle: string;
    capabilitiesIntro: string;
    inputMethodsTitle: string;
    principles: readonly { icon: string; title: string; body: string }[];
    features: readonly {
      icon: string;
      title: string;
      body: string;
      example: string;
    }[];
    inputMethods: readonly { icon: string; label: string; detail: string }[];
    providersTitle: string;
    providers: readonly string[];
    settingsBullets: readonly string[];
    privacyNote: string;
    screenshot: string;
    alt: string;
  };
  syncSection: {
    eyebrow: string;
    icon: string;
    title: string;
    body: string;
    providers: readonly {
      id: string;
      label: string;
      icon: string;
      brandIcon: string | null;
    }[];
    bullets: readonly string[];
  };
  privacySection: {
    eyebrow: string;
    icon: string;
    title: string;
    body: string;
    bullets: readonly string[];
    policyLinkLabel: string;
  };
  customizeSection: {
    eyebrow: string;
    icon: string;
    title: string;
    body: string;
    bullets: readonly string[];
    screenshot: string;
    alt: string;
  };
  deviceGalleryMeta: { title: string; subtitle: string };
  deviceGalleries: readonly {
    id: string;
    label: string;
    icon: string;
    description: string;
    shots: readonly { file: string; label: string }[];
  }[];
  appStore: {
    url: string;
    title: string;
    body: string;
    platforms: readonly string[];
    ctaLabel: string;
  };
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
    links: readonly (
      | { label: string; href: string; icon: string; external: true }
      | { label: string; to: string; icon: string; external: false }
    )[];
  };
  pageTitles: {
    privacy: string;
    changelog: string;
  };
}

export type ContentIcon = MdiIconName | string;
