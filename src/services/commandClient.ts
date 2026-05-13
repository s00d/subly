import { invoke } from "@tauri-apps/api/core";
import type * as DashboardIpc from "@/types/dashboardIpc";
import type { Expense } from "@/schemas/appData";

/**
 * Карта IPC-команд для типобезопасного вызова (расширять по мере необходимости).
 * Значение: аргументы `invoke` и тип результата.
 */
export type CommandMap = {
  expenses_count: { args?: Record<string, never>; result: number };
  expenses_get_by_id: { args: { id: string }; result: Expense | null };
  get_dashboard_summary: {
    args?: Record<string, never>;
    result: DashboardIpc.DashboardSummaryDto;
  };
  get_dashboard_charts: {
    args?: Record<string, never>;
    result: DashboardIpc.DashboardChartsDto;
  };
  get_dashboard_forecast: {
    args?: Record<string, never>;
    result: DashboardIpc.DashboardForecastDto;
  };
  get_dashboard_trends: {
    args?: Record<string, never>;
    result: DashboardIpc.DashboardTrendsDto;
  };
  get_rate_history_widget: {
    args: { targetIds: string[]; days: number };
    result: Record<string, DashboardIpc.RateHistoryPoint[]>;
  };
  app_ready: { args?: Record<string, never>; result: void };
};

export async function invokeCommand<K extends keyof CommandMap>(
  command: K,
  ...args: CommandMap[K]["args"] extends Record<string, never> | undefined
    ? [payload?: CommandMap[K]["args"]]
    : [payload: CommandMap[K]["args"]]
): Promise<CommandMap[K]["result"]> {
  const payload = args[0];
  return callCommand(command as string, payload as Record<string, unknown> | undefined);
}

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
