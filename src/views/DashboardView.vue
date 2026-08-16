<script setup lang="ts">
import { computed } from "vue";
import { RouterLink } from "vue-router";
import { Database, FolderOpen, Globe, Terminal } from "lucide-vue-next";
import { useAppStore } from "@/stores/app";
import StatusPill from "@/components/StatusPill.vue";

const store = useAppStore();
const www = computed(() => {
  if (!store.snap) return "";
  return `${store.snap.root}\\${store.snap.config.documentRoot}`.replaceAll("/", "\\");
});
const site = computed(() => {
  const port = store.snap?.config.webServer === "nginx"
    ? store.snap?.config.nginxPort
    : store.snap?.config.apachePort;
  return port && port !== 80 ? `http://localhost:${port}/` : "http://localhost/";
});

const names: Record<string, string> = {
  apache: "Apache",
  nginx: "Nginx",
  mariadb: "MariaDB",
  php: "PHP",
};

async function toggle(id: string, running: boolean) {
  if (running) await store.stopService(id);
  else await store.startService(id);
}
</script>

<template>
  <div v-if="store.snap" class="space-y-8">
    <section class="grid grid-cols-4 gap-3">
      <button
        v-for="s in store.snap.services"
        :key="s.id"
        class="surface p-5 text-left transition hover:bg-panel-2"
        :disabled="store.busy"
        @click="toggle(s.id, s.running)"
      >
        <div class="mb-8 flex items-center justify-between">
          <div class="text-xs text-muted">{{ s.port ? `:${s.port}` : "—" }}</div>
          <StatusPill :on="s.running" />
        </div>
        <div class="text-lg font-medium">{{ names[s.id] ?? s.name }}</div>
        <div class="mt-1 truncate text-xs text-muted">{{ s.version }}</div>
      </button>
    </section>

    <section class="grid grid-cols-[1.4fr_1fr] gap-3">
      <div class="surface p-6">
        <div class="text-[11px] uppercase tracking-wider text-muted">сайты</div>
        <p class="mt-2 max-w-md text-sm leading-relaxed text-muted">
          Проекты из <span class="text-text">www</span> открываются как
          <span class="text-text">http://localhost/папка/</span>
        </p>
        <div class="mt-5 flex flex-wrap gap-2">
          <button class="btn-accent rounded-lg px-4 py-2 text-sm" @click="store.openUrl(site)">localhost</button>
          <button class="btn-ghost rounded-lg px-4 py-2 text-sm" @click="store.openUrl(site + 'phpmyadmin/')">phpMyAdmin</button>
          <button class="btn-ghost rounded-lg px-4 py-2 text-sm" @click="store.openPath(www)">папка www</button>
        </div>
      </div>
      <div class="surface divide-y divide-line overflow-hidden p-0">
        <button class="flex w-full items-center gap-3 px-5 py-3.5 text-left text-sm hover:bg-panel-2" @click="store.openPath(store.snap.root)">
          <FolderOpen :size="16" class="text-muted" /> Корень LaX
        </button>
        <button class="flex w-full items-center gap-3 px-5 py-3.5 text-left text-sm hover:bg-panel-2" @click="store.openTerminal(www)">
          <Terminal :size="16" class="text-muted" /> Терминал
        </button>
        <button class="flex w-full items-center gap-3 px-5 py-3.5 text-left text-sm hover:bg-panel-2" @click="store.openUrl(site)">
          <Globe :size="16" class="text-muted" /> Браузер
        </button>
        <button class="flex w-full items-center gap-3 px-5 py-3.5 text-left text-sm hover:bg-panel-2" @click="store.openUrl(site + 'phpmyadmin/')">
          <Database :size="16" class="text-muted" /> База данных
        </button>
      </div>
    </section>

    <section>
      <div class="mb-3 flex items-center justify-between">
        <h3 class="text-sm font-medium">Проекты</h3>
        <RouterLink to="/projects" class="text-xs text-muted hover:text-text">все</RouterLink>
      </div>
      <div v-if="!store.snap.projects.length" class="text-sm text-muted">Пока пусто.</div>
      <div class="grid grid-cols-3 gap-3">
        <button
          v-for="p in store.snap.projects.slice(0, 6)"
          :key="p.name"
          class="surface p-4 text-left hover:bg-panel-2"
          @click="store.openUrl(p.url)"
        >
          <div class="font-medium">{{ p.name }}</div>
          <div class="mt-1 truncate text-xs text-muted">{{ p.url.replace("http://", "") }}</div>
        </button>
      </div>
    </section>
  </div>
</template>
