import { invoke } from "@tauri-apps/api/core";

export class CommandError extends Error {
  command: string;
  cause: unknown;

  constructor(command: string, message: string, cause: unknown) {
    super(message);
    this.name = "CommandError";
    this.command = command;
    this.cause = cause;
  }
}

function normalizeInvokeError(command: string, error: unknown): CommandError {
  if (typeof error === "string") {
    return new CommandError(command, error, error);
  }
  if (error && typeof error === "object") {
    const maybeMessage = (error as { message?: unknown }).message;
    if (typeof maybeMessage === "string" && maybeMessage.trim().length > 0) {
      return new CommandError(command, maybeMessage, error);
    }
  }
  return new CommandError(command, `Command '${command}' failed`, error);
}

export async function callCommand<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    throw normalizeInvokeError(command, error);
  }
}
