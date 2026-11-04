import type { CtxItem } from "@/composables/contextMenu";
import type { ProjectInfo, TunnelInfo } from "@/types";
import { t } from "@/lib/i18n";

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
    share: (path: string) => void;
    unshare: (path: string) => void;
  },
  tunnel?: TunnelInfo | null,
): CtxItem[] {
  const items: CtxItem[] = [
    { label: t("proj.openSite"), run: () => act.openUrl(p.url) },
  ];
  if (p.kind === "vite") {
    items.push({ label: "Vite :5173", run: () => act.openUrl("http://localhost:5173/") });
  }
  items.push(
    { label: t("proj.ctxFolder"), run: () => act.openPath(p.path) },
    { label: t("proj.ctxTerminal"), run: () => act.openTerminal(p.path) },
    { label: "VS Code", run: () => act.openVscode(p.path) },
    { type: "sep" },
  );
  if (tunnel) {
    items.push({ label: t("share.stop"), accent: true, run: () => act.unshare(p.path) });
  } else {
    items.push({ label: t("share.start"), run: () => act.share(p.path) });
  }
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
