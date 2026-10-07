import { test, expect } from "@playwright/test";
import { mockBackend } from "./mockBackend";

test.beforeEach(async ({ page }) => {
  await mockBackend(page);
  await page.goto("/");
});

test("navigation shows only one page and portable settings save across page switches", async ({ page }) => {
  await expect(page.getByText("ここはメインページです")).toBeVisible();
  await page.getByRole("button", { name: "設定", exact: true }).click();
  await expect(page.getByText("ここはメインページです")).toHaveCount(0);
  const retention = page.getByLabel("ログの保持件数");
  await expect(retention).toHaveValue("2000");
  await expect(page.getByText("D:\\Portable\\settiong.toml")).toBeVisible();
  await retention.fill("0");
  await expect(page.getByRole("button", { name: "保存", exact: true })).toBeDisabled();
  await retention.fill("1000000");
  await expect(page.getByRole("button", { name: "保存", exact: true })).toBeDisabled();
  await retention.fill("25");
  await page.getByRole("button", { name: "保存", exact: true }).click();
  await expect(page.getByRole("status")).toHaveText("設定を保存しました。");
  await page.getByRole("button", { name: "メイン", exact: true }).click();
  await page.getByRole("button", { name: "設定", exact: true }).click();
  await expect(retention).toHaveValue("25");
});

test("save failures keep the active setting and show an error", async ({ page }) => {
  await page.evaluate(() => (window as any).__testBackend.failSave());
  await page.getByRole("button", { name: "設定", exact: true }).click();
  await page.getByLabel("ログの保持件数").fill("10");
  await page.getByRole("button", { name: "保存", exact: true }).click();
  await expect(page.getByRole("alert")).toContainText("設定を保存できませんでした");
  await page.getByRole("button", { name: "メイン", exact: true }).click();
  await page.getByRole("button", { name: "設定", exact: true }).click();
  await expect(page.getByLabel("ログの保持件数")).toHaveValue("2000");
});

test("log filters intersect, newest first, and survive navigation", async ({ page }) => {
  await page.getByRole("button", { name: "ログ", exact: true }).click();
  await expect(page.getByRole("button", { name: "ログの詳細を表示: entry-1500", exact: true })).toBeVisible();
  await expect(page.getByText("ファイル保存なし（メモリへの記録は継続）")).toBeVisible();
  const importance = page.getByLabel("重要度");
  const category = page.getByLabel("種類");
  await expect(importance).toHaveValue("all");
  await expect(category).toHaveValue("all");
  await expect(page.locator("fieldset, legend")).toHaveCount(0);
  await importance.selectOption("Error");
  await expect(page.getByText("該当 500 件", { exact: false })).toBeVisible();
  await category.selectOption("Setting");
  await expect(page.getByText("該当 250 件", { exact: false })).toBeVisible();
  await importance.selectOption("Info");
  await expect(page.getByText("該当 500 件", { exact: false })).toBeVisible();
  await importance.selectOption("Error");
  await category.selectOption("all");
  await expect(page.getByText("該当 500 件", { exact: false })).toBeVisible();
  await category.selectOption("Setting");
  await expect(page.getByText("該当 250 件", { exact: false })).toBeVisible();
  await page.getByRole("button", { name: "ログの詳細を表示: entry-1495", exact: true }).click();
  await expect(page.getByRole("dialog")).toContainText("entry-1495");
  await page.keyboard.press("Escape");
  await expect(page.getByRole("dialog")).toHaveCount(0);
  await page.getByRole("button", { name: "メイン", exact: true }).click();
  await page.getByRole("button", { name: "ログ", exact: true }).click();
  await expect(importance).toHaveValue("Error");
  await expect(category).toHaveValue("Setting");
});

