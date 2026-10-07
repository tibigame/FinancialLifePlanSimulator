export const levels = ["Trace", "Debug", "Info", "Warn", "Error"] as const;
export const categories = ["Common", "Setting", "Frontend", "External"] as const;
export type Severity = (typeof levels)[number];
export type Category = (typeof categories)[number];
export type PageId = "main" | "logs" | "settings";

export interface LogEntry {
  id: number;
  timestamp: string;
  level: Severity;
  category: Category;
  source: string;
  message: string;
}
export interface FileStatus {
  path: string | null;
  error: string | null;
  active: boolean;
}
export interface LogQuery {
  levels: Severity[];
  categories: Category[];
  anchorId: number | null;
  offset: number;
  limit: number;
}
export interface LogPage {
  entries: LogEntry[];
  total: number;
  retained: number;
  retentionLimit: number;
  anchorId: number;
  file: FileStatus;
}
export interface Settings {
  logRetention: number;
  path: string | null;
  loadWarning: string | null;
}
