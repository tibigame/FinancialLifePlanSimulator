import type { Page } from "@playwright/test";
import type { LogEntry, LogQuery, Severity, Category } from "../../src/shared/types";

export async function mockBackend(page: Page, count = 1500) {
  await page.addInitScript((initialCount) => {
    let limit = 2000;
    let fixRandomSeed = true;
    let sequence = initialCount;
    let saveFails = false;
    let largeDataset = false;
    let entries: LogEntry[] = Array.from({ length: initialCount }, (_, index) => ({
      id: index + 1,
      timestamp: "2026-10-08 12:34:50.123 +09:00",
      level: index % 3 === 0 ? "Error" : "Info",
      category: index % 2 === 0 ? "Setting" : "Common",
      source: "test",
      message: `entry-${index + 1}`,
    }));
    const settings = () => ({
      logRetention: limit,
      fixRandomSeed,
      path: "D:\\Portable\\settiong.toml",
      loadWarning: null,
    });
    const append = (level: Severity, category: Category, source: string, message: string) => {
      entries.push({ id: ++sequence, timestamp: "2026-10-08 12:35:00.000 +09:00", level, category, source, message });
      entries = entries.slice(-limit);
    };
    Object.assign(window, {
      __testBackend: {
        append: (message: string) => append("Info", "Common", "test", message),
        failSave: () => { saveFails = true; },
        snapshot: () => entries,
        useLargeDataset: () => { largeDataset = true; sequence = 999999; limit = 999999; },
      },
      __TAURI_INTERNALS__: {
        invoke: async (command: string, args: Record<string, unknown> = {}) => {
          if (command === "get_settings") return settings();
          if (command === "save_settings") {
            if (saveFails) throw new Error("write denied");
            const next = Number(args.logRetention);
            if (!Number.isInteger(next) || next < 1 || next > 999999) throw new Error("invalid");
            limit = next;
            fixRandomSeed = Boolean(args.fixRandomSeed);
            entries = entries.slice(-limit);
            return settings();
          }
          if (command === "write_frontend_log") {
            append(args.level as Severity, "Frontend", String(args.source), String(args.message));
            return;
          }
          if (command === "get_logs") {
            const query = args.query as unknown as LogQuery;
            const anchorId = query.anchorId ?? sequence;
            if (largeDataset) {
              const total = query.levels.includes("Info") && query.categories.includes("Common") ? anchorId : 0;
              return {
                entries: Array.from({ length: Math.max(0, Math.min(query.limit, total - query.offset)) }, (_, index) => ({
                  id: anchorId - query.offset - index,
                  timestamp: "2026-10-08 12:34:50.123 +09:00", level: "Info", category: "Common",
                  source: "test", message: `entry-${anchorId - query.offset - index}`,
                })),
                total, retained: limit, retentionLimit: limit, anchorId,
                file: { active: true, path: "D:\\Portable\\log\\20261008123450.log", error: null },
              };
            }
            const filtered = entries.filter((entry) => entry.id <= anchorId && query.levels.includes(entry.level) && query.categories.includes(entry.category)).reverse();
            return {
              entries: filtered.slice(query.offset, query.offset + query.limit),
              total: filtered.length, retained: entries.length, retentionLimit: limit, anchorId,
              file: { active: false, error: "write denied", path: null },
            };
          }
          throw new Error(`Unexpected command: ${command}`);
        },
      },
    });
  }, count);
}
