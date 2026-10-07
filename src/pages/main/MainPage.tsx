import { text } from "../../shared/text";
import styles from "./MainPage.module.css";

export function MainPage() {
  return <section class={styles.page} aria-label={text("app.main")}>
    <p>{text("pages.main.message")}</p>
  </section>;
}
