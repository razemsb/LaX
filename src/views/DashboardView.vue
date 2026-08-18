<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { RouterLink } from "vue-router";
import {
  Blocks,
  Database,
  FolderOpen,
  Globe,
  Inbox,
  Layers,
  Server,
  Terminal,
} from "lucide-vue-next";
import { useAppStore } from "@/stores/app";
import StatusPill from "@/components/StatusPill.vue";
import { showContextMenu } from "@/composables/contextMenu";
import { projectContextItems } from "@/lib/projectCommands";
import type { ProjectInfo } from "@/types";

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
  mailpit: "Mailpit",
};

const icons: Record<string, typeof Server> = {
  apache: Server,
  nginx: Layers,
  mariadb: Database,
  php: Blocks,
  mailpit: Inbox,
};

const databases = ref<string[]>([]);
const newDb = ref("");
const selectedDb = ref("");
const importing = ref(false);
const dbNote = ref("");
const dbUp = computed(() => store.db?.running === true);

function pmaUrl(name?: string) {
  const base = site.value.replace(/\/$/, "");
  if (!name) return `${base}/phpmyadmin/`;
  return `${base}/phpmyadmin/index.php?route=/database/structure&db=${encodeURIComponent(name)}`;
}

async function loadDbs() {
  dbNote.value = "";
  if (!dbUp.value) {
    databases.value = [];
    return;
  }
  try {
    databases.value = await store.listDatabases();
    if (selectedDb.value && !databases.value.includes(selectedDb.value)) {
      selectedDb.value = "";
    }
    if (!selectedDb.value && databases.value.length) {
      selectedDb.value = databases.value[0];
    }
  } catch (e) {
    databases.value = [];
    dbNote.value = String(e);
  }
}

watch(dbUp, loadDbs, { immediate: true });

async function createDb() {
  const name = newDb.value.trim();
  if (!name) return;
  databases.value = await store.createDatabase(name);
  selectedDb.value = name;
  newDb.value = "";
}

async function onSql(ev: Event) {
  const input = ev.target as HTMLInputElement;
  const file = input.files?.[0];
  if (!file || !selectedDb.value) return;
  importing.value = true;
  dbNote.value = "";
  try {
    const sql = await file.text();
    await store.importSql(selectedDb.value, sql);
    dbNote.value = `импортировано в ${selectedDb.value}`;
  } catch (e) {
    dbNote.value = String(e);
  } finally {
    importing.value = false;
    input.value = "";
  }
}

async function toggle(id: string, running: boolean) {
  if (running) await store.stopService(id);
  else await store.startService(id);
}

function onProjectMenu(e: MouseEvent, p: ProjectInfo) {
  showContextMenu(e, projectContextItems(p, {
    openUrl: (url) => store.openUrl(url),
    openPath: (path) => store.openPath(path),
    openTerminal: (path) => store.openTerminal(path),
    openVscode: (path) => store.openVscode(path),
    run: (path, action) => store.runProjectAction(path, action),
  }));
}
</script>

