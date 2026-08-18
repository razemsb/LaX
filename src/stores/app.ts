import { defineStore } from "pinia";
import { invoke } from "@tauri-apps/api/core";
import type { LaxConfig, PhpExtension, PhpQuickPatch, PhpQuickSettings, Snapshot } from "@/types";
import { applyTheme } from "@/lib/themes";

export const useAppStore = defineStore("app", {
  state: () => ({
    snap: null as Snapshot | null,
    loading: false,
    busy: false,
    error: "" as string,
    dismissedPort: "" as string,
    updateProgress: "" as string,
  }),
  getters: {
    runningCount: (s) => s.snap?.services.filter((x) => x.running).length ?? 0,
    web: (s) => s.snap?.services.find((x) => x.id === s.snap?.config.webServer),
    php: (s) => s.snap?.services.find((x) => x.id === "php"),
    db: (s) => s.snap?.services.find((x) => x.id === "mariadb"),
  },
  actions: {
    async refresh() {
      this.loading = true;
      try {
        this.snap = await invoke<Snapshot>("snapshot");
        if (this.snap?.config.theme) applyTheme(this.snap.config.theme);
        this.error = "";
      } catch (e) {
        this.error = String(e);
      } finally {
        this.loading = false;
      }
    },
    async refreshStatus() {
      try {
        this.snap = await invoke<Snapshot>("snapshot");
        if (this.snap?.config.theme) applyTheme(this.snap.config.theme);
      } catch {
        /* keep last snap */
      }
    },
    async startAll() {
      return this.run("start_all");
    },
    async stopAll() {
      return this.run("stop_all");
    },
    async startService(id: string) {
      return this.run("start_service", { id });
    },
    async stopService(id: string) {
      return this.run("stop_service", { id });
    },
    async switchPhp(version: string) {
      return this.run("switch_php", { version });
    },
    async saveConfig(config: LaxConfig) {
      return this.run("save_config", { config });
    },
    async setTheme(theme: string) {
      applyTheme(theme);
      this.error = "";
      try {
        this.snap = await invoke<Snapshot>("set_theme", { theme });
      } catch (e) {
        this.error = String(e);
        throw e;
      }
    },
    async createProject(name: string, kind = "php") {
      this.busy = true;
      this.error = "";
      try {
        this.snap = await invoke<Snapshot>("create_project", { name, kind });
      } catch (e) {
        this.error = String(e);
        throw e;
      } finally {
        this.busy = false;
      }
    },
    async openUrl(url: string) {
      await invoke("open_url", { url });
    },
    async openPath(path: string) {
      await invoke("open_path", { path });
    },
    async openTerminal(path: string) {
      await invoke("open_terminal", { path });
    },
    async openVscode(path: string) {
      await invoke("open_vscode", { path });
    },
    async runProjectAction(path: string, action: string) {
      this.error = "";
      try {
        await invoke("run_project_action", { path, action });
      } catch (e) {
        this.error = String(e);
        throw e;
      }
    },
    async openIni(which: string) {
      await invoke("open_ini", { which });
    },
    async listPhpExtensions() {
      return invoke<PhpExtension[]>("list_php_extensions");
    },
    async setPhpExtension(name: string, enabled: boolean) {
      this.error = "";
      try {
        await invoke("set_php_extension", { name, enabled });
      } catch (e) {
        this.error = String(e);
        throw e;
      }
    },
    async phpQuickSettings() {
      return invoke<PhpQuickSettings>("php_quick_settings");
    },
    async setPhpQuickSettings(patch: PhpQuickPatch) {
      this.error = "";
      try {
        return await invoke<PhpQuickSettings>("set_php_quick_settings", { patch });
      } catch (e) {
        this.error = String(e);
        throw e;
      }
    },
    async switchWebPort(port: number) {
      return this.run("switch_web_port", { port });
    },
    async listDatabases() {
      return invoke<string[]>("list_databases");
    },
    async createDatabase(name: string) {
      this.error = "";
      try {
        return await invoke<string[]>("create_database", { name });
      } catch (e) {
        this.error = String(e);
        throw e;
      }
    },
    async importSql(dbName: string, sql: string) {
      this.error = "";
      this.busy = true;
      try {
        await invoke("import_sql", { dbName, sql });
      } catch (e) {
        this.error = String(e);
        throw e;
      } finally {
        this.busy = false;
      }
    },
    async readLogs(which: string) {
      return invoke<string>("read_logs", { which });
    },
    async checkUpdate() {
      return this.run("check_update");
    },
    async applyUpdate() {
      this.busy = true;
      this.error = "";
      this.updateProgress = "скачиваю обновление…";
      try {
        await invoke("apply_update");
        await this.refresh();
      } catch (e) {
        const msg = String(e);
        if (/webview|closed|cancelled/i.test(msg)) return;
        this.error = msg;
      } finally {
        this.busy = false;
        this.updateProgress = "";
      }
    },
    clearError() {
      this.error = "";
    },
    async dismissNotice(which: "message" | "update" | "port") {
      if (which === "port" && this.snap?.portConflict) {
        this.dismissedPort = `${this.snap.portConflict.port}:${this.snap.portConflict.pid}`;
      }
      try {
        this.snap = await invoke<Snapshot>("dismiss_notice", { which });
      } catch {
        if (!this.snap) return;
        if (which === "update") this.snap.update = null;
        if (which === "message") this.snap.message = null;
        if (which === "port") this.snap.portConflict = null;
      }
    },
    async run(cmd: string, args: Record<string, unknown> = {}) {
      this.busy = true;
      this.error = "";
      try {
        this.snap = await invoke<Snapshot>(cmd, args);
        this.dismissedPort = "";
      } catch (e) {
        this.error = String(e);
        try {
          this.snap = await invoke<Snapshot>("snapshot");
        } catch {
          /* ignore */
        }
        throw e;
      } finally {
        this.busy = false;
      }
    },
  },
});
