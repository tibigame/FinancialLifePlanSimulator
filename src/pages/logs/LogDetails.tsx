import { useEffect, useRef } from "preact/hooks";
import { text } from "../../shared/text";
import type { LogEntry } from "../../shared/types";
import styles from "./LogsPage.module.css";

export function LogDetails({ entry, close }: { entry: LogEntry; close: () => void }) {
  const dialog = useRef<HTMLDialogElement>(null);
  useEffect(() => {
    dialog.current?.showModal();
  }, []);
  return <dialog ref={dialog} class={styles.dialog} onClose={close} aria-labelledby="log-details-title">
    <div class={styles.dialogHeader}>
      <h2 id="log-details-title">{text("pages.logs.details")}</h2>
      <button onClick={() => dialog.current?.close()}>{text("app.close")}</button>
    </div>
    <dl>
      <dt>{text("pages.logs.time")}</dt><dd>{entry.timestamp}</dd>
      <dt>{text("pages.logs.level")}</dt><dd>{text(`pages.logs.levels.${entry.level}`)}</dd>
      <dt>{text("pages.logs.category")}</dt><dd>{text(`pages.logs.categories.${entry.category}`)}</dd>
      <dt>{text("pages.logs.source")}</dt><dd>{entry.source}</dd>
    </dl>
    <pre>{entry.message}</pre>
  </dialog>;
}
