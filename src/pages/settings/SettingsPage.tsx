import { text } from "../../shared/text";
import { useSettingsPage } from "./useSettingsPage";
import styles from "./SettingsPage.module.css";

export function SettingsPage() {
  const model = useSettingsPage();
  return <section class={styles.page} aria-labelledby="settings-title">
    <header class={styles.header}>
      <h1 id="settings-title">{text("app.settings")}</h1>
      <p>{text("pages.settings.description")}</p>
    </header>
    {model.error && <p class={styles.error} role="alert">{model.error}</p>}
    {!model.settings ? (
      model.error
        ? <button onClick={model.retry}>{text("app.retry")}</button>
        : <p role="status">{text("app.loading")}</p>
    ) : <>
      {model.settings.loadWarning && <p class={styles.warning} role="status">
        {text("pages.settings.loadWarning")} <span>{model.settings.loadWarning}</span>
      </p>}
      <form class={styles.form} onSubmit={(event) => { event.preventDefault(); void model.save(); }}>
        <h2>{text("pages.settings.logging")}</h2>
        <label for="log-retention">{text("pages.settings.retention")}</label>
        <div class={styles.inputRow}>
          <input id="log-retention" type="number" min="1" max="999999" step="1"
            value={model.value} disabled={model.saving} required
            aria-invalid={!model.valid} aria-describedby="retention-help retention-prune"
            onInput={(event) => model.changeValue(event.currentTarget.value)} />
          <span>{text("pages.settings.unit")}</span>
        </div>
        <p id="retention-help" class={styles.help}>{text("pages.settings.help")}</p>
        <p id="retention-prune" class={styles.help}>{text("pages.settings.prune")}</p>
        <h2 class={styles.randomHeading}>{text("pages.settings.random")}</h2>
        <label class={styles.checkbox}>
          <input type="checkbox" checked={model.fixRandomSeed} disabled={model.saving}
            aria-describedby="random-seed-help"
            onChange={(event) => model.changeFixRandomSeed(event.currentTarget.checked)} />
          {text("pages.settings.fixRandomSeed")}
        </label>
        <p id="random-seed-help" class={styles.help}>{text("pages.settings.randomSeedHelp")}</p>
        <div class={styles.actions}>
          <button type="submit" disabled={model.saving || !model.valid}>
            {text(model.saving ? "pages.settings.saving" : "pages.settings.save")}
          </button>
          {model.saved && <span class={styles.saved} role="status">{text("pages.settings.saved")}</span>}
        </div>
      </form>
      <section class={styles.storage}>
        <h2>{text("pages.settings.portable")}</h2>
        <p>{model.settings.path ?? text("pages.settings.unavailable")}</p>
      </section>
    </>}
  </section>;
}
