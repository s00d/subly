import { buildSiteContent } from "./buildSiteContent";
import type { SiteLocaleStrings } from "./buildSiteContent";

const esStrings: SiteLocaleStrings = {
  site: {
    title: "Subly — Control de suscripciones y gastos",
    description:
      "Rastreador open source de suscripciones y gastos. Local-first, sin analíticas ni seguimiento, sincronización en la nube opcional.",
  },
  localeSwitcherAria: "Idioma",
  headerNav: {
    download: "Descargar",
    menuToggle: "Alternar menú",
    moreLabel: "Más",
    moreMenuAria: "Más secciones",
  },
  nav: {
    why: "Por qué Subly",
    subscriptions: "Suscripciones",
    dashboard: "Panel",
    video: "Demo",
    ai: "IA",
    sync: "Sync",
    faq: "FAQ",
  },
  hero: {
    headline: "Sepa en qué gasta — antes de que se renueve.",
    subline:
      "Un rastreador rápido de suscripciones y gastos en su dispositivo, código MIT totalmente abierto y sin analíticas ni métricas de uso.",
    ctaPrimary: "Descargar última versión",
    ctaSecondary: "Ver en GitHub",
    watchDemo: "Ver demo",
  },
  videoSection: {
    eyebrow: "Demo del producto",
    title: "Subly en acción",
    subtitle: "Cambie entre grabaciones de escritorio y móvil.",
    fallbackLabel: "Abrir el vídeo en una pestaña nueva si no se reproduce",
    desktopLabel: "Escritorio",
    desktopDescription: "Suscripciones, gastos, panel y ajustes en macOS.",
    phoneLabel: "Móvil",
    phoneDescription: "Los mismos flujos en iPhone — listas, calendario y ajustes.",
  },
  platforms: {
    title: "Disponible donde trabaje",
    tagline:
      "Local-first, código abierto completo y sin analíticas — sin cuenta en la nube obligatoria.",
  },
  openValuesSection: {
    eyebrow: "Abierto y honesto",
    title: "App potente. Código abierto. Cero vigilancia.",
    lead:
      "La mayoría de apps financieras quieren sus datos en sus servidores — y un hilo silencioso de analíticas al lado. Subly es lo contrario: código con licencia MIT que puede leer y compilar usted mismo, local-first por defecto y sin pila de seguimiento de ningún tipo. No es «modo privacidad». No es «agregamos de forma anónima». Simplemente sin métricas, sin anuncios, sin píxeles ocultos.",
    pillars: [
      "Código abierto completo",
      "Sin analíticas. Nunca.",
      "Confianza que puede verificar",
    ] as [string, string, string],
    pillarBodies: [
      "Escritorio, móvil, sync e IA opcional — todo el producto está en GitHub bajo MIT. Audítelo, forquéelo, publique su build o contribuya un arreglo. La transparencia es una función.",
      "Sin Google Analytics, Firebase, Mixpanel, Amplitude, Sentry session replay ni SDKs de anuncios. Subly no recopila estadísticas de uso ni telemetría de fallos a nuestros servidores. Cómo usa la app es solo su asunto.",
      "Los datos viven en su dispositivo. La sync usa su nube si la activa. Las claves API están en el llavero del SO. Cuando el README dice «sin seguimiento», puede buscar en el repo y comprobarlo.",
    ] as [string, string, string],
    productHighlights: [
      "Suscripciones, gastos, panel y calendario de renovaciones en un solo lugar",
      "Más de 34 divisas, 12 idiomas, exportaciones CSV/JSON",
      "Asistencia IA opcional — cuatro capacidades, siete proveedores, desactivada hasta que la active",
      "Funciona sin conexión; no requiere cuenta Subly",
    ],
    ctaLabel: "Explorar el código en GitHub",
  },
  whySection: {
    title: "Por qué la gente elige Subly",
    subtitle:
      "Claridad en facturas recurrentes y gastos diarios — código abierto, sin seguimiento y herramientas que respetan sus datos.",
  },
  whyCards: [
    {
      title: "Código abierto, licencia MIT",
      body: "Cada función está en un repositorio público. Sin caja negra alrededor de sus finanzas — inspeccione, compile y comparta a su manera.",
    },
    {
      title: "Sin métricas ni seguimiento",
      body: "No medimos pantallas, clics ni fallos para nosotros. Sus patrones de gasto nunca serán el panel de otro.",
    },
    {
      title: "Sus datos, su dispositivo",
      body: "Suscripciones, gastos y ajustes viven en su equipo o teléfono. Usted controla sin registrarse en nuestros servidores.",
    },
    {
      title: "Sync cuando usted quiera",
      body: "Copia de seguridad opcional vía iCloud, Google Drive, Dropbox, OneDrive o WebDAV — solo si la activa en Ajustes.",
    },
    {
      title: "12 idiomas",
      body: "Inglés, ruso, chino, español, francés, alemán, portugués, japonés, coreano, árabe, hindi y turco.",
    },
    {
      title: "Hecho para la vida real",
      body: "Paneles, calendario de renovaciones, multimoneda, recordatorios y exportaciones — todo en una interfaz tranquila.",
    },
  ],
  featureSections: [
    {
      eyebrow: "Suscripciones",
      title: "Todos los cargos recurrentes en un lugar",
      body: "Añada streaming, software, suministros o cualquier servicio. Facturación diaria, semanal, mensual o anual — más ciclos personalizados donde se admitan.",
      bullets: [
        "Fije favoritos y cambie entre listas compactas o ampliadas",
        "Agrupe por categoría o método de pago; acciones por lotes para ordenar",
        "Historial completo, renovación automática, vencidos, notas y enlaces",
      ],
      alt: "Lista de suscripciones de Subly",
    },
    {
      eyebrow: "Gastos",
      title: "Gastos puntuales sin hoja de cálculo",
      body: "Registre compras con importes, categorías, etiquetas, notas y enlaces opcionales. Filtre por periodo, categoría o etiquetas cuando necesite el panorama completo.",
      bullets: [
        "Exporte gastos a CSV para hojas de cálculo",
        "Vincule gastos a suscripciones cuando encaje en su flujo",
        "Las mismas categorías, etiquetas y métodos de pago que en suscripciones",
      ],
      alt: "Lista de gastos de Subly",
    },
    {
      eyebrow: "Panel",
      title: "La visión global de un vistazo",
      body: "Elija qué bloques aparecen y en qué orden. Desde alertas de vencidos hasta pronósticos y coste vitalicio de suscripciones — su pantalla de inicio se adapta a usted.",
      bullets: [
        "Pagos vencidos y próximos, estadísticas, progreso del presupuesto mensual",
        "Tendencias de gasto, pronósticos, costes vitalicios, medias por categoría",
        "Gráficos de desglose, top gastos, días de la semana, etiquetas e historial de tipos",
      ],
      alt: "Vista general del panel de Subly",
    },
    {
      eyebrow: "Calendario",
      title: "Planifique en torno a nóminas y facturas",
      body: "Los cargos próximos aparecen en una cuadrícula de calendario. Revise totales mensuales y salte desde una fecha al suscripción relacionada.",
      bullets: [
        "Número de pagos y importe total del mes",
        "Vista clara de lo que cae antes del próximo cobro de nómina",
        "Acceso a detalles de la suscripción en un paso",
      ],
      alt: "Vista mensual del calendario de Subly",
    },
    {
      eyebrow: "Dinero y alertas",
      title: "Divisas y recordatorios al día",
      body: "Siga gastos en muchas divisas, actualice tipos de cambio según programa y reciba avisos antes de que salga el dinero.",
      bullets: [
        "Más de 34 divisas con espacio para entradas personalizadas",
        "Actualización automática de tipos del proveedor elegido",
        "Notificaciones del sistema para pagos próximos y vencidos",
        "Recordatorios opcionales por Telegram; títulos y horario personalizables",
      ],
      alt: "Pantalla de tipos de cambio de Subly",
    },
  ],
  dashboardWidgets: [
    "Pagos vencidos y próximos",
    "Estadísticas y progreso del presupuesto",
    "Tendencias y pronósticos de gasto",
    "Coste vitalicio de suscripciones",
    "Gráficos por categoría",
    "Top gastos y comparación mensual",
    "Gastos por etiquetas o día de la semana",
  ],
  aiSection: {
    eyebrow: "Asistencia IA",
    title: "IA opcional — cuatro formas de añadir datos más rápido",
    capabilitiesTitle: "Capacidades",
    capabilitiesIntro:
      "Un punto de asistencia en suscripciones y gastos — active cada función por separado en Ajustes.",
    inputMethodsTitle: "Cómo puede enviar entrada",
    lead:
      "La capa de IA de Subly es totalmente opcional: desactivada hasta que active el interruptor principal en Ajustes. Elige el proveedor, guarda la clave API en el llavero del SO y activa solo las capacidades que quiera. Nada se envía a un modelo hasta que abra el diálogo de asistencia y envíe.",
    principles: [
      {
        title: "Desactivada por defecto",
        body: "Sin llamadas en segundo plano ni subidas silenciosas. Active la IA solo cuando la necesite.",
      },
      {
        title: "Traiga su propia clave",
        body: "APIs de nube de pago o Ollama / LM Studio local — las credenciales nunca están en la base de datos.",
      },
      {
        title: "Vista previa antes de guardar",
        body: "Cada suscripción, gasto o fila importada se muestra para revisión. Nada entra en su libro sin confirmación.",
      },
    ],
    features: [
      {
        title: "Añadir suscripciones rápido",
        body: "Describa un servicio en lenguaje natural — precio, ciclo, categoría, método de pago y etiquetas se rellenan desde su catálogo.",
        example: "«Telegram Premium 300 ₽ al mes, tarjeta»",
      },
      {
        title: "Añadir gastos rápido",
        body: "El mismo diálogo para gastos puntuales: importe, fecha, comercio, categorías y etiquetas desde una frase corta.",
        example: "«Café en Starbucks 5,40 EUR ayer»",
      },
      {
        title: "Escanear ticket y documento",
        body: "Apunte un modelo con visión a una foto o PDF. Líneas, totales, divisa, comercio y fecha vuelven como borrador de gasto.",
        example: "Tickets PNG, JPEG, WebP, HEIC o PDF",
      },
      {
        title: "Importar extracto bancario",
        body: "Importe masivo de CSV, XLSX, PDF o JSON. Análisis heurístico más respaldo IA, con tabla por fila antes de guardar.",
        example: "Exportaciones habituales de tarjeta y banco",
      },
    ],
    inputMethods: [
      { label: "Escribir una descripción", detail: "⌘/Ctrl + Intro para enviar" },
      { label: "Soltar o elegir archivos", detail: "Arrastrar y soltar en escritorio" },
      { label: "Pegar capturas", detail: "En el campo de texto" },
    ],
    providersTitle: "Proveedores compatibles",
    settingsBullets: [
      "Interruptor principal más switches individuales por capacidad",
      "Modelos recomendados y nombre de modelo personalizado",
      "Probar conexión antes de confiar en ella",
      "Insignia Vision cuando el modelo admite OCR de tickets",
    ],
    privacyNote:
      "Las peticiones van solo al proveedor que configuró. Subly no opera su propia LLM ni registra sus prompts en nuestros servidores.",
    alt: "Panel de ajustes de IA de Subly",
  },
  syncSection: {
    eyebrow: "Sync y copia de seguridad",
    title: "Su copia, su nube",
    body: "Si necesita varios dispositivos, active el proveedor que ya usa. Exporte e importe copias completas cuando quiera.",
    bullets: [
      "Cargas de sync cifradas entre sus dispositivos",
      "Exportar o importar copias `.subly`, además de JSON y CSV",
      "Miembros del hogar, categorías y etiquetas viajan con sus datos",
    ],
  },
  privacySection: {
    eyebrow: "Privacidad",
    title: "Privado por defecto",
    body: "Sin analíticas, SDKs de anuncios ni tuberías de «uso anónimo» — y código abierto para no creernos a ciegas.",
    bullets: [
      "Datos principales almacenados localmente en cada dispositivo",
      "Sin métricas in-app, informes de fallos ni seguimiento de terceros a servidores Subly",
      "Claves API, contraseñas de sync y tokens de Telegram en el llavero del SO — no en texto plano en la base de datos",
      "IA y funciones en la nube son opcionales; lea nuestra política de privacidad para detalles",
    ],
    policyLinkLabel: "Privacy Policy (EN)",
  },
  customizeSection: {
    eyebrow: "Hágalo suyo",
    title: "Temas, etiquetas y hogar",
    body: "Más de 15 temas de color, iconos de categoría personalizados, métodos de pago, etiquetas, progreso del presupuesto mensual y accesos directos en la bandeja del sistema en escritorio.",
    bullets: [
      "Miembros del hogar para atribuir gastos correctamente",
      "Bandeja con pagos próximos y apertura rápida",
      "Minimizar a bandeja, inicio automático al arrancar (escritorio)",
    ],
    alt: "Apariencia y ajustes de Subly",
  },
  deviceGalleryMeta: {
    title: "Dentro de la app",
    subtitle: "Los mismos flujos principales en escritorio, móvil y tablet.",
  },
  shotLabels: {
    dashboard: "Panel",
    subscriptions: "Suscripciones",
    expenses: "Gastos",
    calendar: "Calendario",
    currencies: "Divisas",
    settings: "Ajustes",
  },
  deviceGalleries: [
    {
      description:
        "Experiencia completa de escritorio con widgets del panel, bandeja del sistema y exportaciones — para macOS, Windows y Linux.",
    },
    {
      description:
        "Siga suscripciones y gastos en movimiento con las mismas categorías, etiquetas y calendario de renovaciones.",
    },
    {
      description:
        "Más espacio para listas y calendario mensual en iPad y tablets Android sin perder la privacidad local-first.",
    },
  ],
  appStore: {
    title: "También en App Store",
    body: "Subly está en Mac App Store y para iPhone/iPad. La descarga es de pago — su compra apoya el desarrollo continuo. Las builds de escritorio para macOS, Windows y Linux siguen siendo gratuitas en GitHub Releases.",
    ctaLabel: "Ver en App Store",
  },
  faqSection: {
    title: "FAQ",
  },
  faqItems: [
    {
      q: "¿Subly es gratis?",
      a: "Subly es open source bajo licencia MIT. Las builds de GitHub Releases son gratuitas. En las tiendas pueden aplicarse comisiones de la plataforma.",
    },
    {
      q: "¿Funciona sin conexión?",
      a: "Sí. Suscripciones, gastos, panel y calendario funcionan totalmente offline. Sync, actualización de tipos, IA y Telegram necesitan red cuando use esas funciones.",
    },
    {
      q: "¿Necesito IA?",
      a: "No. La IA es opcional y desactivada por defecto. Si la activa, puede añadir suscripciones o gastos rápido, escanear tickets o importar extractos — cada capacidad tiene su interruptor y elige entre siete proveedores con su clave API.",
    },
    {
      q: "¿Es segura la sync en la nube?",
      a: "La sync usa su propia cuenta en la nube (iCloud, Drive, Dropbox, OneDrive o WebDAV). Las credenciales están en el llavero del sistema. Active sync solo si quiere copia multi-dispositivo.",
    },
    {
      q: "¿Qué plataformas se admiten?",
      a: "macOS (Universal), Windows, Linux, iOS y Android. Instaladores de escritorio en GitHub Releases; móvil donde esté publicado en tiendas.",
    },
    {
      q: "¿Cómo hago copia de mis datos?",
      a: "Ajustes → Datos: exporte un archivo `.subly`, JSON o CSV. Importe los mismos formatos al cambiar de dispositivo.",
    },
    {
      q: "¿Dónde se guardan mis datos?",
      a: "En su dispositivo por defecto. No operamos una nube central Subly con sus suscripciones.",
    },
    {
      q: "¿Subly usa analíticas o seguimiento?",
      a: "No. Sin SDKs de analíticas, redes de anuncios ni métricas de uso hacia nosotros. Las funciones opcionales (sync, IA, Telegram, tipos) solo hablan con servicios que configure.",
    },
    {
      q: "¿Puedo leer el código fuente?",
      a: "Sí. Subly es totalmente open source en GitHub bajo MIT — frontend, backend y builds móviles. Puede verificar las afirmaciones de privacidad usted mismo.",
    },
    {
      q: "¿Puedo probarlo antes de instalar?",
      a: "Vea los vídeos demo de escritorio o móvil en esta página, o la galería de capturas abajo.",
    },
  ],
  downloadSection: {
    title: "¿Listo para tomar el control?",
    body: "Builds de escritorio gratuitas en GitHub Releases — elija su plataforma abajo. ¿Prefiere App Store en dispositivos Apple? Esa opción apoya el desarrollo.",
    ctaPrimary: "Descargar última versión",
    githubHeading: "Gratis en GitHub",
    ctaSecondary: "Destacar en GitHub",
    ctaDonate: "Apoyar el desarrollo",
    ctaVideo: "Ver demo",
    changelogLabel: "Changelog",
  },
  footer: {
    tagline: "Subly — control de suscripciones y gastos",
    license: "Licencia MIT",
    donateLabel: "Donar",
    releasesLabel: "Releases",
  },
};

export { esStrings };

export const esContent = buildSiteContent("es", esStrings);
