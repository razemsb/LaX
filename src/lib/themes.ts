export type ThemeId = "noir" | "paper" | "midnight" | "quartz" | "glass" | "obsidian";

export const THEMES: {
  id: ThemeId;
  name: string;
  hint: string;
  experimental?: boolean;
  swatch: [string, string, string];
}[] = [
  { id: "noir", name: "Noir", hint: "чёрный минимализм", swatch: ["#0a0a0b", "#161616", "#e02430"] },
  { id: "paper", name: "Paper", hint: "светлая бумага", swatch: ["#f4f1ea", "#fffdf8", "#c42b2b"] },
  { id: "midnight", name: "Midnight", hint: "ночной синий", swatch: ["#080c16", "#121a2c", "#e24b56"] },
  { id: "quartz", name: "Quartz", hint: "холодный светлый", swatch: ["#eef0f4", "#ffffff", "#d21f2b"] },
  { id: "glass", name: "Glass", hint: "жидкое стекло", experimental: true, swatch: ["#07080e", "#3a4258", "#ff5a66"] },
  { id: "obsidian", name: "Obsidian", hint: "тёмное стекло", experimental: true, swatch: ["#050605", "#1c2620", "#e02430"] },
];

const ids = new Set(THEMES.map((t) => t.id));

export function applyTheme(id: string) {
  const next = ids.has(id as ThemeId) ? id : "noir";
  document.documentElement.dataset.theme = next;
  try {
    localStorage.setItem("lax-theme", next);
  } catch {
    /* ignore */
  }
}

export function bootTheme() {
  try {
    const saved = localStorage.getItem("lax-theme");
    if (saved) applyTheme(saved);
  } catch {
    /* ignore */
  }
}
