import type { SiteContent } from "./types";

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

const siteMeta = {
  title: "Subly — учёт подписок и расходов",
  description:
    "Открытый трекер подписок и трат. Данные на устройстве, без аналитики и трекинга, опциональная синхронизация.",
  siteUrl: "https://s00d.github.io/subly/ru/",
  canonicalUrl: "https://s00d.github.io/subly/ru/",
  locale: "ru_RU",
  softwareVersion: "1.0.4",
  applicationCategory: "FinanceApplication",
  operatingSystems: ["macOS", "Windows", "Linux", "iOS", "Android"],
  ...sharedUrls,
};

const localeSwitcher = {
  ariaLabel: "Язык",
  options: [
    { id: "en" as const, label: "English" },
    { id: "ru" as const, label: "Русский" },
  ],
};

const headerNav = {
  download: "Скачать",
  menuToggle: "Меню",
  moreLabel: "Ещё",
  moreMenuAria: "Другие разделы",
};

const navPrimaryLinks = [
  { id: "why", label: "Зачем Subly", icon: "heart-outline" },
  { id: "subscriptions", label: "Подписки", icon: "repeat" },
  { id: "dashboard", label: "Дашборд", icon: "view-dashboard-outline" },
  { id: "video", label: "Демо", icon: "play-circle-outline" },
] as const;

const navMoreLinks = [
  { id: "ai", label: "ИИ", icon: "robot-outline" },
  { id: "sync", label: "Синхронизация", icon: "cloud-sync-outline" },
  { id: "faq", label: "FAQ", icon: "help-circle-outline" },
] as const;

const navLinks = [...navPrimaryLinks, ...navMoreLinks] as const;

const hero = {
  headline: "Знайте, за что платите — до автопродления.",
  subline:
    "Быстрый учёт подписок и расходов на вашем устройстве, полностью открытый код MIT и никакой аналитики или метрик использования.",
  ctaPrimary: "Скачать последний релиз",
  ctaSecondary: "Открыть на GitHub",
  watchDemo: "Смотреть демо",
};

const videoSection = {
  eyebrow: "Демо продукта",
  title: "Subly в деле",
  subtitle: "Видео со звуком — переключайтесь между десктопом и телефоном.",
  fallbackLabel: "Открыть видео в новой вкладке, если воспроизведение не началось",
  demos: [
    {
      id: "desktop",
      label: "Десктоп",
      icon: "microsoft-windows",
      description:
        "Подписки, расходы, дашборд и настройки на macOS — озвученный обзор.",
      url: sharedUrls.videoDesktopUrl,
      poster: "01_dashboard_overview.png",
      aspect: "video" as const,
    },
    {
      id: "phone",
      label: "Телефон",
      icon: "cellphone",
      description:
        "Те же сценарии на iPhone — списки, календарь и настройки в мобильном обзоре со звуком.",
      url: sharedUrls.videoMobileUrl,
      poster: "phone/01_dashboard_overview.png",
      aspect: "portrait" as const,
    },
  ],
};

const platforms = {
  title: "Везде, где вы работаете",
  items: [
    { name: "macOS", icon: "apple" },
    { name: "Windows", icon: "microsoft-windows" },
    { name: "Linux", icon: "linux" },
    { name: "iOS", icon: "apple-ios" },
    { name: "Android", icon: "android" },
  ],
  tagline:
    "Local-first, открытый исходный код и ноль аналитики — облачный аккаунт не обязателен.",
};

