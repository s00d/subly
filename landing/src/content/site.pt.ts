import { buildSiteContent } from "./buildSiteContent";
import type { SiteLocaleStrings } from "./buildSiteContent";

const ptStrings: SiteLocaleStrings = {
  site: {
    title: "Subly — Controle de assinaturas e gastos",
    description:
      "Rastreador open source de assinaturas e gastos. Local-first, sem analytics nem rastreamento, sincronização na nuvem opcional.",
  },
  localeSwitcherAria: "Idioma",
  headerNav: {
    download: "Baixar",
    menuToggle: "Alternar menu",
    moreLabel: "Mais",
    moreMenuAria: "Mais seções",
  },
  nav: {
    why: "Por que Subly",
    subscriptions: "Assinaturas",
    dashboard: "Painel",
    video: "Demo",
    ai: "IA",
    sync: "Sync",
    faq: "FAQ",
  },
  hero: {
    headline: "Saiba no que você paga — antes de renovar.",
    subline:
      "Um rastreador rápido de assinaturas e gastos no seu dispositivo, código MIT totalmente aberto e sem analytics nem métricas de uso.",
    ctaPrimary: "Baixar última versão",
    ctaSecondary: "Ver no GitHub",
    watchDemo: "Assistir demo",
  },
  videoSection: {
    eyebrow: "Demo do produto",
    title: "Subly em ação",
    subtitle: "Alterne entre gravações de desktop e celular.",
    fallbackLabel: "Abrir o vídeo em nova aba se a reprodução não iniciar",
    desktopLabel: "Desktop",
    desktopDescription: "Assinaturas, gastos, painel e configurações no macOS.",
    phoneLabel: "Celular",
    phoneDescription: "Os mesmos fluxos no iPhone — listas, calendário e configurações.",
  },
  platforms: {
    title: "Disponível onde você trabalha",
    tagline:
      "Local-first, código aberto completo e zero analytics — sem conta na nuvem obrigatória.",
  },
  openValuesSection: {
    eyebrow: "Aberto e honesto",
    title: "App poderoso. Código aberto. Zero vigilância.",
    lead:
      "A maioria dos apps financeiros quer seus dados nos servidores deles — e um fluxo silencioso de analytics ao lado. Subly é o oposto: código sob licença MIT que você pode ler e compilar, local-first por padrão e sem pilha de rastreamento de qualquer tipo. Não é «modo privacidade». Não é «agregamos anonimamente». Só sem métricas, sem anúncios, sem pixels ocultos.",
    pillars: [
      "Totalmente open source",
      "Sem analytics. Nunca.",
      "Confiança que você pode verificar",
    ] as [string, string, string],
    pillarBodies: [
      "Desktop, mobile, sync e IA opcional — o produto inteiro está no GitHub sob MIT. Audite, faça fork, publique seu build ou contribua com correções. Transparência é um recurso.",
      "Sem Google Analytics, Firebase, Mixpanel, Amplitude, Sentry session replay nem SDKs de anúncio. Subly não coleta estatísticas de uso nem telemetria de crash para nossos servidores. Como você usa o app é só seu negócio.",
      "Os dados ficam no seu dispositivo. A sync usa sua nuvem se você ativar. Chaves de API ficam no chaveiro do SO. Quando o README diz «sem rastreamento», você pode buscar no repositório e ver.",
    ] as [string, string, string],
    productHighlights: [
      "Assinaturas, gastos, painel e calendário de renovação em um só lugar",
      "34+ moedas, 12 idiomas, exportações CSV/JSON",
      "Assistência de IA opcional — quatro capacidades, sete provedores, desligada até você ativar",
      "Funciona offline; não precisa de conta Subly",
    ],
    ctaLabel: "Explorar o código no GitHub",
  },
  whySection: {
    title: "Por que as pessoas escolhem Subly",
    subtitle:
      "Clareza em contas recorrentes e gastos do dia a dia — código aberto, sem rastreamento e ferramentas que respeitam seus dados.",
  },
  whyCards: [
    {
      title: "Open source, licença MIT",
      body: "Cada recurso está em um repositório público. Sem caixa preta em volta das suas finanças — inspecione, compile e compartilhe do seu jeito.",
    },
    {
      title: "Sem métricas nem rastreamento",
      body: "Não medimos telas, cliques ou crashes para nós. Seus padrões de gasto nunca viram o painel de outra pessoa.",
    },
    {
      title: "Seus dados, seu dispositivo",
      body: "Assinaturas, gastos e configurações ficam no seu computador ou celular. Você controla sem se cadastrar nos nossos servidores.",
    },
    {
      title: "Sync quando quiser",
      body: "Backup opcional via iCloud, Google Drive, Dropbox, OneDrive ou WebDAV — só se ativar em Configurações.",
    },
    {
      title: "12 idiomas",
      body: "Inglês, russo, chinês, espanhol, francês, alemão, português, japonês, coreano, árabe, hindi e turco.",
    },
    {
      title: "Feito para a vida real",
      body: "Painéis, calendário de renovação, multimoeda, lembretes e exportações — tudo em uma interface tranquila.",
    },
  ],
  featureSections: [
    {
      eyebrow: "Assinaturas",
      title: "Todas as cobranças recorrentes em um lugar",
      body: "Adicione streaming, software, utilidades ou qualquer serviço. Cobrança diária, semanal, mensal ou anual — mais ciclos personalizados onde suportado.",
      bullets: [
        "Fixe favoritos e alterne entre listas compactas ou expandidas",
        "Agrupe por categoria ou forma de pagamento; ações em lote para organizar",
        "Histórico completo, renovação automática, atrasos, notas e links",
      ],
      alt: "Lista de assinaturas do Subly",
    },
    {
      eyebrow: "Gastos",
      title: "Gastos avulsos sem planilha",
      body: "Registre compras com valores, categorias, tags, notas e links opcionais. Filtre por período, categoria ou tags quando precisar do quadro completo.",
      bullets: [
        "Exporte gastos para CSV em planilhas",
        "Vincule gastos a assinaturas quando fizer sentido no seu fluxo",
        "Mesmas categorias, tags e formas de pagamento das assinaturas",
      ],
      alt: "Lista de gastos do Subly",
    },
    {
      eyebrow: "Painel",
      title: "A visão geral de relance",
      body: "Escolha quais blocos aparecem e em que ordem. De alertas de atraso a previsões e custo vitalício de assinaturas — sua tela inicial se adapta a você.",
      bullets: [
        "Pagamentos em atraso e próximos, estatísticas, progresso do orçamento mensal",
        "Tendências de gasto, previsões, custos vitalícios, médias por categoria",
        "Gráficos de divisão, top gastos, dias da semana, tags e histórico de câmbio",
      ],
      alt: "Visão geral do painel do Subly",
    },
    {
      eyebrow: "Calendário",
      title: "Planeje em torno de salário e contas",
      body: "Cobranças futuras aparecem na grade do calendário. Revise totais mensais e vá de uma data direto para a assinatura relacionada.",
      bullets: [
        "Quantidade de pagamentos e valor total do mês",
        "Visão clara do que cai antes do próximo salário",
        "Acesso aos detalhes da assinatura em um passo",
      ],
      alt: "Visão mensal do calendário do Subly",
    },
    {
      eyebrow: "Dinheiro e alertas",
      title: "Moedas e lembretes que acompanham",
      body: "Acompanhe gastos em várias moedas, atualize câmbio em agenda e seja avisado antes do débito.",
      bullets: [
        "34+ moedas com espaço para entradas personalizadas",
        "Atualização automática de câmbio do provedor escolhido",
        "Notificações do sistema para pagamentos próximos e em atraso",
        "Lembretes opcionais no Telegram; títulos e agenda personalizáveis",
      ],
      alt: "Tela de câmbio do Subly",
    },
  ],
  dashboardWidgets: [
    "Pagamentos em atraso e próximos",
    "Estatísticas e progresso do orçamento",
    "Tendências e previsões de gasto",
    "Custo vitalício de assinaturas",
    "Gráficos por categoria",
    "Top gastos e comparação mensal",
    "Gastos por tags ou dia da semana",
  ],
  aiSection: {
    eyebrow: "Assistência de IA",
    title: "IA opcional — quatro formas de adicionar dados mais rápido",
    capabilitiesTitle: "Capacidades",
    capabilitiesIntro:
      "Um ponto de assistência em assinaturas e gastos — ative cada capacidade separadamente em Configurações.",
    inputMethodsTitle: "Como enviar entrada",
    lead:
      "A camada de IA do Subly é totalmente opcional: desligada até você ativar o interruptor principal em Configurações. Você escolhe o provedor, guarda a chave de API no chaveiro do SO e liga só as capacidades que quiser. Nada vai para um modelo até você abrir o diálogo de assistência e enviar.",
    principles: [
      {
        title: "Desligada por padrão",
        body: "Sem chamadas em segundo plano nem uploads silenciosos. Ative a IA só quando precisar.",
      },
      {
        title: "Traga sua própria chave",
        body: "APIs de nuvem pagas ou Ollama / LM Studio local — credenciais nunca ficam no banco de dados.",
      },
      {
        title: "Pré-visualização antes de salvar",
        body: "Cada assinatura, gasto ou linha importada é mostrada para revisão. Nada entra no seu livro sem confirmação.",
      },
    ],
    features: [
      {
        title: "Adicionar assinaturas rápido",
        body: "Descreva um serviço em linguagem natural — preço, ciclo, categoria, forma de pagamento e tags são preenchidos do seu catálogo.",
        example: "«Telegram Premium 300 ₽ por mês, cartão»",
      },
      {
        title: "Adicionar gastos rápido",
        body: "O mesmo diálogo para gastos avulsos: valor, data, loja, categorias e tags a partir de uma frase curta.",
        example: "«Café no Starbucks 5,40 EUR ontem»",
      },
      {
        title: "Digitalizar recibo e documento",
        body: "Aponte um modelo com visão para foto ou PDF. Itens, totais, moeda, loja e data voltam como rascunho de gasto.",
        example: "Recibos PNG, JPEG, WebP, HEIC ou PDF",
      },
      {
        title: "Importar extrato bancário",
        body: "Importação em massa de CSV, XLSX, PDF ou JSON. Análise heurística mais fallback de IA, com tabela por linha antes de salvar.",
        example: "Exportações comuns de cartão e banco",
      },
    ],
    inputMethods: [
      { label: "Digitar uma descrição", detail: "⌘/Ctrl + Enter para enviar" },
      { label: "Soltar ou escolher arquivos", detail: "Arrastar e soltar no desktop" },
      { label: "Colar capturas de tela", detail: "No campo de texto" },
    ],
    providersTitle: "Provedores suportados",
    settingsBullets: [
      "Interruptor principal mais chaves individuais por capacidade",
      "Modelos recomendados e nome de modelo personalizado",
      "Testar conexão antes de confiar nela",
      "Selo Vision quando o modelo suporta OCR de recibo",
    ],
    privacyNote:
      "As requisições vão só ao provedor que você configurou. Subly não opera LLM própria nem registra seus prompts nos nossos servidores.",
    alt: "Painel de configurações de IA do Subly",
  },
  syncSection: {
    eyebrow: "Sync e backup",
    title: "Seu backup, sua nuvem",
    body: "Para vários dispositivos, ative o provedor que você já usa. Exporte e importe backups completos a qualquer momento.",
    bullets: [
      "Cargas de sync criptografadas entre seus dispositivos",
      "Exportar ou importar backups `.subly`, além de JSON e CSV",
      "Membros da casa, categorias e tags viajam com seus dados",
    ],
  },
  privacySection: {
    eyebrow: "Privacidade",
    title: "Privado por padrão",
    body: "Sem analytics, SDKs de anúncio nem pipelines de «uso anônimo» — e código aberto para não precisar acreditar na palavra.",
    bullets: [
      "Dados principais armazenados localmente em cada dispositivo",
      "Sem métricas in-app, relatórios de crash nem rastreamento de terceiros para servidores Subly",
      "Chaves de API, senhas de sync e tokens do Telegram no chaveiro do SO — não em texto puro no banco",
      "IA e recursos na nuvem são opcionais; leia nossa política de privacidade para detalhes",
    ],
    policyLinkLabel: "Privacy Policy (EN)",
  },
  customizeSection: {
    eyebrow: "Do seu jeito",
    title: "Temas, tags e casa",
    body: "Mais de 15 temas de cor, ícones de categoria personalizados, formas de pagamento, tags, progresso do orçamento mensal e atalhos na bandeja do sistema no desktop.",
    bullets: [
      "Membros da casa para atribuir gastos corretamente",
      "Bandeja com pagamentos próximos e abertura rápida",
      "Minimizar para bandeja, iniciar com o sistema (desktop)",
    ],
    alt: "Aparência e configurações do Subly",
  },
  deviceGalleryMeta: {
    title: "Dentro do app",
    subtitle: "Os mesmos fluxos principais no desktop, celular e tablet.",
  },
  shotLabels: {
    dashboard: "Painel",
    subscriptions: "Assinaturas",
    expenses: "Gastos",
    calendar: "Calendário",
    currencies: "Moedas",
    settings: "Configurações",
  },
  deviceGalleries: [
    {
      description:
        "Experiência completa de desktop com widgets do painel, bandeja do sistema e exportações — para macOS, Windows e Linux.",
    },
    {
      description:
        "Acompanhe assinaturas e gastos em movimento com as mesmas categorias, tags e calendário de renovação.",
    },
    {
      description:
        "Mais espaço para listas e calendário mensal no iPad e tablets Android sem abrir mão da privacidade local-first.",
    },
  ],
  appStore: {
    title: "Também na App Store",
    body: "Subly está no Mac App Store e para iPhone/iPad. O download é pago — sua compra ajuda o desenvolvimento contínuo. Builds de desktop para macOS, Windows e Linux continuam gratuitas no GitHub Releases.",
    ctaLabel: "Ver na App Store",
  },
  faqSection: {
    title: "FAQ",
  },
  faqItems: [
    {
      q: "O Subly é gratuito?",
      a: "Subly é open source sob licença MIT. Builds do GitHub Releases são gratuitas. Nas lojas podem haver taxas da plataforma.",
    },
    {
      q: "Funciona offline?",
      a: "Sim. Assinaturas, gastos, painel e calendário funcionam totalmente offline. Sync, câmbio, IA e Telegram precisam de rede quando você usa esses recursos.",
    },
    {
      q: "Preciso de IA?",
      a: "Não. A IA é opcional e desligada por padrão. Se ativar, pode adicionar assinaturas ou gastos rápido, digitalizar recibos ou importar extratos — cada capacidade tem seu interruptor e você escolhe entre sete provedores com sua chave de API.",
    },
    {
      q: "A sync na nuvem é segura?",
      a: "A sync usa sua própria conta na nuvem (iCloud, Drive, Dropbox, OneDrive ou WebDAV). Credenciais ficam no chaveiro do sistema. Ative sync só se quiser backup multi-dispositivo.",
    },
    {
      q: "Quais plataformas são suportadas?",
      a: "macOS (Universal), Windows, Linux, iOS e Android. Instaladores de desktop no GitHub Releases; mobile onde publicado nas lojas.",
    },
    {
      q: "Como faço backup dos meus dados?",
      a: "Configurações → Dados: exporte arquivo `.subly`, JSON ou CSV. Importe os mesmos formatos ao trocar de dispositivo.",
    },
    {
      q: "Onde meus dados ficam armazenados?",
      a: "No seu dispositivo por padrão. Não operamos uma nuvem central Subly com suas assinaturas.",
    },
    {
      q: "O Subly usa analytics ou rastreamento?",
      a: "Não. Sem SDKs de analytics, redes de anúncio nem métricas de uso para nós. Recursos opcionais (sync, IA, Telegram, câmbio) só falam com serviços que você configurar.",
    },
    {
      q: "Posso ler o código-fonte?",
      a: "Sim. Subly é totalmente open source no GitHub sob MIT — frontend, backend e builds mobile. Você pode verificar as afirmações de privacidade.",
    },
    {
      q: "Posso experimentar antes de instalar?",
      a: "Assista aos vídeos demo de desktop ou celular nesta página, ou a galeria de capturas abaixo.",
    },
  ],
  downloadSection: {
    title: "Pronto para assumir o controle?",
    body: "Builds de desktop gratuitas no GitHub Releases — escolha sua plataforma abaixo. Prefere App Store em dispositivos Apple? Essa opção apoia o desenvolvimento.",
    ctaPrimary: "Baixar última versão",
    githubHeading: "Grátis no GitHub",
    ctaSecondary: "Dar estrela no GitHub",
    ctaDonate: "Apoiar o desenvolvimento",
    ctaVideo: "Assistir demo",
    changelogLabel: "Changelog",
  },
  footer: {
    tagline: "Subly — controle de assinaturas e gastos",
    license: "Licença MIT",
    donateLabel: "Doar",
    releasesLabel: "Releases",
  },
};

export { ptStrings };

export const ptContent = buildSiteContent("pt", ptStrings);