test("older logs remain accessible without jumping when new records arrive", async ({ page }) => {
  await page.getByRole("button", { name: "ログ", exact: true }).click();
  const viewport = page.getByRole("rowgroup", { name: "実行ログ" });
  await expect(page.getByRole("button", { name: "ログの詳細を表示: entry-1500", exact: true })).toBeVisible();
  await viewport.evaluate((element) => { element.scrollTop = 16000; });
  await expect(page.getByText("過去のログを閲覧中")).toBeVisible();
  await expect(page.getByRole("button", { name: "ログの詳細を表示: entry-1000", exact: true })).toBeVisible();
  await page.evaluate(() => (window as any).__testBackend.append("live-arrival"));
  await expect(page.getByRole("button", { name: "ログの詳細を表示: entry-1000", exact: true })).toBeVisible();
  expect(await viewport.evaluate((element) => element.scrollTop)).toBe(16000);
  await page.getByRole("button", { name: "設定", exact: true }).click();
  await page.getByRole("button", { name: "ログ", exact: true }).click();
  await expect(page.getByRole("button", { name: "ログの詳細を表示: entry-1000", exact: true })).toBeVisible();
  await page.getByRole("button", { name: "最新のログへ" }).click();
  await expect(page.getByRole("button", { name: "ログの詳細を表示: live-arrival", exact: true })).toBeVisible();
  await viewport.evaluate((element) => { element.scrollTop = element.scrollHeight; });
  await expect(page.getByRole("button", { name: "ログの詳細を表示: entry-1", exact: true })).toBeVisible();
  expect(await page.getByRole("row").count()).toBeLessThan(100);
});

test("frontend console, errors, and promise rejections reach logs with exception details", async ({ page }) => {
  await page.evaluate(() => {
    console.warn("dependency console warning");
    window.dispatchEvent(new ErrorEvent("error", { error: new Error("frontend exception"), filename: "example.ts", lineno: 12 }));
    window.dispatchEvent(new PromiseRejectionEvent("unhandledrejection", { promise: Promise.resolve(), reason: new Error("promise rejection") }));
  });
  await page.getByRole("button", { name: "ログ", exact: true }).click();
  await expect(page.getByRole("button", { name: /ログの詳細を表示: Error: promise rejection/ })).toBeVisible();
  await page.getByRole("button", { name: /ログの詳細を表示: Error: frontend exception/ }).click();
  await expect(page.getByRole("dialog")).toContainText("example.ts:12");
  await expect(page.getByRole("dialog")).toContainText("frontend exception");
  await page.getByRole("button", { name: "閉じる", exact: true }).click();
  await expect(page.getByRole("button", { name: "ログの詳細を表示: dependency console warning", exact: true })).toBeVisible();
});

test("layout uses available space at the minimum window size", async ({ page }) => {
  await page.setViewportSize({ width: 720, height: 480 });
  await page.getByRole("button", { name: "ログ", exact: true }).click();
  await expect(page.getByRole("rowgroup", { name: "実行ログ" })).toBeVisible();
  await expect(page.getByRole("button", { name: "ログの詳細を表示: entry-1500", exact: true })).toBeVisible();
  expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBe(720);
  await page.screenshot({ path: "test-results/logs-small.png" });
});

test("maximum retention remains scrollable with a bounded number of rendered rows", async ({ page }) => {
  await page.evaluate(() => (window as any).__testBackend.useLargeDataset());
  await page.getByRole("button", { name: "ログ", exact: true }).click();
  await expect(page.getByRole("button", { name: "ログの詳細を表示: entry-999999", exact: true })).toBeVisible();
  await expect(page.getByText("該当 999,999 件", { exact: false })).toBeVisible();
  await page.screenshot({ path: "test-results/logs-desktop.png" });
  const viewport = page.getByRole("rowgroup", { name: "実行ログ" });
  await viewport.evaluate((element) => { element.scrollTop = element.scrollHeight; });
  await expect(page.getByRole("button", { name: "ログの詳細を表示: entry-1", exact: true })).toBeVisible();
  expect(await page.getByRole("row").count()).toBeLessThan(100);
});
