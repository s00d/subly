import { buildSiteContent } from "./buildSiteContent";
import type { SiteLocaleStrings } from "./buildSiteContent";

const frStrings: SiteLocaleStrings = {
  site: {
    title: "Subly — Suivi d'abonnements et de dépenses",
    description:
      "Suivi open source des abonnements et dépenses. Local-first, sans analytique ni pistage, synchronisation cloud optionnelle.",
  },
  localeSwitcherAria: "Langue",
  headerNav: {
    download: "Télécharger",
    menuToggle: "Basculer le menu",
    moreLabel: "Plus",
    moreMenuAria: "Autres sections",
  },
  nav: {
    why: "Pourquoi Subly",
    subscriptions: "Abonnements",
    dashboard: "Tableau de bord",
    video: "Démo",
    ai: "IA",
    sync: "Sync",
    faq: "FAQ",
  },
  hero: {
    headline: "Sachez ce que vous payez — avant le renouvellement.",
    subline:
      "Un suivi rapide des abonnements et dépenses sur votre appareil, code MIT entièrement ouvert, sans analytique ni métriques d'utilisation.",
    ctaPrimary: "Télécharger la dernière version",
    ctaSecondary: "Voir sur GitHub",
    watchDemo: "Voir la démo",
  },
  videoSection: {
    eyebrow: "Démo produit",
    title: "Subly en action",
    subtitle: "Basculez entre enregistrements bureau et téléphone.",
    fallbackLabel: "Ouvrir la vidéo dans un nouvel onglet si la lecture ne démarre pas",
    desktopLabel: "Bureau",
    desktopDescription: "Abonnements, dépenses, tableau de bord et réglages sur macOS.",
    phoneLabel: "Téléphone",
    phoneDescription: "Les mêmes parcours sur iPhone — listes, calendrier et réglages.",
  },
  platforms: {
    title: "Disponible partout où vous travaillez",
    tagline:
      "Local-first, entièrement open source et sans analytique — aucun compte cloud imposé.",
  },
  openValuesSection: {
    eyebrow: "Ouvert et honnête",
    title: "Application puissante. Code ouvert. Zéro surveillance.",
    lead:
      "La plupart des apps finance veulent vos données sur leurs serveurs — et un flux discret d'analytique à côté. Subly fait l'inverse : sources sous licence MIT que vous pouvez lire et compiler, local-first par défaut, et aucune pile de pistage. Pas de « mode confidentialité ». Pas de « nous agrégeons anonymement ». Simplement pas de métriques, pas de pubs, pas de pixels cachés.",
    pillars: [
      "Entièrement open source",
      "Pas d'analytique. Jamais.",
      "Une confiance vérifiable",
    ] as [string, string, string],
    pillarBodies: [
      "Bureau, mobile, sync et IA optionnelle — tout le produit est sur GitHub sous MIT. Auditez, forkez, publiez votre build ou contribuez un correctif. La transparence est une fonctionnalité.",
      "Pas de Google Analytics, Firebase, Mixpanel, Amplitude, Sentry session replay ni SDK publicitaires. Subly ne collecte pas de statistiques d'usage ni de télémétrie de crash vers nos serveurs. Votre usage de l'app vous appartient.",
      "Les données restent sur votre appareil. La sync utilise votre cloud si vous l'activez. Les clés API sont dans le trousseau du système. Quand le README dit « pas de pistage », vous pouvez parcourir le dépôt et le constater.",
    ] as [string, string, string],
    productHighlights: [
      "Abonnements, dépenses, tableau de bord et calendrier de renouvellement au même endroit",
      "34+ devises, 12 langues, exports CSV/JSON",
      "Assistance IA optionnelle — quatre capacités, sept fournisseurs, désactivée jusqu'à activation",
      "Fonctionne hors ligne ; aucun compte Subly requis",
    ],
    ctaLabel: "Explorer les sources sur GitHub",
  },
  whySection: {
    title: "Pourquoi les gens choisissent Subly",
    subtitle:
      "Clarté sur les factures récurrentes et les dépenses du quotidien — code ouvert, pas de pistage, et des outils qui respectent vos données.",
  },
  whyCards: [
    {
      title: "Open source, licence MIT",
      body: "Chaque fonctionnalité est dans un dépôt public. Pas de boîte noire autour de vos finances — inspectez, compilez et partagez à vos conditions.",
    },
    {
      title: "Pas de métriques ni de pistage",
      body: "Nous ne mesurons pas écrans, clics ou plantages pour nous. Vos habitudes de dépense ne deviennent jamais le tableau de bord de quelqu'un d'autre.",
    },
    {
      title: "Vos données, votre appareil",
      body: "Abonnements, dépenses et réglages restent sur votre machine ou téléphone. Vous gardez le contrôle sans compte chez nous.",
    },
    {
      title: "Sync quand vous le décidez",
      body: "Sauvegarde optionnelle via iCloud, Google Drive, Dropbox, OneDrive ou WebDAV — seulement si vous l'activez dans les réglages.",
    },
    {
      title: "12 langues",
      body: "Anglais, russe, chinois, espagnol, français, allemand, portugais, japonais, coréen, arabe, hindi et turc.",
    },
    {
      title: "Conçu pour la vraie vie",
      body: "Tableaux de bord, calendrier de renouvellement, multi-devises, rappels et exports — le tout dans une interface calme.",
    },
  ],
  featureSections: [
    {
      eyebrow: "Abonnements",
      title: "Tous les prélèvements récurrents au même endroit",
      body: "Ajoutez streaming, logiciels, fournisseurs ou tout service. Facturation quotidienne, hebdomadaire, mensuelle ou annuelle — plus des cycles personnalisés là où c'est pris en charge.",
      bullets: [
        "Épingler les favoris et basculer entre listes compactes ou étendues",
        "Grouper par catégorie ou moyen de paiement ; actions groupées pour ranger",
        "Historique complet, renouvellement auto, retards, notes et liens",
      ],
      alt: "Liste d'abonnements Subly",
    },
    {
      eyebrow: "Dépenses",
      title: "Dépenses ponctuelles sans tableur",
      body: "Enregistrez achats avec montants, catégories, tags, notes et liens optionnels. Filtrez par période, catégorie ou tags pour la vue d'ensemble.",
      bullets: [
        "Exporter les dépenses en CSV pour tableurs",
        "Lier une dépense à un abonnement quand cela colle à votre flux",
        "Mêmes catégories, tags et moyens de paiement que pour les abonnements",
      ],
      alt: "Liste de dépenses Subly",
    },
    {
      eyebrow: "Tableau de bord",
      title: "La vue d'ensemble en un coup d'œil",
      body: "Choisissez quels blocs apparaissent et dans quel ordre. Des alertes de retard aux prévisions et coût vie des abonnements — votre écran d'accueil s'adapte à vous.",
      bullets: [
        "Paiements en retard et à venir, stats, progression du budget mensuel",
        "Tendances de dépenses, prévisions, coûts vie, moyennes par catégorie",
        "Graphiques de répartition, top dépenses, jours de la semaine, tags et historique des taux",
      ],
      alt: "Vue d'ensemble du tableau de bord Subly",
    },
    {
      eyebrow: "Calendrier",
      title: "Planifier autour des salaires et factures",
      body: "Les prélèvements à venir apparaissent sur une grille calendrier. Consultez les totaux mensuels et passez d'une date à l'abonnement concerné.",
      bullets: [
        "Nombre de paiements et montant total du mois",
        "Vue claire de ce qui tombe avant le prochain salaire",
        "Accès aux détails de l'abonnement en une étape",
      ],
      alt: "Vue mensuelle du calendrier Subly",
    },
    {
      eyebrow: "Argent et alertes",
      title: "Devises et rappels qui suivent le rythme",
      body: "Suivez les dépenses en plusieurs devises, actualisez les taux selon un planning et soyez notifié avant le débit.",
      bullets: [
        "34+ devises avec place pour des entrées personnalisées",
        "Mise à jour automatique des taux depuis le fournisseur choisi",
        "Notifications système pour paiements à venir et en retard",
        "Rappels Telegram optionnels ; titres et planning personnalisables",
      ],
      alt: "Écran des taux de change Subly",
    },
  ],
  dashboardWidgets: [
    "Paiements en retard et à venir",
    "Stats et progression du budget",
    "Tendances et prévisions de dépenses",
    "Coût vie des abonnements",
    "Graphiques par catégorie",
    "Top dépenses et comparaison mensuelle",
    "Dépenses par tags ou jour de la semaine",
  ],
  aiSection: {
    eyebrow: "Assistance IA",
    title: "IA optionnelle — quatre façons d'ajouter des données plus vite",
    capabilitiesTitle: "Capacités",
    capabilitiesIntro:
      "Un point d'assistance sur abonnements et dépenses — activez chaque capacité séparément dans les réglages.",
    inputMethodsTitle: "Comment envoyer une entrée",
    lead:
      "La couche IA de Subly est entièrement optionnelle : désactivée tant que vous n'activez pas l'interrupteur principal dans les réglages. Vous choisissez le fournisseur, stockez la clé API dans le trousseau du système et n'activez que les capacités voulues. Rien n'est envoyé à un modèle tant que vous n'ouvrez pas le dialogue d'assistance et n'envoyez pas.",
    principles: [
      {
        title: "Désactivée par défaut",
        body: "Pas d'appels en arrière-plan ni d'envois silencieux. Activez l'IA seulement quand vous en avez besoin.",
      },
      {
        title: "Apportez votre propre clé",
        body: "API cloud payantes ou Ollama / LM Studio local — les identifiants ne restent jamais dans la base de données.",
      },
      {
        title: "Aperçu avant enregistrement",
        body: "Chaque abonnement, dépense ou ligne importée est montré pour validation. Rien n'entre dans votre registre sans confirmation.",
      },
    ],
    features: [
      {
        title: "Ajout rapide d'abonnements",
        body: "Décrivez un service en langage naturel — prix, cycle, catégorie, moyen de paiement et tags sont préremplis depuis votre catalogue.",
        example: "« Telegram Premium 300 ₽ par mois, carte »",
      },
      {
        title: "Ajout rapide de dépenses",
        body: "Même dialogue pour dépenses ponctuelles : montant, date, commerçant, catégories et tags depuis une courte phrase.",
        example: "« Café Starbucks 5,40 EUR hier »",
      },
      {
        title: "Scan de ticket et document",
        body: "Pointez un modèle vision sur une photo ou PDF. Lignes, totaux, devise, commerçant et date reviennent comme brouillon de dépense.",
        example: "Tickets PNG, JPEG, WebP, HEIC ou PDF",
      },
      {
        title: "Import de relevé bancaire",
        body: "Import en masse de CSV, XLSX, PDF ou JSON. Analyse heuristique plus secours IA, avec tableau ligne par ligne avant enregistrement.",
        example: "Exports carte et banque courants",
      },
    ],
    inputMethods: [
      { label: "Saisir une description", detail: "⌘/Ctrl + Entrée pour envoyer" },
      { label: "Déposer ou choisir des fichiers", detail: "Glisser-déposer sur bureau" },
      { label: "Coller des captures", detail: "Dans le champ de texte" },
    ],
    providersTitle: "Fournisseurs pris en charge",
    settingsBullets: [
      "Interrupteur principal plus bascules individuelles par capacité",
      "Préréglages de modèles recommandés et nom de modèle personnalisé",
      "Tester la connexion avant de s'y fier",
      "Badge Vision quand le modèle prend en charge l'OCR de tickets",
    ],
    privacyNote:
      "Les requêtes vont uniquement au fournisseur que vous avez configuré. Subly n'exploite pas sa propre LLM et n'enregistre pas vos prompts sur nos serveurs.",
    alt: "Panneau des réglages IA Subly",
  },
  syncSection: {
    eyebrow: "Sync et sauvegarde",
    title: "Votre sauvegarde, votre cloud",
    body: "Pour plusieurs appareils, activez le fournisseur que vous utilisez déjà. Exportez et importez des sauvegardes complètes à tout moment.",
    bullets: [
      "Charges sync chiffrées entre vos appareils",
      "Exporter ou importer des sauvegardes `.subly`, plus JSON et CSV",
      "Membres du foyer, catégories et tags voyagent avec vos données",
    ],
  },
  privacySection: {
    eyebrow: "Confidentialité",
    title: "Privé par défaut",
    body: "Pas d'analytique, pas de SDK pub, pas de pipelines « usage anonyme » — et un code ouvert pour ne pas nous croire sur parole.",
    bullets: [
      "Données principales stockées localement sur chaque appareil",
      "Pas de métriques in-app, rapports de crash ni pistage tiers vers les serveurs Subly",
      "Clés API, mots de passe sync et jetons Telegram dans le trousseau du système — pas en clair dans la base",
      "IA et fonctions cloud sont optionnelles ; consultez notre politique de confidentialité pour les détails",
    ],
    policyLinkLabel: "Privacy Policy (EN)",
  },
  customizeSection: {
    eyebrow: "À votre image",
    title: "Thèmes, tags et foyer",
    body: "Plus de 15 thèmes de couleur, icônes de catégorie personnalisées, moyens de paiement, tags, progression du budget mensuel et raccourcis barre système sur bureau.",
    bullets: [
      "Membres du foyer pour attribuer correctement les dépenses",
      "Barre système avec paiements à venir et ouverture rapide",
      "Réduire dans la barre, démarrage auto au boot (bureau)",
    ],
    alt: "Apparence et réglages Subly",
  },
  deviceGalleryMeta: {
    title: "Dans l'application",
    subtitle: "Les mêmes parcours essentiels sur bureau, téléphone et tablette.",
  },
  shotLabels: {
    dashboard: "Tableau de bord",
    subscriptions: "Abonnements",
    expenses: "Dépenses",
    calendar: "Calendrier",
    currencies: "Devises",
    settings: "Réglages",
  },
  deviceGalleries: [
    {
      description:
        "Expérience bureau complète avec widgets du tableau de bord, barre système et exports — pour macOS, Windows et Linux.",
    },
    {
      description:
        "Suivez abonnements et dépenses en déplacement avec les mêmes catégories, tags et calendrier de renouvellement.",
    },
    {
      description:
        "Plus d'espace pour listes et calendrier mensuel sur iPad et tablettes Android sans sacrifier la confidentialité local-first.",
    },
  ],
  appStore: {
    title: "Également sur l'App Store",
    body: "Subly est disponible sur Mac et iPhone/iPad via l'App Store. Téléchargement payant — votre achat soutient le développement continu. Les builds bureau pour macOS, Windows et Linux restent gratuits sur GitHub Releases.",
    ctaLabel: "Voir sur l'App Store",
  },
  faqSection: {
    title: "FAQ",
  },
  faqItems: [
    {
      q: "Subly est-il gratuit ?",
      a: "Subly est open source sous licence MIT. Les builds GitHub Releases sont gratuits. Les boutiques peuvent appliquer des frais de plateforme.",
    },
    {
      q: "Fonctionne-t-il hors ligne ?",
      a: "Oui. Abonnements, dépenses, tableau de bord et calendrier fonctionnent entièrement hors ligne. Sync, taux, IA et Telegram nécessitent le réseau quand vous les utilisez.",
    },
    {
      q: "Ai-je besoin de l'IA ?",
      a: "Non. L'IA est optionnelle et désactivée par défaut. Si activée, ajout rapide d'abonnements ou dépenses, scan de tickets ou import de relevés — chaque capacité a son interrupteur, vous choisissez parmi sept fournisseurs avec votre clé API.",
    },
    {
      q: "La sync cloud est-elle sûre ?",
      a: "La sync utilise votre propre compte cloud (iCloud, Drive, Dropbox, OneDrive ou WebDAV). Les identifiants sont dans le trousseau système. Activez la sync seulement si vous voulez une sauvegarde multi-appareils.",
    },
    {
      q: "Quelles plateformes sont prises en charge ?",
      a: "macOS (Universal), Windows, Linux, iOS et Android. Installateurs bureau sur GitHub Releases ; mobile là où publié en boutique.",
    },
    {
      q: "Comment sauvegarder mes données ?",
      a: "Réglages → Données : exportez un fichier `.subly`, JSON ou CSV. Importez les mêmes formats en changeant d'appareil.",
    },
    {
      q: "Où sont stockées mes données ?",
      a: "Sur votre appareil par défaut. Nous n'exploitons pas de cloud central Subly contenant vos abonnements.",
    },
    {
      q: "Subly utilise-t-il de l'analytique ou du pistage ?",
      a: "Non. Pas de SDK analytique, réseaux pub ni métriques d'usage vers nous. Les fonctions optionnelles (sync, IA, Telegram, taux) ne parlent qu'aux services que vous configurez.",
    },
    {
      q: "Puis-je lire le code source ?",
      a: "Oui. Subly est entièrement open source sur GitHub sous MIT — frontend, backend et builds mobile. Vous pouvez vérifier les affirmations de confidentialité vous-même.",
    },
    {
      q: "Puis-je essayer avant d'installer ?",
      a: "Regardez les vidéos démo bureau ou téléphone sur cette page, ou la galerie de captures ci-dessous.",
    },
  ],
  downloadSection: {
    title: "Prêt à reprendre le contrôle ?",
    body: "Builds bureau gratuits sur GitHub Releases — choisissez votre plateforme ci-dessous. Vous préférez l'App Store sur appareils Apple ? Cela soutient le développement.",
    ctaPrimary: "Télécharger la dernière version",
    githubHeading: "Gratuit sur GitHub",
    ctaSecondary: "Étoiler sur GitHub",
    ctaDonate: "Soutenir le développement",
    ctaVideo: "Voir la démo",
    changelogLabel: "Changelog",
  },
  footer: {
    tagline: "Subly — suivi d'abonnements et de dépenses",
    license: "Licence MIT",
    donateLabel: "Faire un don",
    releasesLabel: "Releases",
  },
};

export { frStrings };

export const frContent = buildSiteContent("fr", frStrings);
