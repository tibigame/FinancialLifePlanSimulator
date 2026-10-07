import { useNavigation } from "./app/useNavigation";
import { PageBoundary } from "./app/PageBoundary";
import { MainPage } from "./pages/main/MainPage";
import { LogsPage } from "./pages/logs/LogsPage";
import { SettingsPage } from "./pages/settings/SettingsPage";
import { text } from "./shared/text";
import type { PageId } from "./shared/types";
import icon from "../src-tauri/icons/icon.svg";
import styles from "./app/App.module.css";

const pages: PageId[] = ["main", "logs", "settings"];

export default function App() {
  const navigation = useNavigation();
  return <div class={styles.shell}>
    <aside class={styles.sidebar}>
      <div class={styles.brand}><img src={icon} alt="" /><span>{text("app.name")}</span></div>
      <nav class={styles.menu} aria-label={text("app.navigation")}>
        {pages.map((page) => <button key={page} aria-current={navigation.page === page ? "page" : undefined}
          onClick={() => navigation.setPage(page)}>{text(`app.${page}`)}</button>)}
      </nav>
    </aside>
    <main class={styles.content}>
      <PageBoundary key={navigation.page}>
        {navigation.page === "main" && <MainPage />}
        {navigation.page === "logs" && <LogsPage cache={navigation.logViewState} />}
        {navigation.page === "settings" && <SettingsPage />}
      </PageBoundary>
    </main>
  </div>;
}
