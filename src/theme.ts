import { getCurrentWindow } from "@tauri-apps/api/window";

export type ThemeChoice = "light" | "dark" | "auto";
export type Theme = "light" | "dark";

export const THEME_KEY = "phylopanel.theme";

const DARK = matchMedia("(prefers-color-scheme: dark)");

export function themeChoice(): ThemeChoice {
  const stored = localStorage.getItem(THEME_KEY);
  return stored === "light" || stored === "dark" || stored === "auto" ? stored : "auto";
}

export function resolve(choice: ThemeChoice): Theme {
  if (choice === "auto") return DARK.matches ? "dark" : "light";
  return choice;
}

/**
 * The native title bar is painted by Windows, so it has to be told about the choice as
 * well; outside a Tauri window (plain `vite` preview) that call simply fails.
 */
function syncWindowFrame(choice: ThemeChoice) {
  void getCurrentWindow()
    .setTheme(choice === "auto" ? null : choice)
    .catch(() => {});
}

export function applyTheme(choice: ThemeChoice) {
  localStorage.setItem(THEME_KEY, choice);
  document.documentElement.dataset.themeChoice = choice;
  document.documentElement.dataset.theme = resolve(choice);
  syncWindowFrame(choice);
}

DARK.addEventListener("change", () => {
  if (themeChoice() === "auto") {
    document.documentElement.dataset.theme = resolve("auto");
    syncWindowFrame("auto");
  }
});