const openValuesSection = {
  eyebrow: "Открыто и честно",
  icon: "github",
  title: "Мощное приложение. Открытый код. Без слежки.",
  lead:
    "Многие финансовые сервисы хотят ваши данные на своих серверах и тихо собирают аналитику. Subly — наоборот: исходники под MIT, данные по умолчанию на устройстве, без метрик, рекламы и скрытых пикселей.",
  pillars: [
    {
      icon: "github",
      title: "Полностью open source",
      body: "Десктоп, мобильные клиенты, синхронизация и опциональный ИИ — весь продукт на GitHub под MIT. Можно проверить, форкнуть и собрать свою версию.",
    },
    {
      icon: "eye-off-outline",
      title: "Никакой аналитики",
      body: "Нет Google Analytics, Firebase, Mixpanel и рекламных SDK. Subly не шлёт статистику использования на наши серверы.",
    },
    {
      icon: "shield-check-outline",
      title: "Доверие, которое проверяется",
      body: "Данные на устройстве. Синхронизация — в ваше облако, если включите. Ключи API — в связке ключей ОС. Утверждения можно проверить в репозитории.",
    },
  ],
  productHighlights: [
    "Подписки, расходы, дашборд и календарь продлений в одном приложении",
    "34+ валют, 12 языков интерфейса, экспорт CSV/JSON",
    "Опциональный ИИ — четыре сценария, семь провайдеров, выключен по умолчанию",
    "Работает офлайн; аккаунт Subly не нужен",
  ],
  ctaLabel: "Исходники на GitHub",
};

const whySection = {
  title: "Почему выбирают Subly",
  subtitle:
    "Ясность по регулярным платежам и повседневным тратам — открытый код, без трекинга и с уважением к вашим данным.",
};

const whyCards = [
  {
    icon: "github",
    title: "Open source, лицензия MIT",
    body: "Весь функционал в публичном репозитории. Никакого «чёрного ящика» вокруг ваших финансов.",
  },
  {
    icon: "eye-off-outline",
    title: "Без метрик и трекинга",
    body: "Мы не считаем экраны и клики для себя. Ваши траты не становятся чужим дашбордом.",
  },
  {
    icon: "cellphone-link-off",
    title: "Ваши данные — на устройстве",
    body: "Подписки, расходы и настройки хранятся локально. Регистрация на наших серверах не требуется.",
  },
  {
    icon: "cloud-sync-outline",
    title: "Синхронизация по желанию",
    body: "iCloud, Google Drive, Dropbox, OneDrive или WebDAV — только если включите в настройках.",
  },
  {
    icon: "translate",
    title: "12 языков",
    body: "Английский, русский, китайский, испанский, французский, немецкий, португальский, японский, корейский, арабский, хинди и турецкий.",
  },
  {
    icon: "view-dashboard-outline",
    title: "Под реальную жизнь",
    body: "Дашборды, календарь продлений, мультивалютность, напоминания и экспорт — в спокойном интерфейсе.",
  },
] as const;

const featureSections = [
  {
    id: "subscriptions",
    eyebrow: "Подписки",
    icon: "repeat",
    title: "Все регулярные платежи в одном месте",
    body: "Стриминг, софт, связь, коммуналка — дневной, недельный, месячный или годовой цикл и свои периоды, где поддерживается.",
    bullets: [
      "Избранное, компактный и развёрнутый список",
      "Группировка по категории или способу оплаты, пакетные действия",
      "История платежей, автопродление, просрочки, заметки и ссылки",
    ],
    screenshot: "02_subscriptions_list.png",
    alt: "Список подписок Subly",
  },
  {
    id: "expenses",
    eyebrow: "Расходы",
    icon: "cash-multiple",
    title: "Разовые траты без таблиц",
    body: "Сумма, категория, теги, заметки и ссылки. Фильтры по периоду, категории и тегам.",
    bullets: [
      "Экспорт расходов в CSV",
      "Связь расхода с подпиской, когда удобно",
      "Те же категории, теги и способы оплаты, что у подписок",
    ],
    screenshot: "03_expenses_list.png",
    alt: "Список расходов Subly",
  },
  {
    id: "dashboard",
    eyebrow: "Дашборд",
    icon: "view-dashboard-outline",
    title: "Картина целиком",
    body: "Выберите блоки и порядок. Просрочки, прогнозы, lifetime cost подписок — главный экран под вас.",
    bullets: [
      "Просроченные и ближайшие платежи, статистика, бюджет месяца",
      "Тренды, прогнозы, средние по категориям",
      "Графики: категории, топ трат, дни недели, теги, курсы",
    ],
    screenshot: "01_dashboard_overview.png",
    alt: "Обзор дашборда Subly",
  },
  {
    id: "calendar",
    eyebrow: "Календарь",
    icon: "calendar-month-outline",
    title: "Планируйте вокруг зарплаты и счетов",
    body: "Предстоящие списания на сетке календаря. Итоги месяца и переход к подписке с даты.",
    bullets: [
      "Количество и сумма платежей за месяц",
      "Видно, что спишется до следующей зарплаты",
      "Переход к карточке подписки в один тап",
    ],
    screenshot: "04_calendar_month.png",
    alt: "Календарь Subly",
  },
  {
    id: "currencies",
    eyebrow: "Деньги и напоминания",
    icon: "currency-usd",
    title: "Валюты и уведомления",
    body: "Много валют, автообновление курсов и оповещения до списания.",
    bullets: [
      "34+ валют и свои записи",
      "Курсы по выбранному провайдеру",
      "Системные уведомления о платежах",
      "Опционально Telegram; свой текст и расписание",
    ],
    screenshot: "05_rates_screen.png",
    alt: "Экран курсов Subly",
  },
] as const;

