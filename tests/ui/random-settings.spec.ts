import { test, expect } from "@playwright/test";
import { mockBackend } from "./mockBackend";

test("fixed seed defaults on and persists after navigation", async ({ page }) => {
  await mockBackend(page);
  await page.goto("/");
  await page.getByRole("button", { name: "設定", exact: true }).click();
  const checkbox = page.getByRole("checkbox", { name: "乱数シードを固定する" });
  await expect(checkbox).toBeChecked();
  await checkbox.uncheck();
  await page.getByRole("button", { name: "保存", exact: true }).click();
  await expect(page.getByRole("status")).toHaveText("設定を保存しました。");
  await page.getByRole("button", { name: "メイン", exact: true }).click();
  await page.getByRole("button", { name: "設定", exact: true }).click();
  await expect(checkbox).not.toBeChecked();
});

test("failed save preserves the previous seed setting", async ({ page }) => {
  await mockBackend(page);
  await page.goto("/");
  await page.getByRole("button", { name: "設定", exact: true }).click();
  const checkbox = page.getByRole("checkbox", { name: "乱数シードを固定する" });
  await checkbox.uncheck();
  await page.evaluate(() => (window as any).__testBackend.failSave());
  await page.getByRole("button", { name: "保存", exact: true }).click();
  await expect(page.getByRole("alert")).toBeVisible();
  await page.getByRole("button", { name: "メイン", exact: true }).click();
  await page.getByRole("button", { name: "設定", exact: true }).click();
  await expect(checkbox).toBeChecked();
});
