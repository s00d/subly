import { callCommand } from "./commandClient";

export type SubscriptionTotpCurrentDto = {
  code: string;
  periodSec: number;
  validUntilMs: number;
};

export type OtpauthImportDto = {
  totpSecret: string;
  label: string;
  issuer: string;
};

export async function subscriptionTotpCurrent(subscriptionId: string): Promise<SubscriptionTotpCurrentDto> {
  return callCommand("subscription_totp_current", { subscriptionId });
}

export async function subscriptionTotpImportOtpauth(uri: string): Promise<OtpauthImportDto> {
  return callCommand("subscription_totp_import_otpauth", { uri });
}

export async function subscriptionTotpDecodeQrBase64(dataBase64: string): Promise<OtpauthImportDto> {
  return callCommand("subscription_totp_decode_qr_base64", { dataBase64 });
}
