import { buildSiteContent } from "./buildSiteContent";
import type { SiteLocaleStrings } from "./buildSiteContent";

const deStrings: SiteLocaleStrings = {
  site: {
    title: "Subly — Abo- & Ausgaben-Tracker",
    description:
      "Open-Source-Tracker für Abonnements und Ausgaben. Local-first, ohne Analytics oder Tracking, optionale Cloud-Synchronisation.",
  },
  localeSwitcherAria: "Sprache",
  headerNav: {
    download: "Download",
    menuToggle: "Menü umschalten",
    moreLabel: "Mehr",
    moreMenuAria: "Weitere Bereiche",
  },
  nav: {
    why: "Warum Subly",
    subscriptions: "Abonnements",
    dashboard: "Dashboard",
    video: "Demo",
    ai: "KI",
    sync: "Sync",
    faq: "FAQ",
  },
  hero: {
    headline: "Wissen, wofür Sie zahlen — bevor es verlängert wird.",
    subline:
      "Ein schneller Abo- und Ausgaben-Tracker auf Ihrem Gerät, vollständig offener MIT-Quellcode und ohne Analytics oder Nutzungsmetriken.",
    ctaPrimary: "Neueste Version herunterladen",
    ctaSecondary: "Auf GitHub ansehen",
    watchDemo: "Demo ansehen",
  },
  videoSection: {
    eyebrow: "Produktdemo",
    title: "Subly in Aktion",
    subtitle: "Wechseln Sie zwischen Desktop- und Handy-Aufnahmen.",
    fallbackLabel: "Video in neuem Tab öffnen, wenn die Wiedergabe nicht startet",
    desktopLabel: "Desktop",
    desktopDescription: "Abonnements, Ausgaben, Dashboard und Einstellungen unter macOS.",
    phoneLabel: "Smartphone",
    phoneDescription: "Dieselben Abläufe auf dem iPhone — Listen, Kalender und Einstellungen.",
  },
  platforms: {
    title: "Überall verfügbar, wo Sie arbeiten",
    tagline:
      "Local-first, vollständig Open Source und ohne Analytics — kein erzwungenes Cloud-Konto.",
  },
  openValuesSection: {
    eyebrow: "Offen & ehrlich",
    title: "Leistungsstarke App. Offener Code. Keine Überwachung.",
    lead:
      "Die meisten Finanz-Apps wollen Ihre Daten auf ihren Servern — und leise Analytics im Hintergrund. Subly ist das Gegenteil: MIT-lizenzierter Quellcode zum Lesen und Selbstbauen, standardmäßig local-first und ohne Tracking-Stack jeder Art. Kein „Privatmodus“. Kein „wir aggregieren anonym“. Einfach keine Metriken, keine Werbung, keine versteckten Pixel.",
    pillars: [
      "Vollständig Open Source",
      "Keine Analytics. Niemals.",
      "Vertrauen, das Sie prüfen können",
    ] as [string, string, string],
    pillarBodies: [
      "Desktop, Mobil, Sync und optionale KI — das gesamte Produkt steht unter MIT auf GitHub. Prüfen, forken, eigene Builds ausliefern oder Fixes beisteuern. Transparenz ist ein Feature.",
      "Kein Google Analytics, Firebase, Mixpanel, Amplitude, Sentry Session Replay oder Werbe-SDKs. Subly sammelt keine Nutzungsstatistiken und sendet keinen Crash-Telemetrie an unsere Server. Wie Sie die App nutzen, bleibt Ihre Sache.",
      "Daten liegen auf Ihrem Gerät. Sync nutzt Ihre Cloud, wenn Sie ihn aktivieren. API-Schlüssel liegen in der OS-Schlüsselbund-Verwaltung. Wenn die README „kein Tracking“ sagt, können Sie das Repo durchsuchen und es sehen.",
    ] as [string, string, string],
    productHighlights: [
      "Abonnements, Ausgaben, Dashboard und Verlängerungskalender an einem Ort",
      "34+ Währungen, 12 Sprachen, CSV/JSON-Exporte",
      "Optionale KI — vier Funktionen, sieben Anbieter, aus bis Sie sie aktivieren",
      "Funktioniert offline; kein Subly-Konto nötig",
    ],
    ctaLabel: "Quellcode auf GitHub erkunden",
  },
  whySection: {
    title: "Warum Menschen Subly wählen",
    subtitle:
      "Klarheit bei wiederkehrenden Rechnungen und Alltagsausgaben — offener Code, kein Tracking und Tools, die Ihre Daten respektieren.",
  },
  whyCards: [
    {
      title: "Open Source, MIT-Lizenz",
      body: "Jede Funktion liegt in einem öffentlichen Repository. Keine Blackbox um Ihre Finanzen — prüfen, bauen und teilen nach Ihren Regeln.",
    },
    {
      title: "Keine Metriken oder Tracking",
      body: "Wir messen keine Bildschirme, Klicks oder Abstürze für uns. Ihre Ausgabenmuster werden nie fremdes Dashboard.",
    },
    {
      title: "Ihre Daten, Ihr Gerät",
      body: "Abonnements, Ausgaben und Einstellungen bleiben auf Rechner oder Handy. Sie behalten die Kontrolle ohne Anmeldung bei unseren Servern.",
    },
    {
      title: "Sync, wenn Sie wollen",
      body: "Optionales Backup über iCloud, Google Drive, Dropbox, OneDrive oder WebDAV — nur wenn Sie es in den Einstellungen aktivieren.",
    },
    {
      title: "12 Sprachen",
      body: "Englisch, Russisch, Chinesisch, Spanisch, Französisch, Deutsch, Portugiesisch, Japanisch, Koreanisch, Arabisch, Hindi und Türkisch.",
    },
    {
      title: "Für den echten Alltag gebaut",
      body: "Dashboards, Verlängerungskalender, Mehrwährung, Erinnerungen und Exporte — alles in einer ruhigen Oberfläche.",
    },
  ],
  featureSections: [
    {
      eyebrow: "Abonnements",
      title: "Alle wiederkehrenden Kosten an einem Ort",
      body: "Streaming, Software, Versorger oder beliebige Dienste. Tägliche, wöchentliche, monatliche oder jährliche Abrechnung — plus eigene Zyklen, wo unterstützt.",
      bullets: [
        "Favoriten anheften und zwischen kompakter oder erweiterter Liste wechseln",
        "Nach Kategorie oder Zahlungsmittel gruppieren; Stapelaktionen zum Aufräumen",
        "Vollständiger Zahlungsverlauf, Auto-Verlängerung, Überfälligkeiten, Notizen und Links",
      ],
      alt: "Subly-Abonnementliste",
    },
    {
      eyebrow: "Ausgaben",
      title: "Einmalige Ausgaben ohne Tabellenkalkulation",
      body: "Erfassen Sie Käufe mit Betrag, Kategorie, Tags, Notizen und optionalen Links. Filtern Sie nach Zeitraum, Kategorie oder Tags für das Gesamtbild.",
      bullets: [
        "Ausgaben als CSV für Tabellen exportieren",
        "Ausgaben mit Abonnements verknüpfen, wenn es zu Ihrem Workflow passt",
        "Gleiche Kategorien, Tags und Zahlungsmittel wie bei Abonnements",
      ],
      alt: "Subly-Ausgabenliste",
    },
    {
      eyebrow: "Dashboard",
      title: "Das große Bild auf einen Blick",
      body: "Wählen Sie, welche Blöcke erscheinen und in welcher Reihenfolge. Von Überfällig-Hinweisen bis Prognosen und Lifetime-Abo-Kosten — Ihr Startbildschirm passt sich Ihnen an.",
      bullets: [
        "Überfällige und anstehende Zahlungen, Statistik, Monatsbudget-Fortschritt",
        "Ausgabentrends, Prognosen, Lifetime-Kosten, Kategorie-Durchschnitte",
        "Diagramme für Aufschlüsselungen, Top-Ausgaben, Wochentage, Tags und Kursverlauf",
      ],
      alt: "Subly-Dashboard-Übersicht",
    },
    {
      eyebrow: "Kalender",
      title: "Planen Sie um Gehalt und Rechnungen",
      body: "Anstehende Abbuchungen erscheinen im Kalender. Monatssummen prüfen und vom Datum direkt zum passenden Abonnement springen.",
      bullets: [
        "Anzahl und Gesamtbetrag der Zahlungen im Monat",
        "Klarer Blick, was vor der nächsten Gehaltszahlung fällig wird",
        "In einem Schritt zu den Abonnement-Details",
      ],
      alt: "Subly-Kalender Monatsansicht",
    },
    {
      eyebrow: "Geld & Erinnerungen",
      title: "Währungen und Benachrichtigungen, die mithalten",
      body: "Ausgaben in vielen Währungen verfolgen, Kurse planmäßig aktualisieren und vor Abbuchungen benachrichtigt werden.",
      bullets: [
        "34+ Währungen mit Platz für eigene Einträge",
        "Automatische Kursupdates vom gewählten Anbieter",
        "Systembenachrichtigungen für anstehende und überfällige Zahlungen",
        "Optional Telegram-Erinnerungen; anpassbare Titel und Zeitplan",
      ],
      alt: "Subly-Währungskurse",
    },
  ],
  dashboardWidgets: [
    "Überfällige & anstehende Zahlungen",
    "Statistik & Budget-Fortschritt",
    "Ausgabentrends & Prognosen",
    "Lifetime-Abo-Kosten",
    "Diagramme nach Kategorie",
    "Top-Ausgaben & Monatsvergleich",
    "Ausgaben nach Tags oder Wochentag",
  ],
  aiSection: {
    eyebrow: "KI-Assistenz",
    title: "Optionale KI — vier Wege, Daten schneller zu erfassen",
    capabilitiesTitle: "Funktionen",
    capabilitiesIntro:
      "Ein Assistenz-Einstieg bei Abonnements und Ausgaben — jede Funktion separat in den Einstellungen aktivieren.",
    inputMethodsTitle: "So können Sie Eingaben senden",
    lead:
      "Sublys KI-Schicht ist vollständig optional: deaktiviert, bis Sie den Hauptschalter in den Einstellungen setzen. Sie wählen den Anbieter, speichern den API-Schlüssel im OS-Schlüsselbund und schalten nur gewünschte Funktionen ein. Nichts geht an ein Modell, bis Sie den Assistenz-Dialog öffnen und absenden.",
    principles: [
      {
        title: "Standardmäßig aus",
        body: "Keine Hintergrundaufrufe, keine stillen Uploads. KI nur aktivieren, wenn Sie sie brauchen.",
      },
      {
        title: "Eigenen Schlüssel mitbringen",
        body: "Bezahlte Cloud-APIs oder lokales Ollama / LM Studio — Zugangsdaten liegen nie in der Datenbank.",
      },
      {
        title: "Vorschau vor dem Speichern",
        body: "Jedes Abo, jede Ausgabe oder importierte Zeile wird zur Prüfung gezeigt. Nichts landet ohne Bestätigung in Ihrer Buchführung.",
      },
    ],
    features: [
      {
        title: "Abos schnell hinzufügen",
        body: "Beschreiben Sie einen Dienst in Klartext — Preis, Zyklus, Kategorie, Zahlungsmittel und Tags werden aus Ihrem Katalog vorausgefüllt.",
        example: "„Telegram Premium 300 ₽ monatlich, Karte“",
      },
      {
        title: "Ausgaben schnell hinzufügen",
        body: "Derselbe Dialog für Einmalausgaben: Betrag, Datum, Händler, Kategorien und Tags aus einem kurzen Satz.",
        example: "„Kaffee bei Starbucks 5,40 EUR gestern“",
      },
      {
        title: "Beleg- & Dokumentenscan",
        body: "Vision-fähiges Modell für Foto oder PDF. Positionen, Summen, Währung, Händler und Datum als Ausgaben-Entwurf.",
        example: "PNG, JPEG, WebP, HEIC oder PDF-Belege",
      },
      {
        title: "Kontoauszug importieren",
        body: "Massenimport von CSV, XLSX, PDF oder JSON. Heuristische Analyse plus KI-Fallback, mit Zeilentabelle vor dem Speichern.",
        example: "Karten- und Bankexporte in gängigen Formaten",
      },
    ],
    inputMethods: [
      { label: "Beschreibung eingeben", detail: "⌘/Strg + Eingabe zum Senden" },
      { label: "Dateien ablegen oder wählen", detail: "Drag-and-drop auf dem Desktop" },
      { label: "Screenshots einfügen", detail: "In das Textfeld" },
    ],
    providersTitle: "Unterstützte Anbieter",
    settingsBullets: [
      "Hauptschalter plus einzelne Schalter pro Funktion",
      "Empfohlene Modell-Voreinstellungen und eigener Modellname",
      "Verbindung testen, bevor Sie sich darauf verlassen",
      "Vision-Badge, wenn das Modell Beleg-OCR unterstützt",
    ],
    privacyNote:
      "Anfragen gehen nur an den von Ihnen konfigurierten Anbieter. Subly betreibt keine eigene LLM und protokolliert Ihre Prompts nicht auf unseren Servern.",
    alt: "Subly-KI-Einstellungen",
  },
  syncSection: {
    eyebrow: "Sync & Backup",
    title: "Ihr Backup, Ihre Cloud",
    body: "Für mehrere Geräte aktivieren Sie den Anbieter, den Sie ohnehin nutzen. Vollständige Backups jederzeit exportieren und importieren.",
    bullets: [
      "Verschlüsselte Sync-Nutzlasten zwischen Ihren Geräten",
      "`.subly`-Backups sowie JSON und CSV exportieren oder importieren",
      "Haushaltsmitglieder, Kategorien und Tags reisen mit Ihren Daten",
    ],
  },
  privacySection: {
    eyebrow: "Datenschutz",
    title: "Standardmäßig privat",
    body: "Keine Analytics, keine Werbe-SDKs, keine „anonymen Nutzungs“-Pipelines — und offener Quellcode, damit Sie uns nicht blind vertrauen müssen.",
    bullets: [
      "Kern-Daten werden lokal auf jedem Gerät gespeichert",
      "Keine In-App-Metriken, Crash-Reporter oder Third-Party-Tracking zu Subly-Servern",
      "API-Schlüssel, Sync-Passwörter und Telegram-Tokens in der OS-Schlüsselbund-Verwaltung — nicht im Klartext in der DB",
      "KI und Cloud-Funktionen sind optional; Details in unserer Datenschutzerklärung",
    ],
    policyLinkLabel: "Privacy Policy (EN)",
  },
  customizeSection: {
    eyebrow: "Ganz nach Ihnen",
    title: "Themes, Tags und Haushalt",
    body: "Über 15 Farb-Themes, eigene Kategorie-Icons, Zahlungsmittel, Tags, Monatsbudget-Fortschritt und System-Tray-Kürzel auf dem Desktop.",
    bullets: [
      "Haushaltsmitglieder für korrekte Zuordnung von Ausgaben",
      "System-Tray mit anstehenden Zahlungen und Schnellöffnen",
      "In Tray minimieren, Autostart beim Boot (Desktop)",
    ],
    alt: "Subly-Erscheinungsbild und Einstellungen",
  },
  deviceGalleryMeta: {
    title: "In der App",
    subtitle: "Dieselben Kernabläufe auf Desktop, Smartphone und Tablet.",
  },
  shotLabels: {
    dashboard: "Dashboard",
    subscriptions: "Abonnements",
    expenses: "Ausgaben",
    calendar: "Kalender",
    currencies: "Währungen",
    settings: "Einstellungen",
  },
  deviceGalleries: [
    {
      description:
        "Volles Desktop-Erlebnis mit Dashboard-Widgets, System-Tray und Exporten — für macOS, Windows und Linux.",
    },
    {
      description:
        "Abonnements und Ausgaben unterwegs mit denselben Kategorien, Tags und Verlängerungskalender.",
    },
    {
      description:
        "Mehr Platz für Listen und Monatskalender auf iPad und Android-Tablets — ohne local-first-Datenschutz zu opfern.",
    },
  ],
  appStore: {
    title: "Auch im App Store",
    body: "Subly gibt es für Mac und iPhone/iPad im App Store. Der Download ist kostenpflichtig — Ihr Kauf unterstützt die Weiterentwicklung. Desktop-Builds für macOS, Windows und Linux bleiben kostenlos auf GitHub Releases.",
    ctaLabel: "Im App Store ansehen",
  },
  faqSection: {
    title: "FAQ",
  },
  faqItems: [
    {
      q: "Ist Subly kostenlos?",
      a: "Subly ist Open Source unter der MIT-Lizenz. Builds von GitHub Releases sind kostenlos. In Stores können je nach Plattform Gebühren anfallen.",
    },
    {
      q: "Funktioniert es offline?",
      a: "Ja. Abonnements, Ausgaben, Dashboard und Kalender arbeiten vollständig offline. Sync, Kursupdates, KI und Telegram brauchen Netz, wenn Sie diese Funktionen nutzen.",
    },
    {
      q: "Brauche ich KI?",
      a: "Nein. KI ist optional und standardmäßig aus. Wenn aktiv, können Sie Abos oder Ausgaben schnell erfassen, Belege scannen oder Auszüge importieren — jede Funktion hat einen eigenen Schalter, Sie wählen aus sieben Anbietern mit eigenem API-Schlüssel.",
    },
    {
      q: "Ist Cloud-Sync sicher?",
      a: "Sync nutzt Ihr eigenes Cloud-Konto (iCloud, Drive, Dropbox, OneDrive oder WebDAV). Zugangsdaten liegen im System-Schlüsselbund. Aktivieren Sie Sync nur, wenn Sie Mehrgeräte-Backup wollen.",
    },
    {
      q: "Welche Plattformen werden unterstützt?",
      a: "macOS (Universal), Windows, Linux, iOS und Android. Desktop-Installer auf GitHub Releases; Mobile dort, wo im Store veröffentlicht.",
    },
    {
      q: "Wie sichere ich meine Daten?",
      a: "Einstellungen → Daten: `.subly`-Datei, JSON oder CSV exportieren. Dieselben Formate beim Wechsel auf ein neues Gerät importieren.",
    },
    {
      q: "Wo werden meine Daten gespeichert?",
      a: "Standardmäßig auf Ihrem Gerät. Wir betreiben keine zentrale Subly-Cloud mit Ihren Abonnements.",
    },
    {
      q: "Nutzt Subly Analytics oder Tracking?",
      a: "Nein. Keine Analytics-SDKs, Werbenetzwerke oder Nutzungsmetriken an uns. Optionale Funktionen (Sync, KI, Telegram, Kurse) sprechen nur mit von Ihnen konfigurierten Diensten.",
    },
    {
      q: "Kann ich den Quellcode lesen?",
      a: "Ja. Subly ist vollständig Open Source auf GitHub unter MIT — Frontend, Backend und Mobile. Sie können Datenschutz-Aussagen selbst prüfen.",
    },
    {
      q: "Kann ich vor der Installation testen?",
      a: "Sehen Sie die Desktop- oder Handy-Demo-Videos auf dieser Seite oder die Screenshot-Galerie unten.",
    },
  ],
  downloadSection: {
    title: "Bereit, die Kontrolle zu übernehmen?",
    body: "Kostenlose Desktop-Builds von GitHub Releases — Plattform unten wählen. Lieber App Store auf Apple-Geräten? Das unterstützt die Entwicklung.",
    ctaPrimary: "Neueste Version herunterladen",
    githubHeading: "Kostenlos auf GitHub",
    ctaSecondary: "Auf GitHub mit Stern markieren",
    ctaDonate: "Entwicklung unterstützen",
    ctaVideo: "Demo ansehen",
    changelogLabel: "Changelog",
  },
  footer: {
    tagline: "Subly — Abo- & Ausgaben-Tracker",
    license: "MIT-Lizenz",
    donateLabel: "Spenden",
    releasesLabel: "Releases",
  },
};

export { deStrings };

export const deContent = buildSiteContent("de", deStrings);