const dashboardWidgets = [
  "Просроченные и ближайшие платежи",
  "Статистика и прогресс бюджета",
  "Тренды и прогнозы",
  "Lifetime cost подписок",
  "Диаграммы по категориям",
  "Топ расходов и сравнение месяцев",
  "Расходы по тегам и дням недели",
] as const;

const aiSection = {
  eyebrow: "ИИ-помощник",
  icon: "robot-outline",
  title: "Опциональный ИИ — четыре способа быстрее вносить данные",
  capabilitiesTitle: "Возможности",
  capabilitiesIntro:
    "Одна кнопка помощи на экранах подписок и расходов — каждую функцию можно включить отдельно в настройках.",
  inputMethodsTitle: "Как передать данные",
  lead:
    "ИИ в Subly полностью по желанию: выключен, пока не включите главный переключатель. Выбираете провайдера, ключ хранится в связке ключей ОС, активны только нужные сценарии. Запрос уходит в модель только когда вы открыли диалог и отправили данные.",
  principles: [
    {
      icon: "toggle-switch-off-outline",
      title: "Выключен по умолчанию",
      body: "Нет фоновых вызовов и тихих загрузок. Включайте ИИ только когда нужен.",
    },
    {
      icon: "key",
      title: "Свой ключ",
      body: "Облачный API или локальный Ollama / LM Studio — секреты не лежат в базе приложения.",
    },
    {
      icon: "clipboard-check-outline",
      title: "Превью перед сохранением",
      body: "Каждая подписка, расход или строка импорта показывается на проверку. Без подтверждения в учёт не попадает.",
    },
  ],
  features: [
    {
      icon: "repeat",
      title: "Быстрое добавление подписки",
      body: "Опишите сервис текстом — цена, цикл, категория, способ оплаты и теги подставятся из вашего справочника.",
      example: "«Telegram Premium 300 ₽ в месяц, карта»",
    },
    {
      icon: "cash-multiple",
      title: "Быстрое добавление расхода",
      body: "Тот же диалог для разовой траты: сумма, дата, магазин, категории и теги из короткой фразы.",
      example: "«Кофе Starbucks 5.40 EUR вчера»",
    },
    {
      icon: "receipt",
      title: "Скан чека или документа",
      body: "Vision-модель по фото или PDF: позиции, итог, валюта, магазин и дата — черновик расхода.",
      example: "PNG, JPEG, WebP, HEIC или PDF",
    },
    {
      icon: "file-document-multiple-outline",
      title: "Импорт выписки",
      body: "Массовый импорт CSV, XLSX, PDF или JSON. Эвристики и ИИ, таблица строк перед сохранением.",
      example: "Типичные экспорты банков и карт",
    },
  ],
  inputMethods: [
    { icon: "keyboard-outline", label: "Текстом", detail: "⌘/Ctrl + Enter — отправить" },
    { icon: "image-plus-outline", label: "Файл или фото", detail: "Drag-and-drop на десктопе" },
    { icon: "content-paste", label: "Вставка скриншота", detail: "В поле ввода" },
  ],
  providersTitle: "Поддерживаемые провайдеры",
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
    "Главный переключатель и отдельные тумблеры для каждой функции",
    "Рекомендуемые модели и своё имя модели",
    "Проверка соединения перед работой",
    "Метка Vision, если модель умеет OCR чеков",
  ],
  privacyNote:
    "Запросы идут только к выбранному провайдеру. У Subly нет своей LLM и мы не логируем ваши промпты на серверах.",
  screenshot: "06_settings_overview.png",
  alt: "Настройки ИИ в Subly",
};

