import type { CtxItem } from "@/composables/contextMenu";
import type { ProjectInfo } from "@/types";

export type ProjectCmd = { label: string; action: string; hint: string; accent?: boolean };

const primary = ["dev", "start", "serve", "preview", "build"];

export function projectCommands(p: ProjectInfo): ProjectCmd[] {
  const out: ProjectCmd[] = [];
  if (p.hasPackage) {
    out.push({
      label: "npm install",
      action: "npm-install",
      hint: "npm install",
      accent: !p.hasNodeModules,
    });
  }
  if (p.hasComposer) {
    out.push({
      label: "composer install",
      action: "composer-install",
      hint: "composer install",
      accent: !p.hasVendor,
    });
  }
  const have = new Set(p.scripts);
  for (const s of primary) {
    if (!have.has(s)) continue;
    out.push({
      label: `npm run ${s}`,
      action: "npm-run:" + s,
      hint: "npm run " + s,
      accent: s === "dev" || s === "start",
    });
  }
  for (const s of p.scripts) {
    if (primary.includes(s) || s.startsWith("pre") || s.startsWith("post")) continue;
    out.push({ label: `npm run ${s}`, action: "npm-run:" + s, hint: "npm run " + s });
  }
  return out;
}

export function projectContextItems(
  p: ProjectInfo,
  act: {
    openUrl: (url: string) => void;
    openPath: (path: string) => void;
    openTerminal: (path: string) => void;
    openVscode: (path: string) => void;
    run: (path: string, action: string) => void;
  },
): CtxItem[] {
  const items: CtxItem[] = [
    { label: "Открыть сайт", run: () => act.openUrl(p.url) },
  ];
  if (p.kind === "vite") {
    items.push({ label: "Vite :5173", run: () => act.openUrl("http://localhost:5173/") });
  }
  items.push(
    { label: "Папка", run: () => act.openPath(p.path) },
    { label: "Терминал", run: () => act.openTerminal(p.path) },
    { label: "VS Code", run: () => act.openVscode(p.path) },
  );
  const cmds = projectCommands(p);
  if (cmds.length) {
    items.push({ type: "sep" });
    for (const c of cmds) {
      items.push({
        label: c.label,
        accent: c.accent,
        run: () => act.run(p.path, c.action),
      });
    }
  }
  return items;
}
