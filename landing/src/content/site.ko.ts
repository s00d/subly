import { buildSiteContent } from "./buildSiteContent";
import type { SiteLocaleStrings } from "./buildSiteContent";

const koStrings: SiteLocaleStrings = {
  site: {
    title: "Subly — 구독·지출 트래커",
    description:
      "오픈소스 구독·지출 관리 앱. 로컬 우선, 분석·추적 없음, 선택적 클라우드 동기화.",
  },
  localeSwitcherAria: "언어",
  headerNav: {
    download: "다운로드",
    menuToggle: "메뉴 열기",
    moreLabel: "더보기",
    moreMenuAria: "추가 섹션",
  },
  nav: {
    why: "Subly를 선택하는 이유",
    subscriptions: "구독",
    dashboard: "대시보드",
    video: "데모",
    ai: "AI",
    sync: "동기화",
    faq: "FAQ",
  },
  hero: {
    headline: "갱신되기 전에, 무엇에 얼마를 내는지 파악하세요.",
    subline:
      "빠른 구독·지출 트래커. 데이터는 기기에 남고, MIT 라이선스의 완전 오픈소스이며 분석이나 사용 지표를 보내지 않습니다.",
    ctaPrimary: "최신 릴리스 다운로드",
    ctaSecondary: "GitHub에서 보기",
    watchDemo: "데모 보기",
  },
  videoSection: {
    eyebrow: "제품 데모",
    title: "Subly 실제 사용 모습",
    subtitle: "데스크톱과 휴대폰 녹화를 전환하세요.",
    fallbackLabel: "재생이 안 되면 새 탭에서 동영상 열기",
    desktopLabel: "데스크톱",
    desktopDescription: "macOS에서 구독, 지출, 대시보드, 설정.",
    phoneLabel: "휴대폰",
    phoneDescription: "iPhone에서도 같은 흐름 — 목록, 캘린더, 설정.",
  },
  platforms: {
    title: "일하는 모든 환경에서",
    tagline: "로컬 우선, 완전 오픈소스, 분석 제로 — 강제 클라우드 계정 없음.",
  },
  openValuesSection: {
    eyebrow: "열린·정직한",
    title: "강력한 앱. 열린 코드. 감시 제로.",
    lead:
      "대부분의 재무 앱은 데이터를 자사 서버에 두고 조용히 분석을 수집합니다. Subly는 반대입니다. MIT 라이선스 소스를 읽고 직접 빌드할 수 있고, 기본은 로컬 우선이며 어떤 추적 스택도 없습니다. ‘프라이버시 모드’도 ‘익명 집계’도 아닙니다. 지표도, 광고도, 숨은 픽셀도 없습니다.",
    pillars: ["완전 오픈소스", "분석 영구 없음", "검증 가능한 신뢰"],
    pillarBodies: [
      "데스크톱, 모바일, 동기화, 선택 AI — 제품 전체가 GitHub에 MIT 라이선스로 공개됩니다. 감사, 포크, 자체 빌드, 기여가 가능합니다. 투명성도 기능입니다.",
      "Google Analytics, Firebase, Mixpanel, Amplitude, Sentry 세션 리플레이, 광고 SDK 없음. Subly는 사용 통계나 크래시 원격 분석을 당사 서버로 보내지 않습니다. 앱 사용 방식은 오직 사용자의 것입니다.",
      "데이터는 기기에 저장됩니다. 동기화는 켠 경우에만 사용자 클라우드를 사용합니다. API 키는 OS 키체인에 있습니다. README의 ‘추적 없음’은 저장소에서 grep으로 확인할 수 있습니다.",
    ],
    productHighlights: [
      "구독, 지출, 대시보드, 갱신 캘린더를 한곳에",
      "34+ 통화, 12개 언어, CSV/JSON보내기",
      "선택 AI 보조 — 4가지 기능, 7개 제공자, 기본 꺼짐",
      "오프라인 동작. Subly 계정 불필요",
    ],
    ctaLabel: "GitHub에서 소스 탐색",
  },
  whySection: {
    title: "사람들이 Subly를 고르는 이유",
    subtitle:
      "정기 결제와 일상 지출을 명확하게 — 오픈 코드, 추적 없음, 데이터를 존중하는 도구.",
  },
  whyCards: [
    {
      title: "오픈소스, MIT 라이선스",
      body: "모든 기능이 공개 저장소에 있습니다. 재정 주변에 블랙박스 없음 — 검사, 빌드, 공유는 사용자 조건으로.",
    },
    {
      title: "지표·추적 없음",
      body: "화면, 클릭, 크래시를 당사용으로 측정하지 않습니다. 지출 패턴이 남의 대시보드가 되지 않습니다.",
    },
    {
      title: "내 데이터, 내 기기",
      body: "구독, 지출, 설정은 PC나 휴대폰에 저장됩니다. 당사 서버 가입 없이 통제할 수 있습니다.",
    },
    {
      title: "원할 때만 동기화",
      body: "iCloud, Google Drive, Dropbox, OneDrive, WebDAV 선택 백업 — 설정에서 켠 경우에만.",
    },
    {
      title: "12개 언어",
      body: "영어, 러시아어, 중국어, 스페인어, 프랑스어, 독일어, 포르투갈어, 일본어, 한국어, 아랍어, 힌디어, 터키어.",
    },
    {
      title: "실생활을 위한 설계",
      body: "대시보드, 갱신 캘린더, 다통화, 알림,보내기 — 차분한 UI에 모두 모였습니다.",
    },
  ],
  featureSections: [
    {
      eyebrow: "구독",
      title: "모든 정기 결제를 한곳에",
      body: "스트리밍, 소프트웨어, 공과금 등 모든 서비스를 추가. 일·주·월·년 결제와 지원 환경의 사용자 지정 주기.",
      bullets: [
        "즐겨찾기 고정, 간결/펼침 목록 전환",
        "카테고리·결제 수단별 그룹, 일괄 작업으로 정리",
        "전체 결제 내역, 자동 갱신, 연체 추적, 메모·링크",
      ],
      alt: "Subly 구독 목록",
    },
    {
      eyebrow: "지출",
      title: "스프레드시트 없는 일회성 지출",
      body: "금액, 카테고리, 태그, 메모, 선택 링크로 기록. 기간·카테고리·태그로 필터해 전체 그림을 봅니다.",
      bullets: [
        "지출을 CSV로보내어 스프레드시트에서 분석",
        "워크플로에 맞게 지출을 구독에 연결",
        "구독과 동일한 카테고리, 태그, 결제 수단",
      ],
      alt: "Subly 지출 목록",
    },
    {
      eyebrow: "대시보드",
      title: "한눈에 큰 그림",
      body: "표시 블록과 순서를 선택. 연체 알림부터 예측, 구독 평생 비용까지 — 홈 화면이 사용자에게 맞춰집니다.",
      bullets: [
        "연체·예정 결제, 통계, 월 예산 진행",
        "지출 추세, 예측, 평생 비용, 카테고리 평균",
        "분류, 상위 지출, 요일 패턴, 태그, 환율 이력 차트",
      ],
      alt: "Subly 대시보드 개요",
    },
    {
      eyebrow: "캘린더",
      title: "급여일·청구에 맞춰 계획",
      body: "다가오는 결제가 캘린더 격자에 표시됩니다. 월 합계를 보고 날짜에서 관련 구독으로 바로 이동.",
      bullets: [
        "월 결제 건수와 총액",
        "다음 급여 전에 무엇이 빠지는지 명확",
        "한 단계로 구독 상세로",
      ],
      alt: "Subly 월간 캘린더",
    },
    {
      eyebrow: "통화·알림",
      title: "다통화와 알림이 따라옴",
      body: "여러 통화로 지출을 추적하고, 일정에 맞춰 환율을 갱신하며, 출금 전에 알림을 받습니다.",
      bullets: [
        "34+ 통화, 사용자 정의 항목 추가",
        "선택한 제공자에서 환율 자동 갱신",
        "예정·연체 결제 시스템 알림",
        "선택 Telegram 알림, 제목·일정 사용자 지정",
      ],
      alt: "Subly 환율 화면",
    },
  ],
  dashboardWidgets: [
    "연체·예정 결제",
    "통계·예산 진행",
    "지출 추세·예측",
    "구독 평생 비용",
    "카테고리 분류 차트",
    "상위 지출·월 비교",
    "태그·요일별 지출",
  ],
  aiSection: {
    eyebrow: "AI 보조",
    title: "선택 AI — 데이터 입력을 빠르게 하는 4가지 방법",
    capabilitiesTitle: "기능",
    capabilitiesIntro:
      "구독·지출 화면의 공통 보조 진입점 — 설정에서 기능별로 개별 활성화.",
    inputMethodsTitle: "입력 보내는 방법",
    lead:
      "Subly의 AI 계층은 완전 선택입니다. 설정의 마스터 스위치를 켜기 전까지 비활성. 제공자를 고르고 API 키를 OS 키체인에 저장하며, 필요한 기능만 켭니다. 보조 대화상자를 열고 제출할 때만 모델로 전송됩니다.",
    principles: [
      {
        title: "기본 꺼짐",
        body: "백그라운드 호출, 조용한 업로드 없음. 필요할 때만 AI를 켭니다.",
      },
      {
        title: "자체 키 사용",
        body: "유료 클라우드 API 또는 로컬 Ollama / LM Studio — 자격 증명은 DB에 두지 않습니다.",
      },
      {
        title: "저장 전 미리보기",
        body: "구독, 지출, 가져온 행은 모두 확인 후 저장. 확인 없이 장부에 들어가지 않습니다.",
      },
    ],
    features: [
      {
        title: "구독 빠른 추가",
        body: "서비스를 자연어로 설명 — 가격, 주기, 카테고리, 결제 수단, 태그가 기존 카탈로그로 채워집니다.",
        example: "「Telegram Premium 월 300₽, 카드」",
      },
      {
        title: "지출 빠른 추가",
        body: "같은 대화상자로 일회 지출. 짧은 문장에서 금액, 날짜, 가맹점, 카테고리, 태그 파싱.",
        example: "「어제 스타벅스 커피 5.40 EUR」",
      },
      {
        title: "영수증·문서 스캔",
        body: "비전 지원 모델에 사진·PDF. 항목, 합계, 통화, 가맹점, 날짜가 지출 초안으로.",
        example: "PNG, JPEG, WebP, HEIC, PDF 영수증",
      },
      {
        title: "은행 명세 가져오기",
        body: "CSV, XLSX, PDF, JSON 일괄 가져오기. 휴리스틱과 AI 폴백, 저장 전 행별 표.",
        example: "일반적인 카드·은행보내기 형식",
      },
    ],
    inputMethods: [
      { label: "설명 입력", detail: "⌘/Ctrl + Enter로 전송" },
      { label: "파일 놓기 또는 선택", detail: "데스크톱 드래그 앤 드롭" },
      { label: "스크린샷 붙여넣기", detail: "텍스트 필드에" },
    ],
    providersTitle: "지원 제공자",
    settingsBullets: [
      "마스터 토글과 기능별 개별 스위치",
      "권장 모델 프리셋·사용자 모델 이름",
      "사용 전 연결 테스트",
      "영수증 OCR 지원 시 Vision 배지",
    ],
    privacyNote:
      "요청은 설정한 제공자로만 갑니다. Subly는 자체 LLM을 운영하지 않으며 프롬프트를 당사 서버에 기록하지 않습니다.",
    alt: "Subly AI 설정 패널",
  },
  syncSection: {
    eyebrow: "동기화·백업",
    title: "내 백업, 내 클라우드",
    body: "여러 기기가 필요하면 이미 쓰는 제공자를 켭니다. 언제든 전체 백업보내기·가져오기.",
    bullets: [
      "기기 간 암호화된 동기화 페이로드",
      "`.subly` 백업과 JSON, CSV보내기·가져오기",
      "가구 구성원, 카테고리, 태그가 데이터와 함께 이동",
    ],
  },
  privacySection: {
    eyebrow: "프라이버시",
    title: "기본이 프라이빗",
    body: "분석, 광고 SDK, ‘익명 사용’ 파이프라인 없음 — 오픈 코드베이스로 말만 믿을 필요 없음.",
    bullets: [
      "핵심 데이터는 각 기기에 로컬 저장",
      "앱 내 지표, 크래시 리포터, Subly 서버로의 제3자 추적 없음",
      "API 키, 동기화 비밀번호, Telegram 토큰은 OS 키체인 — DB 평문 저장 안 함",
      "AI·클라우드 기능은 선택. 자세한 내용은 개인정보 처리방침 참고",
    ],
    policyLinkLabel: "Privacy Policy (EN)",
  },
  customizeSection: {
    eyebrow: "나만의 방식",
    title: "테마, 태그, 가구",
    body: "15+ 색 테마, 카테고리 아이콘, 결제 수단, 태그, 월 예산 진행, 데스크톱 시스템 트레이.",
    bullets: [
      "가구 구성원으로 지출 올바르게 귀속",
      "트레이에 예정 결제·빠른 열기",
      "트레이로 최소화, 부팅 시 자동 시작(데스크톱)",
    ],
    alt: "Subly 모양 및 설정",
  },
  deviceGalleryMeta: {
    title: "앱 안쪽",
    subtitle: "데스크톱, 휴대폰, 태블릿에서 같은 핵심 흐름.",
  },
  shotLabels: {
    dashboard: "대시보드",
    subscriptions: "구독",
    expenses: "지출",
    calendar: "캘린더",
    currencies: "통화",
    settings: "설정",
  },
  deviceGalleries: [
    {
      description:
        "대시보드 위젯, 시스템 트레이,보내기의 풀 데스크톱 — macOS, Windows, Linux용.",
    },
    {
      description:
        "이동 중에도 구독·지출 기록. 카테고리, 태그, 갱신 캘린더는 데스크톱과 동일.",
    },
    {
      description:
        "iPad·Android 태블릿에서 목록·월 캘린더에 여유. 로컬 우선 프라이버시는 유지.",
    },
  ],
  appStore: {
    title: "App Store에서도",
    body: "Subly는 Mac과 iPhone/iPad에서 App Store로 이용할 수 있습니다. 유료 다운로드는 지속 개발을 돕습니다. macOS, Windows, Linux 데스크톱 빌드는 GitHub Releases에서 계속 무료입니다.",
    ctaLabel: "App Store에서 보기",
  },
  faqSection: {
    title: "FAQ",
  },
  faqItems: [
    {
      q: "Subly는 무료인가요?",
      a: "Subly는 MIT 라이선스 오픈소스입니다. GitHub Releases에서 무료로 빌드를 받을 수 있습니다. 스토어에는 플랫폼 수수료가 적용될 수 있습니다.",
    },
    {
      q: "오프라인에서 되나요?",
      a: "네. 구독, 지출, 대시보드, 캘린더는 완전 오프라인입니다. 동기화, 환율, AI, Telegram은 해당 기능 사용 시 네트워크가 필요합니다.",
    },
    {
      q: "AI가 필수인가요?",
      a: "아니요. AI는 선택이며 기본 꺼짐입니다. 켜면 구독·지출 빠른 추가, 영수증 스캔, 명세 가져오기 — 기능별 스위치와 7개 제공자 중 자체 API 키로 선택합니다.",
    },
    {
      q: "클라우드 동기화는 안전한가요?",
      a: "동기화는 사용자 클라우드 계정(iCloud, Drive, Dropbox, OneDrive, WebDAV)을 사용합니다. 자격 증명은 시스템 키체인에 저장됩니다. 여러 기기 백업이 필요할 때만 켜세요.",
    },
    {
      q: "어떤 플랫폼을 지원하나요?",
      a: "macOS(Universal), Windows, Linux, iOS, Android. 데스크톱 설치 파일은 GitHub Releases, 모바일은 게시된 스토어 배포를 따릅니다.",
    },
    {
      q: "데이터는 어떻게 백업하나요?",
      a: "설정 → 데이터에서 `.subly`, JSON, CSV를보냅니다. 새 기기에서는 같은 형식을 가져옵니다.",
    },
    {
      q: "데이터는 어디에 저장되나요?",
      a: "기본은 사용자 기기입니다. 구독을 보관하는 중앙 Subly 클라우드는 운영하지 않습니다.",
    },
    {
      q: "분석·추적을 사용하나요?",
      a: "아니요. 분석 SDK, 광고 네트워크, 당사로의 사용 지표 전송이 없습니다. 선택 기능(동기화, AI, Telegram, 환율)은 설정한 서비스와만 통신합니다.",
    },
    {
      q: "소스 코드를 볼 수 있나요?",
      a: "네. Subly는 GitHub에 MIT로 완전 오픈소스 — 프론트, 백엔드, 모바일 빌드. 프라이버시 주장을 직접 검증할 수 있습니다.",
    },
    {
      q: "설치 전에 체험할 수 있나요?",
      a: "이 페이지의 데스크톱·휴대폰 데모 영상이나 아래 스크린샷 갤러리를 보세요.",
    },
  ],
  downloadSection: {
    title: "지출을 통제할 준비가 되셨나요?",
    body: "GitHub Releases의 무료 데스크톱 빌드 — 아래에서 플랫폼을 고르세요. Apple 기기는 App Store도 — 개발을 지원합니다.",
    ctaPrimary: "최신 릴리스 다운로드",
    githubHeading: "GitHub에서 무료",
    ctaSecondary: "GitHub에 스타",
    ctaDonate: "개발 지원",
    ctaVideo: "데모 보기",
    changelogLabel: "변경 기록",
  },
  footer: {
    tagline: "Subly — 구독·지출 트래커",
    license: "MIT 라이선스",
    donateLabel: "기부",
    releasesLabel: "릴리스",
  },
};

export const koContent = buildSiteContent("ko", koStrings);
