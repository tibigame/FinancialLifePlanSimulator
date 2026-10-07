import { invoke } from "@tauri-apps/api/core";
import type { Severity } from "./types";

const originalError = console.error.bind(console);
let installed = false;

export function describe(value: unknown): string {
  if (value instanceof Error) return value.stack ?? `${value.name}: ${value.message}`;
  if (typeof value === "string") return value;
  try {
    const encoded = JSON.stringify(value);
    return encoded ?? String(value);
  } catch {
    try { return String(value); } catch { return "[unprintable]"; }
  }
}

export function reportFrontend(level: Severity, source: string, ...values: unknown[]): void {
  void invoke("write_frontend_log", { level, source, message: values.map(describe).join(" ") })
    .catch((error: unknown) => originalError("Log forwarding failed:", error));
}

export function installFrontendLogging(): void {
  if (installed) return;
  installed = true;
  const mappings = [
    ["trace", "Trace"], ["debug", "Debug"], ["log", "Info"],
    ["info", "Info"], ["warn", "Warn"], ["error", "Error"],
  ] as const;
  for (const [method, level] of mappings) {
    const original = console[method].bind(console);
    console[method] = (...values: unknown[]) => {
      original(...values);
      reportFrontend(level, `console.${method}`, ...values);
    };
  }
  window.addEventListener("error", (event: Event) => {
    if (event instanceof ErrorEvent) {
      reportFrontend("Error", `${event.filename}:${event.lineno}:${event.colno}`, event.error ?? event.message);
    } else {
      const target = event.target;
      if (target instanceof HTMLElement) {
        reportFrontend("Error", "window.resource", `${target.tagName}: ${target.getAttribute("src") ?? target.getAttribute("href") ?? ""}`);
      }
    }
  }, true);
  window.addEventListener("unhandledrejection", (event) => {
    reportFrontend("Error", "window.unhandledrejection", event.reason);
  });
}