const syncSection = {
  eyebrow: "Синхронизация и бэкап",
  icon: "backup-restore",
  title: "Ваш бэкап — ваше облако",
  body: "Нужны несколько устройств — включите привычный провайдер. Полный экспорт и импорт в любой момент.",
  providers: [
    { id: "icloud", label: "iCloud", icon: "apple-icloud", brandIcon: null },
    { id: "google-drive", label: "Google Drive", icon: "google-drive", brandIcon: "google-drive.svg" },
    { id: "dropbox", label: "Dropbox", icon: "dropbox", brandIcon: "dropbox.svg" },
    { id: "onedrive", label: "OneDrive", icon: "microsoft-onedrive", brandIcon: "onedrive.svg" },
    { id: "webdav", label: "WebDAV", icon: "web", brandIcon: null },
  ],
  bullets: [
    "Шифрованные payload между вашими устройствами",
    "Экспорт и импорт `.subly`, JSON и CSV",
    "Члены семьи, категории и теги переезжают с данными",
  ],
};

const privacySection = {
  eyebrow: "Конфиденциальность",
  icon: "shield-lock-outline",
  title: "Приватность по умолчанию",
  body: "Без аналитики, рекламных SDK и «анонимной статистики» — плюс открытый код, чтобы не верить на слово.",
  bullets: [
    "Основные данные локально на каждом устройстве",
    "Нет in-app метрик и crash-репортов на серверы Subly",
    "Ключи API, пароли синка и токены Telegram — в связке ключей ОС",
    "ИИ и облако — только по желанию; политика на английском",
  ],
  policyLinkLabel: "Privacy Policy (EN)",
};

const customizeSection = {
  eyebrow: "Настройте под себя",
  icon: "palette-outline",
  title: "Темы, теги и семья",
  body: "15+ цветовых тем, иконки категорий, способы оплаты, теги, прогресс бюджета и трей на десктопе.",
  bullets: [
    "Члены семьи для корректной атрибуции трат",
    "Трей с ближайшими платежами и быстрым открытием",
    "Сворачивание в трей, автозапуск (десктоп)",
  ],
  screenshot: "06_settings_overview.png",
  alt: "Внешний вид и настройки Subly",
};

const deviceGalleryMeta = {
  title: "Внутри приложения",
  subtitle: "Одни и те же сценарии на десктопе, телефоне и планшете.",
};

const desktopShots = [
  { file: "01_dashboard_overview.png", label: "Дашборд" },
  { file: "02_subscriptions_list.png", label: "Подписки" },
  { file: "03_expenses_list.png", label: "Расходы" },
  { file: "04_calendar_month.png", label: "Календарь" },
  { file: "05_rates_screen.png", label: "Валюты" },
  { file: "06_settings_overview.png", label: "Настройки" },
] as const;

const mobileShots = [
  { file: "phone/01_dashboard_overview.png", label: "Дашборд" },
  { file: "phone/02_subscriptions_list.png", label: "Подписки" },
  { file: "phone/03_expenses_list.png", label: "Расходы" },
  { file: "phone/04_calendar_month.png", label: "Календарь" },
  { file: "phone/05_rates_screen.png", label: "Валюты" },
  { file: "phone/06_settings_overview.png", label: "Настройки" },
] as const;

const tabletShots = [
  { file: "tablet/01_dashboard_overview.png", label: "Дашборд" },
  { file: "tablet/02_subscriptions_list.png", label: "Подписки" },
  { file: "tablet/03_expenses_list.png", label: "Расходы" },
  { file: "tablet/04_calendar_month.png", label: "Календарь" },
  { file: "tablet/05_rates_screen.png", label: "Валюты" },
  { file: "tablet/06_settings_overview.png", label: "Настройки" },
] as const;

