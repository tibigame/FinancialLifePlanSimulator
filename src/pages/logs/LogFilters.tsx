import { text } from "../../shared/text";
import { categories, levels, type Category, type Severity } from "../../shared/types";
import styles from "./LogsPage.module.css";

interface Props {
  selectedLevels: Severity[];
  selectedCategories: Category[];
  changeLevels: (levels: Severity[]) => void;
  changeCategories: (categories: Category[]) => void;
}

function selection<T extends string>(selected: T[], options: readonly T[]): string {
  return selected.length === options.length ? "all" : (selected[0] ?? "all");
}

export function LogFilters(props: Props) {
  return <div class={styles.filters}>
    <select class={styles.filterSelect} aria-label={text("pages.logs.level")}
      value={selection(props.selectedLevels, levels)}
      onChange={(event) => props.changeLevels(event.currentTarget.value === "all"
        ? [...levels]
        : [event.currentTarget.value as Severity])}>
      <option value="all">{text("pages.logs.allLevels")}</option>
      {levels.map((level) => <option key={level} value={level}>
        {text(`pages.logs.levels.${level}`)}
      </option>)}
    </select>
    <select class={styles.filterSelect} aria-label={text("pages.logs.category")}
      value={selection(props.selectedCategories, categories)}
      onChange={(event) => props.changeCategories(event.currentTarget.value === "all"
        ? [...categories]
        : [event.currentTarget.value as Category])}>
      <option value="all">{text("pages.logs.allCategories")}</option>
      {categories.map((category) => <option key={category} value={category}>
        {text(`pages.logs.categories.${category}`)}
      </option>)}
    </select>
  </div>;
}
