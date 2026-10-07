import { text } from "../../shared/text";
import { LogDetails } from "./LogDetails";
import { LogFilters } from "./LogFilters";
import { ROW_HEIGHT, useLogsPage, type LogViewState } from "./useLogsPage";
import styles from "./LogsPage.module.css";

export function LogsPage({ cache }: { cache: LogViewState }) {
  const model = useLogsPage(cache);
  const page = model.result?.page;
  return <section class={styles.page} aria-labelledby="logs-title">
    <header class={styles.header}>
      <div><h1 id="logs-title">{text("app.logs")}</h1><p>{text("pages.logs.description")}</p></div>
      <button onClick={model.latest}>{text("pages.logs.latest")}</button>
    </header>
    <LogFilters {...model} />
    <div class={styles.summary}>
      <span>{page ? text("pages.logs.count", {
        matched: page.total.toLocaleString("ja-JP"),
        retained: page.retained.toLocaleString("ja-JP"),
        limit: page.retentionLimit.toLocaleString("ja-JP"),
      }) : text("app.loading")}</span>
      <span>{text(model.following ? "pages.logs.following" : "pages.logs.paused")}</span>
    </div>
    {model.error && <p role="alert" class={styles.error}>{text("app.error")} {model.error}</p>}
    <div class={styles.table} role="table" aria-label={text("pages.logs.table")} aria-rowcount={(page?.total ?? 0) + 1}>
      <div class={styles.columnHeader} role="row" aria-rowindex={1}>
        <span role="columnheader">{text("pages.logs.time")}</span>
        <span role="columnheader">{text("pages.logs.level")}</span>
        <span role="columnheader">{text("pages.logs.category")}</span>
        <span role="columnheader">{text("pages.logs.message")}</span>
      </div>
      <div ref={model.viewport} class={styles.viewport} onScroll={model.onScroll} tabIndex={0} role="rowgroup" aria-label={text("pages.logs.table")}>
        {page?.total === 0 && <p class={styles.empty}>{text("pages.logs.empty")}</p>}
        <div class={styles.spacer} style={{ height: `${(page?.total ?? 0) * ROW_HEIGHT}px` }}>
          <div class={styles.rows} style={{ top: `${(model.result?.offset ?? 0) * ROW_HEIGHT}px` }}>
            {page?.entries.map((entry, index) => <div class={styles.row} role="row"
              aria-rowindex={(model.result?.offset ?? 0) + index + 2} key={entry.id} style={{ height: `${ROW_HEIGHT}px` }}>
              <span role="cell" class={styles.timestamp} title={entry.timestamp}>{entry.timestamp.slice(0, 23)}</span>
              <span role="cell" class={styles[entry.level]}>{text(`pages.logs.levels.${entry.level}`)}</span>
              <span role="cell">{text(`pages.logs.categories.${entry.category}`)}</span>
              <span role="cell" class={styles.messageCell}>
                <button class={styles.message} onClick={() => model.selectEntry(entry)}
                  title={text("pages.logs.openDetails")} aria-label={`${text("pages.logs.openDetails")}: ${entry.message}`}>
                  <span class={styles.source}>{entry.source}</span>{entry.message}
                </button>
              </span>
            </div>)}
          </div>
        </div>
      </div>
    </div>
    <footer class={styles.footer}>
      {page && <div class={page.file.active ? styles.fileActive : styles.fileDisabled}>
        {text(page.file.active ? "pages.logs.fileActive" : "pages.logs.fileDisabled")}
        {page.file.path && <span title={page.file.path}>{page.file.path}</span>}
        {page.file.error && <span>{page.file.error}</span>}
      </div>}
      <p>{text("pages.logs.trimmed")}</p>
    </footer>
    {model.selectedEntry && <LogDetails entry={model.selectedEntry} close={() => model.selectEntry(null)} />}
  </section>;
}
