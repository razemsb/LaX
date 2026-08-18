export type ThemeId = "noir" | "paper" | "midnight" | "quartz" | "glass" | "obsidian";

export const THEMES: {
  id: ThemeId;
  name: string;
  hint: string;
  experimental?: boolean;
  swatch: [string, string, string];
}[] = [
  { id: "noir", name: "Noir", hint: "матовый чёрный", swatch: ["#09090b", "#1a1a1c", "#e02430"] },
  { id: "paper", name: "Paper", hint: "тёплая бумага", swatch: ["#f6f1e6", "#fffdf8", "#c42b2b"] },
  { id: "midnight", name: "Midnight", hint: "ночной индиго", swatch: ["#070b14", "#152038", "#e24b56"] },
  { id: "quartz", name: "Quartz", hint: "холодный белый", swatch: ["#eef1f6", "#ffffff", "#d21f2b"] },
  { id: "glass", name: "Glass", hint: "холодный индиго", swatch: ["#10182a", "#182238", "#e02430"] },
  { id: "obsidian", name: "Obsidian", hint: "тёмный мох", swatch: ["#0c100e", "#161c18", "#e02430"] },
];

const ids = new Set(THEMES.map((t) => t.id));

export function applyTheme(id: string) {
  const next = ids.has(id as ThemeId) ? id : "noir";
  const root = document.documentElement;
  if (root.getAttribute("data-theme") === next) return;
  root.removeAttribute("data-theme");
  void root.offsetWidth;
  root.setAttribute("data-theme", next);
  root.style.colorScheme = next === "paper" || next === "quartz" ? "light" : "dark";
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
