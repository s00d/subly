/**
 * Available logo assets from public/assets/*.svg
 * Grouped: Services first, then Payment methods
 */

export interface LogoAsset {
  name: string;
  path: string;
  group: "service" | "payment";
}

export const logoAssets: LogoAsset[] = [
  // ---- Popular Services ----
  { name: "Netflix", path: "/assets/netflix.svg", group: "service" },
  { name: "Spotify", path: "/assets/spotify.svg", group: "service" },
  { name: "YouTube", path: "/assets/youtube.svg", group: "service" },
  { name: "Amazon", path: "/assets/amazon.svg", group: "service" },
  { name: "Disney+", path: "/assets/disney-plus.svg", group: "service" },
  { name: "HBO", path: "/assets/hbo.svg", group: "service" },
  { name: "Twitch", path: "/assets/twitch.svg", group: "service" },
  { name: "GitHub", path: "/assets/github.svg", group: "service" },
  { name: "Dropbox", path: "/assets/dropbox.svg", group: "service" },
  { name: "Adobe", path: "/assets/adobe.svg", group: "service" },
  { name: "Microsoft", path: "/assets/microsoft.svg", group: "service" },
  { name: "Slack", path: "/assets/slack.svg", group: "service" },
  { name: "Zoom", path: "/assets/zoom.svg", group: "service" },
  { name: "Steam", path: "/assets/steam.svg", group: "service" },
  { name: "PlayStation", path: "/assets/playstation.svg", group: "service" },
  { name: "Xbox", path: "/assets/xbox.svg", group: "service" },
  { name: "Nintendo", path: "/assets/nintendo.svg", group: "service" },
  { name: "iCloud", path: "/assets/icloud.svg", group: "service" },
  { name: "Notion", path: "/assets/notion.svg", group: "service" },
  { name: "Figma", path: "/assets/figma.svg", group: "service" },
  { name: "ChatGPT", path: "/assets/chatgpt.svg", group: "service" },
  { name: "VPN", path: "/assets/vpn.svg", group: "service" },
  { name: "Cloud Storage", path: "/assets/cloud-storage.svg", group: "service" },
  { name: "Music", path: "/assets/music-service.svg", group: "service" },
  { name: "Gaming", path: "/assets/gaming.svg", group: "service" },
  { name: "Fitness", path: "/assets/fitness.svg", group: "service" },

  // ---- Payment Methods ----
  { name: "PayPal", path: "/assets/paypal.svg", group: "payment" },
  { name: "Visa", path: "/assets/visa.svg", group: "payment" },
  { name: "Visa Alt", path: "/assets/visa-alt.svg", group: "payment" },
  { name: "Mastercard", path: "/assets/mastercard.svg", group: "payment" },
  { name: "Mastercard Alt", path: "/assets/mastercard-alt.svg", group: "payment" },
  { name: "American Express", path: "/assets/american-express.svg", group: "payment" },
  { name: "Apple Pay", path: "/assets/apple-pay.svg", group: "payment" },
  { name: "Google Pay", path: "/assets/google-pay.svg", group: "payment" },
  { name: "Samsung Pay", path: "/assets/samsung-pay.svg", group: "payment" },
  { name: "Amazon Pay", path: "/assets/amazon-pay.svg", group: "payment" },
  { name: "Klarna", path: "/assets/klarna.svg", group: "payment" },
  { name: "SEPA", path: "/assets/sepa.svg", group: "payment" },
  { name: "Crypto", path: "/assets/crypto.svg", group: "payment" },
  { name: "Maestro", path: "/assets/maestro.svg", group: "payment" },
  { name: "Card Generic", path: "/assets/card-generic.svg", group: "payment" },
  { name: "Card Gold", path: "/assets/card-generic-gold.svg", group: "payment" },
  { name: "Diners Club", path: "/assets/diners.svg", group: "payment" },
  { name: "Discover", path: "/assets/discover.svg", group: "payment" },
  { name: "JCB", path: "/assets/jcb.svg", group: "payment" },
  { name: "UnionPay", path: "/assets/unionpay.svg", group: "payment" },
  { name: "Alipay", path: "/assets/alipay.svg", group: "payment" },
  { name: "WeChat Pay", path: "/assets/wechat-pay.svg", group: "payment" },
  { name: "Bancontact", path: "/assets/bancontact.svg", group: "payment" },
  { name: "BLIK", path: "/assets/blik.svg", group: "payment" },
  { name: "EPS", path: "/assets/eps.svg", group: "payment" },
  { name: "Giropay", path: "/assets/giropay.svg", group: "payment" },
  { name: "iDEAL", path: "/assets/ideal.svg", group: "payment" },
  { name: "MobilePay", path: "/assets/mobilepay.svg", group: "payment" },
  { name: "Paysafecard", path: "/assets/paysafecard.svg", group: "payment" },
  { name: "Skrill", path: "/assets/skrill.svg", group: "payment" },
  { name: "Swish", path: "/assets/swish.svg", group: "payment" },
  { name: "TWINT", path: "/assets/twint.svg", group: "payment" },
  { name: "V PAY", path: "/assets/vpay.svg", group: "payment" },
  { name: "Invoice", path: "/assets/invoice.svg", group: "payment" },
  { name: "Vipps", path: "/assets/vipps.svg", group: "payment" },
  { name: "Elo", path: "/assets/elo.svg", group: "payment" },
  { name: "Hipercard", path: "/assets/hipercard.svg", group: "payment" },
  { name: "Przelewy24", path: "/assets/przelewy24.svg", group: "payment" },
  { name: "Dankort", path: "/assets/dankort.svg", group: "payment" },
];

/** Helper to check if icon is an image path */
export function isImageIcon(icon: string): boolean {
  return icon.startsWith("/") || icon.startsWith("http") || icon.startsWith("data:");
}
