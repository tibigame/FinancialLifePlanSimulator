import { invoke } from "@tauri-apps/api/core";
import type { LogPage, LogQuery, Settings } from "./types";

export const getLogs = (query: LogQuery) => invoke<LogPage>("get_logs", { query });
export const getSettings = () => invoke<Settings>("get_settings");
export const saveSettings = (logRetention: number, fixRandomSeed: boolean) =>
  invoke<Settings>("save_settings", { logRetention, fixRandomSeed });

export function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
