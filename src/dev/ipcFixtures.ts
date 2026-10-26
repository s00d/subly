/**
 * Dev-only IPC record & replay, so the UI can be designed in a plain browser (`pnpm dev`).
 *
 * 1. Inside `tauri dev` (ideally on a demo DB, `SUBLY_DATA_DIR=…`), every app command response is
 *    POSTed to the Vite dev server, which keeps the latest one per command+args in `.dev/ipc-fixtures.json`.
 * 2. In a browser (no Tauri runtime), `mockIPC` answers from that file: exact args first, then the
 *    latest response for the command, else `null`. Events are mocked locally.
 *
 * Only app commands going through `commandClient` are recorded; plugin calls (keyring reads with real
 * secrets included) never are.
 */
import { mockIPC, mockWindows } from "@tauri-apps/api/mocks";
import { setIpcRecorder } from "@/services/commandClient";

const ENDPOINT = "/__ipc-fixtures";

type Fixture = { cmd: string; args: unknown; result: unknown };

function argsKey(cmd: string, args: unknown): string {
  return `${cmd}:${JSON.stringify(args ?? {})}`;
}

export function installIpcRecorder(): void {
  setIpcRecorder((cmd, args, result) => {
    const body: Fixture = { cmd, args, result };
    void fetch(ENDPOINT, { method: "POST", body: JSON.stringify(body) }).catch(() => {});
  });
}

export async function installIpcMock(): Promise<void> {
  const fixtures: Fixture[] = await fetch(ENDPOINT)
    .then((r) => (r.ok ? r.json() : []))
    .catch(() => []);
  const exact = new Map<string, unknown>();
  const latest = new Map<string, unknown>();
  for (const f of fixtures) {
    exact.set(argsKey(f.cmd, f.args), f.result);
    latest.set(f.cmd, f.result);
  }
  mockWindows("main");
  mockIPC(
    (cmd, args) => {
      const key = argsKey(cmd, args);
      if (exact.has(key)) return exact.get(key);
      if (latest.has(cmd)) return latest.get(cmd);
      if (cmd.startsWith("plugin:os|")) return cmd.endsWith("platform") ? "macos" : "";
      if (cmd.startsWith("plugin:")) return null;
      console.warn(`[ipc-mock] no fixture for ${cmd}`);
      return null;
    },
    { shouldMockEvents: true },
  );
  console.info(`[ipc-mock] ${fixtures.length} fixtures loaded`);
}
