import { buildSiteContent } from "./buildSiteContent";
import type { SiteLocaleStrings } from "./buildSiteContent";

const zhStrings: SiteLocaleStrings = {
  site: {
    title: "Subly — 订阅与支出记账",
    description:
      "开源订阅与支出记账应用。本地优先，无分析无追踪，可选云同步。",
  },
  localeSwitcherAria: "语言",
  headerNav: {
    download: "下载",
    menuToggle: "打开菜单",
    moreLabel: "更多",
    moreMenuAria: "更多版块",
  },
  nav: {
    why: "为何选 Subly",
    subscriptions: "订阅",
    dashboard: "仪表盘",
    video: "演示",
    ai: "AI",
    sync: "同步",
    faq: "常见问题",
  },
  hero: {
    headline: "续费之前，先看清要付什么。",
    subline:
      "轻快的订阅与支出记账工具，数据留在您的设备上；完整开源 MIT 许可，绝不向后台发送分析或使用指标。",
    ctaPrimary: "下载最新版本",
    ctaSecondary: "在 GitHub 查看",
    watchDemo: "观看演示",
  },
  videoSection: {
    eyebrow: "产品演示",
    title: "看看 Subly 的实际效果",
    subtitle: "在桌面与手机录制之间切换。",
    fallbackLabel: "若无法播放，请在新标签页中打开视频",
    desktopLabel: "桌面端",
    desktopDescription: "macOS 上的订阅、支出、仪表盘与设置。",
    phoneLabel: "手机",
    phoneDescription: "iPhone 上同样的流程——列表、日历与设置。",
  },
  platforms: {
    title: "在您工作的每个平台上可用",
    tagline: "本地优先、完全开源、零分析——无需强制注册云账号。",
  },
  openValuesSection: {
    eyebrow: "开放且坦诚",
    title: "强大应用。开放代码。零监控。",
    lead:
      "多数理财应用想把数据放在自家服务器上，并悄悄收集分析数据。Subly 恰恰相反：MIT 许可的源码可供阅读与自行构建，默认本地优先，没有任何追踪体系——不是「隐私模式」，不是「我们只做匿名汇总」，而是没有指标、没有广告、没有隐藏像素。",
    pillars: ["完全开源", "永不分析", "可验证的信任"],
    pillarBodies: [
      "桌面、移动、同步与可选 AI——整个产品均在 GitHub 上以 MIT 许可发布。可审计、可 fork、可发布自己的构建或贡献修复。透明本身就是一项功能。",
      "没有 Google Analytics、Firebase、Mixpanel、Amplitude、Sentry 会话回放或广告 SDK。Subly 不会向我们的服务器收集使用统计或崩溃遥测。您如何使用应用，只属于您自己。",
      "数据保存在您的设备上。若您开启同步，则使用您自己的云。API 密钥存放在系统钥匙串中。当 README 写着「无追踪」时，您可以在仓库里亲自查证。",
    ],
    productHighlights: [
      "订阅、支出、仪表盘与续费日历，尽在一处",
      "34+ 种货币、12 种语言、CSV/JSON 导出",
      "可选 AI 辅助——四项能力、七家提供商，默认关闭",
      "可离线使用；无需 Subly 账号",
    ],
    ctaLabel: "在 GitHub 探索源码",
  },
  whySection: {
    title: "人们为何选择 Subly",
    subtitle:
      "让定期账单与日常支出一目了然——开放代码、无追踪，并尊重您的数据。",
  },
  whyCards: [
    {
      title: "开源，MIT 许可",
      body: "所有功能都在公开仓库中。您的财务没有黑箱——可按自己的方式检查、构建与分享。",
    },
    {
      title: "无指标、无追踪",
      body: "我们不会为自己统计界面、点击或崩溃。您的消费模式不会变成别人的仪表盘。",
    },
    {
      title: "您的数据，您的设备",
      body: "订阅、支出与设置保存在您的电脑或手机上。无需在我们的服务器上注册即可掌控一切。",
    },
    {
      title: "按需同步",
      body: "可选通过 iCloud、Google Drive、Dropbox、OneDrive 或 WebDAV 备份——仅在设置中开启时生效。",
    },
    {
      title: "12 种语言",
      body: "英语、俄语、中文、西班牙语、法语、德语、葡萄牙语、日语、韩语、阿拉伯语、印地语和土耳其语。",
    },
    {
      title: "为真实生活而设计",
      body: "仪表盘、续费日历、多币种、提醒与导出——一切汇聚在清爽界面中。",
    },
  ],
  featureSections: [
    {
      eyebrow: "订阅",
      title: "所有定期扣款，集中一处",
      body: "添加流媒体、软件、公用事业或任意服务。支持按日、周、月、年计费，以及在支持处的自定义周期。",
      bullets: [
        "置顶收藏，在紧凑与展开列表间切换",
        "按类别或支付方式分组；批量操作便于整理",
        "完整付款记录、自动续费标记、逾期追踪、备注与链接",
      ],
      alt: "Subly 订阅列表",
    },
    {
      eyebrow: "支出",
      title: "一次性支出，无需电子表格",
      body: "记录金额、类别、标签、备注与可选链接。需要全貌时可按时段、类别或标签筛选。",
      bullets: [
        "将支出导出为 CSV，便于在表格中分析",
        "在适合您的工作流时，将支出关联到订阅",
        "与订阅共用类别、标签与支付方式",
      ],
      alt: "Subly 支出列表",
    },
    {
      eyebrow: "仪表盘",
      title: "一眼看清全局",
      body: "选择显示的模块及顺序。从逾期提醒到预测与订阅终身成本——主屏幕随您而变。",
      bullets: [
        "逾期与即将到期付款、统计、月度预算进度",
        "支出趋势、预测、终身成本、类别均值",
        "分类占比、大额支出、星期分布、标签视图与汇率历史图表",
      ],
      alt: "Subly 仪表盘概览",
    },
    {
      eyebrow: "日历",
      title: "围绕发薪日与账单规划",
      body: "即将扣款显示在日历网格上。查看月度合计，并从日期一步跳转到相关订阅。",
      bullets: [
        "当月付款笔数与总金额",
        "清楚看到下次发薪前会扣哪些款",
        "一步进入订阅详情",
      ],
      alt: "Subly 月历视图",
    },
    {
      eyebrow: "货币与提醒",
      title: "多币种与提醒，跟得上您",
      body: "用多种货币记账，按计划刷新汇率，并在扣款前收到通知。",
      bullets: [
        "34+ 种货币，并可添加自定义条目",
        "按所选提供商自动更新汇率",
        "系统通知：即将到期与逾期付款",
        "可选 Telegram 提醒；可自定义标题与时间",
      ],
      alt: "Subly 汇率界面",
    },
  ],
  dashboardWidgets: [
    "逾期与即将到期付款",
    "统计与预算进度",
    "支出趋势与预测",
    "订阅终身成本",
    "类别占比图表",
    "大额支出与月度对比",
    "按标签或星期分布的支出",
  ],
  aiSection: {
    eyebrow: "AI 辅助",
    title: "可选 AI——四种更快录入数据的方式",
    capabilitiesTitle: "能力",
    capabilitiesIntro:
      "在订阅与支出界面共用一个辅助入口——可在设置中分别开启各项功能。",
    inputMethodsTitle: "如何发送输入",
    lead:
      "Subly 的 AI 层完全可选：在设置中打开总开关之前保持关闭。您选择提供商，将 API 密钥存入系统钥匙串，并只开启需要的能力。只有在您打开辅助对话框并提交时，才会向模型发送内容。",
    principles: [
      {
        title: "默认关闭",
        body: "无后台调用、无静默上传。仅在需要时启用 AI。",
      },
      {
        title: "自带密钥",
        body: "付费云 API 或本地 Ollama / LM Studio 端点——凭据不会写入数据库。",
      },
      {
        title: "保存前预览",
        body: "每条订阅、支出或导入行都会先展示供确认。未经确认不会写入账本。",
      },
    ],
    features: [
      {
        title: "快速添加订阅",
        body: "用自然语言描述服务——价格、周期、类别、支付方式与标签会根据现有目录预填。",
        example: "「Telegram Premium 每月 300 ₽，银行卡」",
      },
      {
        title: "快速添加支出",
        body: "同一对话框处理一次性支出：从简短句子解析金额、日期、商户、类别与标签。",
        example: "「昨天星巴克咖啡 5.40 EUR」",
      },
      {
        title: "收据与文档扫描",
        body: "将支持视觉的模型对准照片或 PDF。明细、合计、货币、商户与日期会生成支出草稿。",
        example: "PNG、JPEG、WebP、HEIC 或 PDF 收据",
      },
      {
        title: "银行对账单导入",
        body: "批量导入 CSV、XLSX、PDF 或 JSON 对账单。启发式解析加 AI 兜底，保存前显示逐行表格。",
        example: "常见格式的银行卡与银行导出",
      },
    ],
    inputMethods: [
      { label: "输入描述", detail: "⌘/Ctrl + Enter 发送" },
      { label: "拖放或选择文件", detail: "桌面端支持拖放" },
      { label: "粘贴截图", detail: "粘贴到文本框" },
    ],
    providersTitle: "支持的提供商",
    settingsBullets: [
      "总开关及各项能力的独立开关",
      "推荐模型预设与自定义模型名称",
      "依赖前先测试连接",
      "模型支持收据 OCR 时显示 Vision 标记",
    ],
    privacyNote:
      "请求仅发往您配置的提供商。Subly 不运营自有 LLM，也不会在我们的服务器上记录您的提示词。",
    alt: "Subly AI 设置面板",
  },
  syncSection: {
    eyebrow: "同步与备份",
    title: "您的备份，您的云",
    body: "需要多台设备时，启用您已在使用的提供商。随时导出或导入完整备份。",
    bullets: [
      "设备之间加密的同步载荷",
      "导出或导入 `.subly` 备份，以及 JSON 与 CSV",
      "家庭成员、类别与标签随数据一起迁移",
    ],
  },
  privacySection: {
    eyebrow: "隐私",
    title: "默认私密",
    body: "无分析、无广告 SDK、无「匿名使用」管道——加上开放代码库，您无需只听我们一面之词。",
    bullets: [
      "核心数据默认保存在各设备本地",
      "无应用内指标、崩溃上报或向 Subly 服务器的第三方追踪",
      "API 密钥、同步密码与 Telegram 令牌存放在系统钥匙串——不会以明文写入数据库",
      "AI 与云功能均为可选；完整说明请参阅隐私政策",
    ],
    policyLinkLabel: "Privacy Policy (EN)",
  },
  customizeSection: {
    eyebrow: "按您的方式",
    title: "主题、标签与家庭",
    body: "15+ 种配色主题、自定义类别图标、支付方式、标签、月度预算进度，以及桌面端系统托盘快捷方式。",
    bullets: [
      "家庭成员，正确归属消费",
      "系统托盘显示即将到期付款并快速打开",
      "最小化到托盘、开机自启（桌面端）",
    ],
    alt: "Subly 外观与设置",
  },
  deviceGalleryMeta: {
    title: "应用内部",
    subtitle: "桌面、手机与平板上的核心流程一致。",
  },
  shotLabels: {
    dashboard: "仪表盘",
    subscriptions: "订阅",
    expenses: "支出",
    calendar: "日历",
    currencies: "货币",
    settings: "设置",
  },
  deviceGalleries: [
    {
      description:
        "完整桌面体验：仪表盘组件、系统托盘快捷方式与导出——适用于 macOS、Windows 与 Linux。",
    },
    {
      description:
        "随时随地记录订阅与支出，类别、标签与续费日历与桌面端一致。",
    },
    {
      description:
        "在 iPad 与 Android 平板上为列表与月历留出更多空间，同时保持本地优先的隐私。",
    },
  ],
  appStore: {
    title: "也可在 App Store 获取",
    body: "Subly 可通过 App Store 在 Mac 与 iPhone/iPad 上安装。付费下载有助于支持持续开发。macOS、Windows 与 Linux 的桌面构建仍可在 GitHub Releases 免费获取。",
    ctaLabel: "在 App Store 查看",
  },
  faqSection: {
    title: "常见问题",
  },
  faqItems: [
    {
      q: "Subly 免费吗？",
      a: "Subly 在 MIT 许可下开源。可从 GitHub Releases 免费下载构建。商店上架可能按平台收取费用。",
    },
    {
      q: "可以离线使用吗？",
      a: "可以。订阅、支出、仪表盘与日历可完全离线使用。同步、汇率更新、AI 与 Telegram 在您使用这些功能时需要网络。",
    },
    {
      q: "必须使用 AI 吗？",
      a: "不必。AI 为可选且默认关闭。开启后可快速添加订阅或支出、扫描收据或导入对账单——每项能力有独立开关，您使用自己的 API 密钥从七家支持的提供商中选择。",
    },
    {
      q: "云同步安全吗？",
      a: "同步使用您自己的云账号（iCloud、Drive、Dropbox、OneDrive 或 WebDAV）。凭据保存在系统钥匙串中。仅在需要多设备备份时开启同步。",
    },
    {
      q: "支持哪些平台？",
      a: "macOS（Universal）、Windows、Linux、iOS 与 Android。桌面安装包见 GitHub Releases；移动版在已发布的商店渠道提供。",
    },
    {
      q: "如何备份数据？",
      a: "前往 设置 → 数据，导出 `.subly`、JSON 或 CSV 文件。迁移到新设备时导入相同格式即可。",
    },
    {
      q: "数据存在哪里？",
      a: "默认在您的设备上。我们不运营存放您订阅信息的中央 Subly 云。",
    },
    {
      q: "Subly 会使用分析或追踪吗？",
      a: "不会。应用不包含分析 SDK、广告网络或向我们发送的使用指标。可选功能（同步、AI、Telegram、汇率更新）仅与您配置的服务通信。",
    },
    {
      q: "可以阅读源代码吗？",
      a: "可以。Subly 在 GitHub 上以 MIT 许可完全开源——前端、后端与移动构建。您可以自行验证隐私相关声明。",
    },
    {
      q: "安装前可以先体验吗？",
      a: "请观看本页的桌面或手机演示视频，或浏览下方的截图画廊。",
    },
  ],
  downloadSection: {
    title: "准备好掌控支出了吗？",
    body: "GitHub Releases 提供免费桌面构建——在下方选择您的平台。更倾向在 Apple 设备上使用 App Store？该方式有助于支持开发。",
    ctaPrimary: "下载最新版本",
    githubHeading: "GitHub 免费获取",
    ctaSecondary: "在 GitHub 标星",
    ctaDonate: "支持开发",
    ctaVideo: "观看演示",
    changelogLabel: "更新日志",
  },
  footer: {
    tagline: "Subly — 订阅与支出记账",
    license: "MIT 许可证",
    donateLabel: "捐赠",
    releasesLabel: "发布版本",
  },
};

export const zhContent = buildSiteContent("zh", zhStrings);