<template>
  <div v-if="store.snap" class="space-y-4 xl:space-y-7">
    <section class="grid grid-cols-2 gap-2 sm:gap-3 md:grid-cols-3 xl:grid-cols-5">
      <button
        v-for="s in store.snap.services"
        :key="s.id"
        class="surface svc-card flex min-h-[108px] min-w-0 flex-col p-3 text-left sm:min-h-[124px] sm:p-4"
        :class="s.running ? 'svc-on' : ''"
        :disabled="store.busy"
        @click="toggle(s.id, s.running)"
      >
        <div class="mb-4 flex items-center justify-between gap-2">
          <span class="icon-well" :class="s.running ? 'text-ok bg-ok/10' : ''">
            <component :is="icons[s.id] ?? Server" :size="15" />
          </span>
          <StatusPill :on="s.running" />
        </div>
        <div class="mt-auto min-w-0">
          <div class="truncate text-[15px] font-medium sm:text-base">{{ names[s.id] ?? s.name }}</div>
          <div class="mt-1 flex items-center justify-between gap-2 text-[11px] text-muted">
            <span>{{ s.port ? `:${s.port}` : "—" }}</span>
            <span class="truncate">{{ s.version }}</span>
          </div>
        </div>
      </button>
    </section>

    <section class="grid grid-cols-1 gap-3 xl:grid-cols-[1.5fr_1fr]">
      <div class="surface p-4 sm:p-5 xl:p-6">
        <div class="eyebrow">сайты</div>
        <p class="mt-2 max-w-lg text-sm leading-relaxed text-muted">
          Проекты из <span class="text-text">www</span> открываются как
        </p>
        <code class="mt-2 inline-block rounded-md bg-lift px-2 py-1 text-xs text-text">http://localhost/папка/</code>
        <div class="mt-5 flex flex-wrap gap-2">
          <button class="btn-accent rounded-lg px-4 py-2 text-sm" @click="store.openUrl(site)">localhost</button>
          <button class="btn-ghost rounded-lg px-4 py-2 text-sm" @click="store.openUrl(pmaUrl())">phpMyAdmin</button>
          <button class="btn-ghost rounded-lg px-4 py-2 text-sm" @click="store.openUrl('http://localhost:8025')">почта</button>
          <button class="btn-ghost rounded-lg px-4 py-2 text-sm" @click="store.openPath(www)">папка www</button>
        </div>
        <p v-if="!store.snap.mailpitAvailable" class="mt-3 text-xs leading-relaxed text-muted">
          Mailpit не найден — письма mail() никуда не попадут. Запусти
          <span class="text-text">npm run fetch-tools</span>
        </p>
        <p v-else class="mt-3 text-xs leading-relaxed text-muted">
          PHP mail() ловит Mailpit. Laravel: MAIL_HOST=127.0.0.1 MAIL_PORT=1025 MAIL_MAILER=smtp
        </p>
      </div>
      <div class="surface grid grid-cols-1 overflow-hidden p-1 sm:grid-cols-2 xl:grid-cols-1">
        <button class="flex items-center gap-3 rounded-xl px-4 py-3 text-left text-sm hover:bg-lift" @click="store.openPath(store.snap.root)">
          <span class="icon-well"><FolderOpen :size="15" /></span> Корень LaX
        </button>
        <button class="flex items-center gap-3 rounded-xl px-4 py-3 text-left text-sm hover:bg-lift" @click="store.openTerminal(www)">
          <span class="icon-well"><Terminal :size="15" /></span> Терминал
        </button>
        <button class="flex items-center gap-3 rounded-xl px-4 py-3 text-left text-sm hover:bg-lift" @click="store.openUrl(site)">
          <span class="icon-well"><Globe :size="15" /></span> Браузер
        </button>
        <button class="flex items-center gap-3 rounded-xl px-4 py-3 text-left text-sm hover:bg-lift" @click="store.openUrl('http://localhost:8025')">
          <span class="icon-well"><Inbox :size="15" /></span> Ящик Mailpit
        </button>
        <button class="flex items-center gap-3 rounded-xl px-4 py-3 text-left text-sm hover:bg-lift sm:col-span-2 xl:col-span-1" @click="store.openUrl(pmaUrl())">
          <span class="icon-well"><Database :size="15" /></span> База данных
        </button>
      </div>
    </section>

    <section class="surface p-4 sm:p-5 xl:p-6">
      <div class="eyebrow">базы MariaDB</div>
      <p v-if="!dbUp" class="mt-2 text-sm text-muted">Сначала запусти MariaDB.</p>
      <div v-else class="mt-4 space-y-3">
        <div class="flex flex-col gap-2 sm:flex-row">
          <input v-model="newDb" placeholder="имя_базы" class="field min-w-0 flex-1" @keydown.enter.prevent="createDb" />
          <button class="btn-accent shrink-0 rounded-lg px-4 py-2 text-sm" :disabled="store.busy || !newDb.trim()" @click="createDb">
            Создать
          </button>
        </div>
        <div class="flex flex-col gap-2 sm:flex-row sm:items-center">
          <select v-model="selectedDb" class="field min-w-0 flex-1">
            <option disabled value="">выбери базу</option>
            <option v-for="d in databases" :key="d" :value="d">{{ d }}</option>
          </select>
          <div class="flex gap-2">
            <button
              class="btn-ghost min-w-0 flex-1 rounded-lg px-4 py-2 text-sm sm:flex-none"
              :disabled="!selectedDb"
              @click="store.openUrl(pmaUrl(selectedDb))"
            >
              phpMyAdmin
            </button>
            <label class="btn-ghost flex-1 cursor-pointer rounded-lg px-4 py-2 text-center text-sm sm:flex-none" :class="{ 'opacity-50': !selectedDb || importing }">
              {{ importing ? "импорт…" : "импорт .sql" }}
              <input type="file" accept=".sql,.txt" class="hidden" :disabled="!selectedDb || importing" @change="onSql" />
            </label>
          </div>
        </div>
        <p v-if="dbNote" class="text-xs text-muted">{{ dbNote }}</p>
      </div>
    </section>

    <section>
      <div class="mb-3 flex items-center justify-between">
        <h3 class="text-sm font-medium">Проекты</h3>
        <RouterLink to="/projects" class="text-xs text-muted hover:text-text">все</RouterLink>
      </div>
      <div v-if="!store.snap.projects.length" class="text-sm text-muted">Пока пусто.</div>
      <div class="grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-3">
        <button
          v-for="p in store.snap.projects.slice(0, 6)"
          :key="p.name"
          data-ctx
          class="surface min-w-0 p-4 text-left"
          @click="store.openUrl(p.url)"
          @contextmenu="onProjectMenu($event, p)"
        >
          <div class="font-medium">{{ p.name }}</div>
          <div class="mt-1 truncate text-xs text-muted">{{ p.url.replace("http://", "") }}</div>
          <div class="mt-3 inline-flex rounded-md bg-lift px-2 py-0.5 text-[10px] uppercase tracking-wider text-muted">{{ p.kind }}</div>
        </button>
      </div>
    </section>
  </div>
</template>