const deviceGalleries = [
  {
    id: "desktop",
    label: "macOS",
    icon: "apple",
    description:
      "Полный десктоп: виджеты дашборда, трей, экспорт — macOS, Windows и Linux.",
    shots: desktopShots,
  },
  {
    id: "phone",
    label: "iPhone",
    icon: "cellphone",
    description: "Подписки и расходы в дороге — те же категории, теги и календарь.",
    shots: mobileShots,
  },
  {
    id: "tablet",
    label: "Планшет",
    icon: "tablet",
    description: "Больше места для списков и календаря на iPad и Android-планшетах.",
    shots: tabletShots,
  },
] as const;

const appStore = {
  url: sharedUrls.appStoreUrl,
  title: "Также в App Store",
  body: "Subly в Mac App Store и для iPhone/iPad. Платная загрузка поддерживает разработку. Сборки macOS, Windows и Linux на GitHub Releases остаются бесплатными.",
  platforms: ["macOS", "iOS", "iPadOS"],
  ctaLabel: "Открыть в App Store",
};

const faqSection = {
  title: "Вопросы и ответы",
};

const faqItems = [
  {
    q: "Subly бесплатный?",
    a: "Исходный код под MIT. Сборки с GitHub Releases бесплатны. В сторах могут быть комиссии площадки.",
  },
  {
    q: "Работает офлайн?",
    a: "Да: подписки, расходы, дашборд и календарь офлайн. Синк, курсы, ИИ и Telegram нуждаются в сети, когда вы их используете.",
  },
  {
    q: "Нужен ли ИИ?",
    a: "Нет. ИИ выключен по умолчанию. При включении — быстрый ввод, чеки, выписки; отдельные тумблеры и свой ключ у семи провайдеров.",
  },
  {
    q: "Безопасна ли синхронизация?",
    a: "Используется ваше облако (iCloud, Drive, Dropbox, OneDrive, WebDAV). Учётные данные — в связке ключей ОС.",
  },
  {
    q: "Какие платформы?",
    a: "macOS, Windows, Linux, iOS, Android. Десктоп — GitHub Releases; мобильные — там, где опубликовано в сторах.",
  },
  {
    q: "Как сделать бэкап?",
    a: "Настройки → Данные: экспорт `.subly`, JSON или CSV. Импорт тех же форматов на новом устройстве.",
  },
  {
    q: "Где хранятся данные?",
    a: "На устройстве по умолчанию. Центрального облака Subly с вашими подписками нет.",
  },
  {
    q: "Есть ли аналитика?",
    a: "Нет SDK аналитики, рекламы и метрик на наши серверы. Синк, ИИ, Telegram и курсы — только к сервисам, которые вы настроите.",
  },
  {
    q: "Можно ли посмотреть исходники?",
    a: "Да, весь проект на GitHub под MIT — можно проверить заявления о приватности самим.",
  },
  {
    q: "Можно попробовать до установки?",
    a: "На этой странице — видео десктопа и телефона со звуком и галерея скриншотов.",
  },
] as const;

const downloadSection = {
  title: "Готовы взять расходы под контроль?",
  body: "Бесплатные десктоп-сборки на GitHub — выберите платформу ниже. На Apple-устройствах можно поддержать разработку через App Store.",
  ctaPrimary: "Скачать последний релиз",
  githubHeading: "Бесплатно на GitHub",
  ctaSecondary: "Звезда на GitHub",
  ctaDonate: "Поддержать разработку",
  ctaVideo: "Смотреть демо",
  changelogLabel: "Changelog",
};

const pageTitles = {
  privacy: "Privacy Policy — Subly",
  changelog: "Changelog — Subly",
};

const footer = {
  tagline: "Subly — учёт подписок и расходов",
  license: "Лицензия MIT",
  donateLabel: "Донат",
  links: [
    { label: "GitHub", href: sharedUrls.repoUrl, icon: "github", external: true },
    { label: "Релизы", href: sharedUrls.releasesUrl, icon: "download", external: true },
    { label: "Privacy", to: "/privacy", icon: "shield-check-outline", external: false },
    { label: "Changelog", to: "/changelog", icon: "history", external: false },
  ],
} as const;

export const ruContent: SiteContent = {
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
