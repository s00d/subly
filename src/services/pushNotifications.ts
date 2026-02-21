import { platform } from "@tauri-apps/plugin-os";
import {
  checkPermissions,
  requestPermissions,
  registerForRemoteNotifications,
  watchNotifications,
  type NotificationEvent,
} from "@inkibra/tauri-plugin-notifications";

export interface PushSetupResult {
  permissionGranted: boolean;
  registered: boolean;
  token?: string;
  error?: string;
  skipped?: boolean;
}

function isMobilePlatform(): boolean {
  const p = platform();
  return p === "ios" || p === "android";
}

/**
 * Request push notification permissions and register for remote notifications.
 * Only runs on mobile platforms — returns early on desktop.
 */
export async function setupPushNotifications(): Promise<PushSetupResult> {
  if (!isMobilePlatform()) {
    return { permissionGranted: false, registered: false, skipped: true };
  }

  try {
    const permStatus = await checkPermissions();

    if (permStatus.status !== "granted") {
      const reqResult = await requestPermissions();
      if (reqResult.status !== "granted") {
        return { permissionGranted: false, registered: false };
      }
    }

    const registration = await registerForRemoteNotifications();

    return {
      permissionGranted: true,
      registered: registration.success,
      token: registration.token,
      error: registration.error,
    };
  } catch (e) {
    console.warn("Push notification setup failed:", e);
    return { permissionGranted: false, registered: false, error: String(e) };
  }
}

/**
 * Start listening for incoming push notification events.
 * Only runs on mobile platforms.
 */
export async function startPushNotificationListener(
  onEvent: (event: NotificationEvent) => void,
): Promise<boolean> {
  if (!isMobilePlatform()) {
    return false;
  }

  try {
    const result = await watchNotifications(onEvent);
    return result.success;
  } catch (e) {
    console.warn("Push notification listener setup failed:", e);
    return false;
  }
}
