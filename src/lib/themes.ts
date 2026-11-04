export type ThemeId = "noir" | "paper" | "midnight" | "quartz" | "glass" | "obsidian";

export const THEMES: {
  id: ThemeId;
  name: string;
  experimental?: boolean;
  swatch: [string, string, string];
}[] = [
  { id: "noir", name: "Noir", swatch: ["#09090b", "#1a1a1c", "#e02430"] },
  { id: "paper", name: "Paper", swatch: ["#e9dfcc", "#fbf5e9", "#bf2e2e"] },
  { id: "midnight", name: "Midnight", swatch: ["#04070f", "#111d33", "#e5484d"] },
  { id: "quartz", name: "Quartz", swatch: ["#eef1f6", "#ffffff", "#d21f2b"] },
  { id: "glass", name: "Glass", swatch: ["#101a2e", "#243356", "#e02430"] },
  { id: "obsidian", name: "Obsidian", swatch: ["#080d0b", "#1a241f", "#e02430"] },
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
