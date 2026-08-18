import type { LaxConfig } from "@/types";

export function dbAdminId(cfg?: Pick<LaxConfig, "dbAdmin"> | null) {
  return cfg?.dbAdmin === "dbgate" ? "dbgate" : "phpmyadmin";
}

export function dbAdminLabel(cfg?: Pick<LaxConfig, "dbAdmin"> | null) {
  return dbAdminId(cfg) === "dbgate" ? "DbGate" : "phpMyAdmin";
}

export function dbAdminUrl(site: string, cfg?: Pick<LaxConfig, "dbAdmin"> | null, db?: string) {
  if (dbAdminId(cfg) === "dbgate") {
    return "http://localhost:8030/";
  }
  const base = site.replace(/\/$/, "");
  if (!db) return `${base}/phpmyadmin/`;
  return `${base}/phpmyadmin/index.php?route=/database/structure&db=${encodeURIComponent(db)}`;
}
