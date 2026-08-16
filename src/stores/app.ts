import { defineStore } from "pinia";
import { invoke } from "@tauri-apps/api/core";
import type { LaxConfig, PhpExtension, ProjectInfo, ServiceInfo, Snapshot } from "@/types";

export const useAppStore = defineStore("app", {
  state: () => ({
    snap: null as Snapshot | null,
    loading: false,
    busy: false,
    error: "" as string,
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
        this.error = "";
      } catch (e) {
        this.error = String(e);
      } finally {
        this.loading = false;
      }
    },
    async refreshStatus() {
      if (!this.snap) return;
      try {
        const services = await invoke<ServiceInfo[]>("status");
        this.snap.services = services;
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
    async createProject(name: string) {
      this.busy = true;
      this.error = "";
      try {
        const project = await invoke<ProjectInfo>("create_project", { name });
        await this.refresh();
        return project;
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
    async readLogs(which: string) {
      return invoke<string>("read_logs", { which });
    },
    async run(cmd: string, args: Record<string, unknown> = {}) {
      this.busy = true;
      this.error = "";
      try {
        this.snap = await invoke<Snapshot>(cmd, args);
      } catch (e) {
        this.error = String(e);
        throw e;
      } finally {
        this.busy = false;
      }
    },
  },
});
