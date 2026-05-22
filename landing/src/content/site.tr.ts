import { buildSiteContent } from "./buildSiteContent";
import type { SiteLocaleStrings } from "./buildSiteContent";

const trStrings: SiteLocaleStrings = {
  site: {
    title: "Subly — Abonelik ve Harcama Takibi",
    description:
      "Açık kaynak abonelik ve harcama takipçisi. Veriler önce cihazınızda, analitik veya izleme yok, isteğe bağlı bulut senkronizasyonu.",
  },
  localeSwitcherAria: "Dil",
  headerNav: {
    download: "İndir",
    menuToggle: "Menüyü aç/kapat",
    moreLabel: "Daha fazla",
    moreMenuAria: "Ek bölümler",
  },
  nav: {
    why: "Neden Subly",
    subscriptions: "Abonelikler",
    dashboard: "Pano",
    video: "Demo",
    ai: "Yapay zeka",
    sync: "Senkron",
    faq: "SSS",
  },
  hero: {
    headline: "Ne ödediğinizi bilin — yenilenmeden önce.",
    subline:
      "Cihazınızda kalan hızlı bir abonelik ve harcama takipçisi; tamamen MIT açık kaynak ve asla analitik veya kullanım metrikleri göndermez.",
    ctaPrimary: "En son sürümü indir",
    ctaSecondary: "GitHub'da görüntüle",
    watchDemo: "Demoyu izle",
  },
  videoSection: {
    eyebrow: "Ürün demosu",
    title: "Subly'yi iş başında görün",
    subtitle: "Masaüstü ve telefon kayıtları arasında geçiş yapın.",
    fallbackLabel: "Oynatma başlamazsa videoyu yeni sekmede açın",
    desktopLabel: "Masaüstü",
    desktopDescription: "macOS'ta abonelikler, harcamalar, pano ve ayarlar.",
    phoneLabel: "Telefon",
    phoneDescription: "iPhone'da aynı akışlar — listeler, takvim ve ayarlar.",
  },
  platforms: {
    title: "Çalıştığınız her yerde",
    tagline: "Önce cihazda, tamamen açık kaynak, analitiksiz — zorunlu bulut hesabı yok.",
  },
  openValuesSection: {
    eyebrow: "Açık ve dürüst",
    title: "Güçlü uygulama. Açık kod. Sıfır gözetim.",
    lead:
      "Çoğu finans uygulaması verinizi kendi sunucularında ister — yanında sessiz bir analitik akışıyla. Subly tam tersi: okuyup derleyebileceğiniz MIT kaynağı, varsayılan olarak veri cihazda ve hiçbir izleme yığını yok. «Gizlilik modu» değil. «Anonim topluyoruz» değil. Sadece metrik yok, reklam yok, gizli piksel yok.",
    pillars: [
      "Tamamen açık kaynak",
      "Asla analitik yok",
      "Doğrulayabileceğiniz güven",
    ],
    pillarBodies: [
      "Masaüstü, mobil, senkron ve isteğe bağlı yapay zeka — tüm ürün GitHub'da MIT lisansıyla. Denetleyin, çatallayın, kendi derlemenizi yayınlayın veya katkı verin. Şeffaflık bir özelliktir.",
      "Google Analytics, Firebase, Mixpanel, Amplitude, Sentry session replay veya reklam SDK'sı yok. Subly kullanım istatistiği veya çökme telemetrisini sunucularımıza göndermez. Uygulamayı nasıl kullandığınız yalnızca sizin işiniz.",
      "Veri cihazınızda. Senkronu açarsanız kendi bulutunuz. API anahtarları işletim sistemi anahtar zincirinde. README «izleme yok» diyorsa depoda arayıp görebilirsiniz.",
    ],
    productHighlights: [
      "Abonelikler, harcamalar, pano ve yenileme takvimi tek yerde",
      "34+ para birimi, 12 dil, CSV/JSON dışa aktarma",
      "İsteğe bağlı yapay zeka — dört yetenek, yedi sağlayıcı, açana kadar kapalı",
      "Çevrimdışı çalışır; Subly hesabı gerekmez",
    ],
    ctaLabel: "Kaynağı GitHub'da keşfedin",
  },
  whySection: {
    title: "İnsanlar neden Subly'yi seçiyor",
    subtitle:
      "Tekrarlayan faturalar ve günlük harcamalarda netlik — açık kod, izleme yok ve verinize saygılı araçlar.",
  },
  whyCards: [
    {
      title: "Açık kaynak, MIT lisansı",
      body: "Her özellik herkese açık depoda. Finanslarınızın etrafında kara kutu yok — inceleyin, derleyin ve kendi şartlarınızla paylaşın.",
    },
    {
      title: "Metrik veya izleme yok",
      body: "Kendimiz için ekran, tıklama veya çöküş ölçmüyoruz. Harcama kalıplarınız başkasının panosu olmaz.",
    },
    {
      title: "Veriniz, cihazınız",
      body: "Abonelikler, harcamalar ve ayarlar bilgisayarınızda veya telefonunuzda. Sunucularımıza kayıt olmadan kontrol sizde.",
    },
    {
      title: "İstediğinizde senkron",
      body: "iCloud, Google Drive, Dropbox, OneDrive veya WebDAV ile isteğe bağlı yedek — yalnızca Ayarlar'da etkinleştirirseniz.",
    },
    {
      title: "12 dil",
      body: "İngilizce, Rusça, Çince, İspanyolca, Fransızca, Almanca, Portekizce, Japonca, Korece, Arapça, Hintçe ve Türkçe.",
    },
    {
      title: "Gerçek hayat için",
      body: "Panolar, yenileme takvimi, çoklu para birimi, hatırlatmalar ve dışa aktarma — hepsi sakin bir arayüzde.",
    },
  ],
  featureSections: [
    {
      eyebrow: "Abonelikler",
      title: "Tüm tekrarlayan ücretler tek yerde",
      body: "Yayın, yazılım, faturalar veya herhangi bir hizmet ekleyin. Günlük, haftalık, aylık veya yıllık faturalama — desteklendiğinde özel döngüler.",
      bullets: [
        "Favorileri sabitleyin, kompakt veya geniş listeler",
        "Kategori veya ödeme yöntemine göre gruplayın; toplu temizlik",
        "Tam ödeme geçmişi, otomatik yenileme, gecikme takibi, notlar ve bağlantılar",
      ],
      alt: "Subly abonelik listesi",
    },
    {
      eyebrow: "Harcamalar",
      title: "Tek seferlik harcama, tablo yok",
      body: "Tutar, kategori, etiket, not ve isteğe bağlı bağlantılarla kayıt. Tam resim için dönem, kategori veya etikete göre filtreleyin.",
      bullets: [
        "Harcamaları CSV olarak dışa aktarın",
        "İş akışınıza uyarsa harcamayı aboneliğe bağlayın",
        "Aboneliklerle aynı kategoriler, etiketler ve ödeme yöntemleri",
      ],
      alt: "Subly harcama listesi",
    },
    {
      eyebrow: "Pano",
      title: "Büyük resmi bir bakışta görün",
      body: "Hangi blokların görüneceğini ve sırasını seçin. Gecikmiş uyarılardan tahminlere ve yaşam boyu abonelik maliyetine — ana ekran size göre.",
      bullets: [
        "Gecikmiş ve yaklaşan ödemeler, istatistikler, aylık bütçe ilerlemesi",
        "Harcama trendleri, tahminler, yaşam boyu maliyet, kategori ortalamaları",
        "Dağılım, en yüksek harcamalar, hafta günü, etiket ve kur geçmişi grafikleri",
      ],
      alt: "Subly pano genel görünümü",
    },
    {
      eyebrow: "Takvim",
      title: "Maaş ve faturalara göre planlayın",
      body: "Yaklaşan ücretler takvim ızgarasında. Aylık toplamları görün ve bir tarihten ilgili aboneliğe atlayın.",
      bullets: [
        "Ayın ödeme sayısı ve toplam tutarı",
        "Sonraki maaştan önce ne kesilecek — net görünüm",
        "Tek adımda abonelik ayrıntılarına geçiş",
      ],
      alt: "Subly takvim ay görünümü",
    },
    {
      eyebrow: "Para ve uyarılar",
      title: "Para birimleri ve hatırlatmalar ayak uydurur",
      body: "Birden çok para biriminde harcama takibi, döviz kurlarını zamanlayın ve para hesaptan çıkmadan bildirim alın.",
      bullets: [
        "34+ para birimi ve özel girişler",
        "Seçtiğiniz sağlayıcıdan otomatik kur güncellemesi",
        "Yaklaşan ve gecikmiş ödemeler için sistem bildirimleri",
        "İsteğe bağlı Telegram hatırlatmaları; özelleştirilebilir başlık ve zamanlama",
      ],
      alt: "Subly döviz kurları ekranı",
    },
  ],
  dashboardWidgets: [
    "Gecikmiş ve yaklaşan ödemeler",
    "İstatistikler ve bütçe ilerlemesi",
    "Harcama trendleri ve tahminler",
    "Yaşam boyu abonelik maliyeti",
    "Kategori dağılım grafikleri",
    "En yüksek harcamalar ve ay karşılaştırması",
    "Etiket veya hafta gününe göre harcamalar",
  ],
  aiSection: {
    eyebrow: "Yapay zeka yardımı",
    title: "İsteğe bağlı yapay zeka — veriyi daha hızlı eklemenin dört yolu",
    capabilitiesTitle: "Yetenekler",
    capabilitiesIntro:
      "Abonelikler ve harcamalarda tek yardım girişi — her özelliği Ayarlar'da ayrı açın.",
    inputMethodsTitle: "Girdi nasıl gönderilir",
    lead:
      "Subly'nin yapay zeka katmanı tamamen isteğe bağlı: Ayarlar'daki ana anahtarı açana kadar kapalı. Sağlayıcıyı seçersiniz, API anahtarını işletim sistemi anahtar zincirinde saklarsınız, yalnızca istediğiniz yetenekleri açarsınız. Yardım iletişimini açıp gönderene kadar modele hiçbir şey gitmez.",
    principles: [
      {
        title: "Varsayılan olarak kapalı",
        body: "Arka planda çağrı veya sessiz yükleme yok. Yapay zekayı yalnızca gerektiğinde açın.",
      },
      {
        title: "Kendi anahtarınız",
        body: "Ücretli bulut API'leri veya yerel Ollama / LM Studio — kimlik bilgileri veritabanında değil.",
      },
      {
        title: "Kaydetmeden önce önizleme",
        body: "Her abonelik, harcama veya içe aktarılan satır inceleme için gösterilir. Onay olmadan deftere girmez.",
      },
    ],
    features: [
      {
        title: "Hızlı abonelik ekleme",
        body: "Hizmeti doğal dilde anlatın — fiyat, döngü, kategori, ödeme yöntemi ve etiketler mevcut kataloğunuzdan dolar.",
        example: "«Telegram Premium 300 ₽ aylık, kart»",
      },
      {
        title: "Hızlı harcama ekleme",
        body: "Tek seferlik harcama için aynı iletişim: kısa cümleden tutar, tarih, işyeri, kategoriler ve etiketler.",
        example: "«Starbucks kahve 5.40 EUR dün»",
      },
      {
        title: "Fiş ve belge tarama",
        body: "Görü yeteneği olan modele fotoğraf veya PDF. Kalemler, toplamlar, para birimi, işyeri ve tarih — harcama taslağı.",
        example: "PNG, JPEG, WebP, HEIC veya PDF fişler",
      },
      {
        title: "Banka ekstresi içe aktarma",
        body: "CSV, XLSX, PDF veya JSON toplu içe aktarma. Sezgisel ayrıştırma ve yapay zeka yedeklemesi, kaydetmeden önce satır tablosu.",
        example: "Kart ve banka dışa aktarmalarının yaygın formatları",
      },
    ],
    inputMethods: [
      { label: "Açıklama yazın", detail: "Göndermek için ⌘/Ctrl + Enter" },
      { label: "Dosya bırakın veya seçin", detail: "Masaüstünde sürükle-bırak" },
      { label: "Ekran görüntüsü yapıştırın", detail: "Metin alanına" },
    ],
    providersTitle: "Desteklenen sağlayıcılar",
    settingsBullets: [
      "Ana anahtar ve her yetenek için ayrı düğmeler",
      "Önerilen model ön ayarları ve özel model adı",
      "Güvenmeden önce bağlantı testi",
      "Model fiş OCR destekliyorsa Vision rozeti",
    ],
    privacyNote:
      "İstekler yalnızca yapılandırdığınız sağlayıcıya gider. Subly kendi LLM'ini işletmez ve istemlerinizi sunucularımızda günlüklemez.",
    alt: "Subly yapay zeka ayarları paneli",
  },
  syncSection: {
    eyebrow: "Senkron ve yedek",
    title: "Yedeğiniz, bulutunuz",
    body: "Birden fazla cihaz gerektiğinde zaten kullandığınız sağlayıcıyı açın. Tam yedekleri istediğiniz zaman dışa/içe aktarın.",
    bullets: [
      "Cihazlarınız arasında şifreli senkron yükleri",
      "`.subly` yedekleri ile JSON ve CSV dışa/içe aktarma",
      "Hane üyeleri, kategoriler ve etiketler verinizle birlikte",
    ],
  },
  privacySection: {
    eyebrow: "Gizlilik",
    title: "Varsayılan olarak özel",
    body: "Analitik, reklam SDK'sı veya «anonim kullanım» hatları yok — ve açık kod, yalnızca sözümüze güvenmek zorunda kalmamanız için.",
    bullets: [
      "Temel veri her cihazda yerel olarak",
      "Subly sunucularına uygulama içi metrik, çökme raporu veya üçüncü taraf izleme yok",
      "API anahtarları, senkron parolaları ve Telegram jetonları işletim sistemi anahtar zincirinde — veritabanında düz metin değil",
      "Yapay zeka ve bulut isteğe bağlı; ayrıntılar için gizlilik politikamızı okuyun",
    ],
    policyLinkLabel: "Privacy Policy (EN)",
  },
  customizeSection: {
    eyebrow: "Kendinize göre yapın",
    title: "Temalar, etiketler ve hane",
    body: "15'ten fazla renk teması, özel kategori simgeleri, ödeme yöntemleri, etiketler, aylık bütçe ilerlemesi ve masaüstünde sistem tepsisi kısayolları.",
    bullets: [
      "Harcamayı doğru atamak için hane üyeleri",
      "Yaklaşan ödemeler ve hızlı açma ile sistem tepsisi",
      "Tepsiye küçültme, önyüklemede otomatik başlatma (masaüstü)",
    ],
    alt: "Subly görünüm ve ayarlar",
  },
  deviceGalleryMeta: {
    title: "Uygulamanın içinde",
    subtitle: "Masaüstü, telefon ve tablette aynı temel akışlar.",
  },
  shotLabels: {
    dashboard: "Pano",
    subscriptions: "Abonelikler",
    expenses: "Harcamalar",
    calendar: "Takvim",
    currencies: "Para birimleri",
    settings: "Ayarlar",
  },
  deviceGalleries: [
    {
      description:
        "Pano bileşenleri, sistem tepsisi kısayolları ve dışa aktarma ile tam masaüstü deneyimi — macOS, Windows ve Linux için.",
    },
    {
      description:
        "Hareket halindeyken abonelik ve harcama — aynı kategoriler, etiketler ve yenileme takvimi.",
    },
    {
      description:
        "iPad ve Android tabletlerde listeler ve aylık takvim için daha fazla alan; önce cihazda gizlilik korunur.",
    },
  ],
  appStore: {
    title: "App Store'da da",
    body: "Subly Mac ve iPhone/iPad için App Store'da. Ücretli indirmedir — satın almanız sürekli geliştirmeyi destekler. macOS, Windows ve Linux masaüstü derlemeleri GitHub Releases'ta ücretsiz kalır.",
    ctaLabel: "App Store'da görüntüle",
  },
  faqSection: {
    title: "Sık sorulan sorular",
  },
  faqItems: [
    {
      q: "Subly ücretsiz mi?",
      a: "Subly MIT lisansıyla açık kaynaktır. GitHub Releases'tan derlemeleri ücretsiz indirin. Mağaza listelerinde geçerli olduğunda platform ücretleri uygulanabilir.",
    },
    {
      q: "Çevrimdışı çalışır mı?",
      a: "Evet. Abonelikler, harcamalar, pano ve takvim tamamen çevrimdışı. Senkron, kur güncellemesi, yapay zeka ve Telegram kullandığınızda ağ gerekir.",
    },
    {
      q: "Yapay zeka gerekli mi?",
      a: "Hayır. Yapay zeka isteğe bağlı ve varsayılan olarak kapalı. Açtığınızda hızlı ekleme, fiş tarama veya ekstre içe aktarma — her yetenek ayrı anahtar, yedi sağlayıcıdan seçim ve kendi API anahtarınız.",
    },
    {
      q: "Bulut senkronu güvenli mi?",
      a: "Senkron kendi bulut hesabınızı kullanır (iCloud, Drive, Dropbox, OneDrive veya WebDAV). Kimlik bilgileri sistem anahtar zincirinde. Yalnızca çoklu cihaz yedek istiyorsanız açın.",
    },
    {
      q: "Hangi platformlar destekleniyor?",
      a: "macOS (Universal), Windows, Linux, iOS ve Android. Masaüstü yükleyicileri GitHub Releases'ta; mobil derlemeler mağazada yayımlandığı yerlerde.",
    },
    {
      q: "Verilerimi nasıl yedeklerim?",
      a: "Ayarlar → Veri'den `.subly`, JSON veya CSV dışa aktarın. Yeni cihazda aynı formatları içe aktarın.",
    },
    {
      q: "Verilerim nerede saklanıyor?",
      a: "Varsayılan olarak cihazınızda. Aboneliklerinizi tutan merkezi bir Subly bulutu işletmiyoruz.",
    },
    {
      q: "Subly analitik veya izleme kullanıyor mu?",
      a: "Hayır. Uygulamada analitik SDK'sı, reklam ağı veya bize gönderilen kullanım metrikleri yok. İsteğe bağlı özellikler (senkron, yapay zeka, Telegram, kurlar) yalnızca yapılandırdığınız hizmetlerle konuşur.",
    },
    {
      q: "Kaynak kodu okuyabilir miyim?",
      a: "Evet. Subly GitHub'da MIT ile tamamen açık kaynak — ön yüz, arka uç ve mobil. Gizlilik iddialarını kendiniz doğrulayabilirsiniz.",
    },
    {
      q: "Kurmadan önce deneyebilir miyim?",
      a: "Bu sayfadaki masaüstü veya telefon demo videolarını izleyin veya aşağıdaki ekran görüntüsü galerisine göz atın.",
    },
  ],
  downloadSection: {
    title: "Kontrolü ele almaya hazır mısınız?",
    body: "GitHub Releases'tan ücretsiz masaüstü derlemeleri — aşağıdan platformunuzu seçin. Apple cihazlarda App Store mu? Geliştirmeyi destekler.",
    ctaPrimary: "En son sürümü indir",
    githubHeading: "GitHub'da ücretsiz",
    ctaSecondary: "GitHub'da yıldızla",
    ctaDonate: "Geliştirmeyi destekle",
    ctaVideo: "Demoyu izle",
    changelogLabel: "Changelog",
  },
  footer: {
    tagline: "Subly — abonelik ve harcama takipçisi",
    license: "MIT Lisansı",
    donateLabel: "Bağış",
    releasesLabel: "Sürümler",
  },
};

export { trStrings };

export const trContent = buildSiteContent("tr", trStrings);
