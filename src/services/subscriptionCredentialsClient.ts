import { callCommand } from "./commandClient";
import type { SubscriptionCredentials } from "@/schemas/appData";

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

/**
 * Pull the full credentials blob from the OS keyring on demand. The first
 * call after a fresh install / rebuild triggers exactly one system password
 * prompt; subsequent calls within the same session are silent. Returns
 * `null` when nothing is stored.
 */
export async function subscriptionCredentialsGet(
  subscriptionId: string,
): Promise<SubscriptionCredentials | null> {
  return callCommand("subscription_credentials_get", { subscriptionId });
}

export async function subscriptionTotpCurrent(subscriptionId: string): Promise<SubscriptionTotpCurrentDto> {
  return callCommand("subscription_totp_current", { subscriptionId });
}

export async function subscriptionTotpImportOtpauth(uri: string): Promise<OtpauthImportDto> {
  return callCommand("subscription_totp_import_otpauth", { uri });
}

export async function subscriptionTotpDecodeQrBase64(dataBase64: string): Promise<OtpauthImportDto> {
  return callCommand("subscription_totp_decode_qr_base64", { dataBase64 });
}
