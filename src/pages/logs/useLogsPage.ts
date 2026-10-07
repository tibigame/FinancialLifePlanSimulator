import { useEffect, useLayoutEffect, useRef, useState } from "preact/hooks";
import { errorMessage, getLogs } from "../../shared/api";
import { categories, levels, type Category, type LogEntry, type LogPage, type Severity } from "../../shared/types";

export const ROW_HEIGHT = 32;
const OVERSCAN = 8;

export interface LogViewState {
  levels: Severity[];
  categories: Category[];
  scrollTop: number;
  anchorId: number | null;
}
export function createLogViewState(): LogViewState {
  return { levels: [...levels], categories: [...categories], scrollTop: 0, anchorId: null };
}

export function useLogsPage(cache: LogViewState) {
  const viewport = useRef<HTMLDivElement>(null);
  const [selectedLevels, setLevels] = useState(cache.levels);
  const [selectedCategories, setCategories] = useState(cache.categories);
  const [scrollTop, setScrollTop] = useState(cache.scrollTop);
  const [height, setHeight] = useState(600);
  const [result, setResult] = useState<{ page: LogPage; offset: number } | null>(null);
  const [error, setError] = useState("");
  const [selectedEntry, selectEntry] = useState<LogEntry | null>(null);
  const [refresh, setRefresh] = useState(0);
  const offset = Math.max(0, Math.floor(scrollTop / ROW_HEIGHT) - OVERSCAN);
  const count = Math.min(200, Math.ceil(height / ROW_HEIGHT) + OVERSCAN * 2);
  const anchorId = cache.anchorId;

  useLayoutEffect(() => {
    const element = viewport.current;
    if (!element) return;
    const observer = new ResizeObserver(() => setHeight(element.clientHeight));
    observer.observe(element);
    return () => observer.disconnect();
  }, []);

  useEffect(() => {
    let active = true;
    let timer: ReturnType<typeof setTimeout>;
    async function read() {
      try {
        const page = await getLogs({ levels: selectedLevels, categories: selectedCategories, anchorId, offset, limit: count });
        if (!active) return;
        setResult({ page, offset });
        setError("");
      } catch (reason) {
        if (active) setError(errorMessage(reason));
      }
      if (active) timer = setTimeout(() => { void read(); }, 1000);
    }
    timer = setTimeout(() => { void read(); }, 30);
    return () => { active = false; clearTimeout(timer); };
  }, [selectedLevels, selectedCategories, anchorId, offset, count, refresh]);

  useLayoutEffect(() => {
    const element = viewport.current;
    if (!element || !result) return;
    const maximum = Math.max(0, result.page.total * ROW_HEIGHT - element.clientHeight);
    const top = Math.min(cache.scrollTop, maximum);
    element.scrollTop = top;
    if (top !== cache.scrollTop) {
      cache.scrollTop = top;
      setScrollTop(top);
    }
  }, [result, cache]);

  function onScroll() {
    const top = viewport.current?.scrollTop ?? 0;
    cache.scrollTop = top;
    cache.anchorId = top <= 1 ? null : cache.anchorId ?? result?.page.anchorId ?? null;
    setScrollTop(top);
  }

  function latest() {
    cache.anchorId = null;
    cache.scrollTop = 0;
    if (viewport.current) viewport.current.scrollTop = 0;
    setScrollTop(0);
    setRefresh((value) => value + 1);
  }

  function changeLevels(next: Severity[]) {
    cache.levels = next;
    setLevels(next);
    latest();
  }
  function changeCategories(next: Category[]) {
    cache.categories = next;
    setCategories(next);
    latest();
  }

  return {
    viewport, result, error, selectedLevels, selectedCategories, selectedEntry,
    following: anchorId === null,
    onScroll, latest, changeLevels, changeCategories, selectEntry,
  };
}
