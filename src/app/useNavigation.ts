import { useRef, useState } from "preact/hooks";
import { createLogViewState } from "../pages/logs/useLogsPage";
import type { PageId } from "../shared/types";

export function useNavigation() {
  const [page, setPage] = useState<PageId>("main");
  const logViewState = useRef(createLogViewState());
  return { page, setPage, logViewState: logViewState.current };
}
