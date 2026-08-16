<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { RouterLink } from "vue-router";
import { Database, FolderOpen, Globe, Inbox, Terminal } from "lucide-vue-next";
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
  mailpit: "Mailpit",
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
</script>

<template>
  <div v-if="store.snap" class="space-y-4 lg:space-y-8">
    <section class="grid grid-cols-2 gap-3 lg:grid-cols-5">
      <button
        v-for="s in store.snap.services"
        :key="s.id"
        class="surface min-w-0 p-4 text-left transition hover:bg-panel-2 lg:p-5"
        :disabled="store.busy"
        @click="toggle(s.id, s.running)"
      >
        <div class="mb-4 flex items-center justify-between gap-2 lg:mb-8">
          <div class="shrink-0 text-xs text-muted">{{ s.port ? `:${s.port}` : "—" }}</div>
          <StatusPill :on="s.running" />
        </div>
        <div class="truncate text-base font-medium lg:text-lg">{{ names[s.id] ?? s.name }}</div>
        <div class="mt-1 truncate text-xs text-muted">{{ s.version }}</div>
      </button>
    </section>

    <section class="grid grid-cols-1 gap-3 lg:grid-cols-[1.4fr_1fr]">
      <div class="surface p-4 lg:p-6">
        <div class="text-[11px] uppercase tracking-wider text-muted">сайты</div>
        <p class="mt-2 max-w-md text-sm leading-relaxed text-muted">
          Проекты из <span class="text-text">www</span> открываются как
          <span class="text-text">http://localhost/папка/</span>
        </p>
        <div class="mt-5 flex flex-wrap gap-2">
          <button class="btn-accent rounded-lg px-4 py-2 text-sm" @click="store.openUrl(site)">localhost</button>
          <button class="btn-ghost rounded-lg px-4 py-2 text-sm" @click="store.openUrl(pmaUrl())">phpMyAdmin</button>
          <button class="btn-ghost rounded-lg px-4 py-2 text-sm" @click="store.openUrl('http://localhost:8025')">почта</button>
          <button class="btn-ghost rounded-lg px-4 py-2 text-sm" @click="store.openPath(www)">папка www</button>
        </div>
        <p v-if="!store.snap.mailpitAvailable" class="mt-3 text-xs text-muted">
          Mailpit не найден — письма mail() никуда не попадут. Запусти
          <span class="text-text">npm run fetch-tools</span>
        </p>
        <p v-else class="mt-3 text-xs text-muted">
          PHP mail() ловит Mailpit. Laravel: MAIL_HOST=127.0.0.1 MAIL_PORT=1025 MAIL_MAILER=smtp
        </p>
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
        <button class="flex w-full items-center gap-3 px-5 py-3.5 text-left text-sm hover:bg-panel-2" @click="store.openUrl('http://localhost:8025')">
          <Inbox :size="16" class="text-muted" /> Ящик Mailpit
        </button>
        <button class="flex w-full items-center gap-3 px-5 py-3.5 text-left text-sm hover:bg-panel-2" @click="store.openUrl(pmaUrl())">
          <Database :size="16" class="text-muted" /> База данных
        </button>
      </div>
    </section>

    <section class="surface p-4 lg:p-6">
      <div class="text-[11px] uppercase tracking-wider text-muted">базы MariaDB</div>
      <p v-if="!dbUp" class="mt-2 text-sm text-muted">Сначала запусти MariaDB.</p>
      <div v-else class="mt-4 space-y-4">
        <div class="flex flex-wrap gap-2">
          <input v-model="newDb" placeholder="имя_базы" class="field min-w-0 flex-1" @keydown.enter.prevent="createDb" />
          <button class="btn-accent shrink-0 rounded-lg px-4 py-2 text-sm" :disabled="store.busy || !newDb.trim()" @click="createDb">
            Создать
          </button>
        </div>
        <div class="flex flex-wrap items-center gap-2">
          <select v-model="selectedDb" class="field min-w-0 flex-1">
            <option disabled value="">выбери базу</option>
            <option v-for="d in databases" :key="d" :value="d">{{ d }}</option>
          </select>
          <button
            class="btn-ghost shrink-0 rounded-lg px-4 py-2 text-sm"
            :disabled="!selectedDb"
            @click="store.openUrl(pmaUrl(selectedDb))"
          >
            phpMyAdmin
          </button>
          <label class="btn-ghost shrink-0 cursor-pointer rounded-lg px-4 py-2 text-sm" :class="{ 'opacity-50': !selectedDb || importing }">
            {{ importing ? "импорт…" : "импорт .sql" }}
            <input type="file" accept=".sql,.txt" class="hidden" :disabled="!selectedDb || importing" @change="onSql" />
          </label>
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
      <div class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
        <button
          v-for="p in store.snap.projects.slice(0, 6)"
          :key="p.name"
          class="surface min-w-0 p-4 text-left hover:bg-panel-2"
          @click="store.openUrl(p.url)"
        >
          <div class="font-medium">{{ p.name }}</div>
          <div class="mt-1 truncate text-xs text-muted">{{ p.url.replace("http://", "") }}</div>
          <div class="mt-2 text-[10px] uppercase tracking-wider text-muted">{{ p.kind }}</div>
        </button>
      </div>
    </section>
  </div>
</template>
