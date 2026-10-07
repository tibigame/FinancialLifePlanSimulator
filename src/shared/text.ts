import resources from "../../resources/ja.json";

type Paths<T> = {
  [K in keyof T & string]: T[K] extends string ? K : `${K}.${Paths<T[K]>}`;
}[keyof T & string];
export type TextId = Paths<typeof resources>;

export function text(id: TextId, values: Record<string, string | number> = {}): string {
  let current: unknown = resources;
  for (const key of id.split(".")) {
    if (typeof current !== "object" || current === null || !(key in current)) return id;
    current = (current as Record<string, unknown>)[key];
  }
  if (typeof current !== "string") return id;
  return current.replace(/\{(\w+)\}/g, (placeholder, key: string) => String(values[key] ?? placeholder));
}
