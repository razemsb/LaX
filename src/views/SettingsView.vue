<script setup lang="ts">
import { reactive, watch } from "vue";
import { useAppStore } from "@/stores/app";
import type { LaxConfig } from "@/types";
import { THEMES } from "@/lib/themes";

const store = useAppStore();
const form = reactive<LaxConfig>({
  documentRoot: "www",
  tld: "localhost",
  autoVhost: false,
  webServer: "apache",
  apachePort: 80,
  nginxPort: 80,
  mysqlPort: 3306,
  phpVersion: "php-trash-8.2",
  mysqlVersion: "mariadb-10.11.13",
  nginxVersion: "nginx-1.14.0",
  apacheVersion: "Apache24",
  phpCgiPorts: [9003, 9004],
  autoStart: false,
  mysqlEnabled: true,
  theme: "noir",
  dbAdmin: "phpmyadmin",
  startWeb: true,
  startMailpit: true,
  startDbGate: false,
});

watch(
  () => store.snap?.config,
  (cfg) => {
    if (!cfg) return;
    Object.assign(form, cfg);
    form.startWeb = cfg.startWeb ?? true;
    form.startMailpit = cfg.startMailpit ?? true;
    form.startDbGate = cfg.startDbGate ?? cfg.dbAdmin === "dbgate";
  },
  { immediate: true },
);

async function save() {
  await store.saveConfig({ ...form, theme: store.snap?.config.theme ?? form.theme });
}

async function pickDbAdmin(id: "phpmyadmin" | "dbgate") {
  if (id === "dbgate" && !store.snap?.dbgateAvailable) {
    await store.installDbGate();
    return;
  }
  await store.setDbAdmin(id);
}
</script>

<template>
  <div v-if="store.snap" class="grid max-w-4xl grid-cols-1 gap-5 xl:grid-cols-2 xl:gap-6">
    <section class="surface space-y-4 p-4 sm:p-5 xl:col-span-2">
      <div>
        <div class="eyebrow">тема</div>
        <p class="mt-1 text-xs text-muted">меняется сразу, стек не трогает</p>
      </div>
      <div class="grid grid-cols-2 gap-2 sm:grid-cols-3">
        <button
          v-for="t in THEMES"
          :key="t.id"
          type="button"
          class="theme-card"
          :aria-pressed="(store.snap.config.theme || 'noir') === t.id"
          @click="store.setTheme(t.id)"
        >
          <div class="mb-3 flex h-9 overflow-hidden rounded-lg">
            <span class="flex-1" :style="{ background: t.swatch[0] }" />
            <span class="flex-1" :style="{ background: t.swatch[1] }" />
            <span class="w-7" :style="{ background: t.swatch[2] }" />
          </div>
          <div class="flex items-center justify-between gap-2">
            <div class="min-w-0">
              <div class="truncate text-sm">{{ t.name }}</div>
              <div class="truncate text-[11px] text-muted">{{ t.hint }}</div>
            </div>
            <span v-if="t.experimental" class="shrink-0 rounded-md bg-lift px-1.5 py-0.5 text-[10px] text-muted">эксп.</span>
          </div>
        </button>
      </div>
    </section>

    <form class="surface space-y-5 p-4 sm:p-5" @submit.prevent="save">
      <label class="block">
        <div class="mb-2 eyebrow">корень сайтов</div>
        <input v-model="form.documentRoot" class="field" />
      </label>
      <label class="block">
        <div class="mb-2 eyebrow">веб-сервер</div>
        <select v-model="form.webServer" class="field">
          <option value="apache">Apache</option>
          <option value="nginx">Nginx</option>
        </select>
      </label>
      <div>
        <div class="mb-2 eyebrow">панель баз</div>
        <div class="grid grid-cols-2 gap-2">
          <button
            type="button"
            class="theme-card"
            :aria-pressed="(store.snap.config.dbAdmin || 'phpmyadmin') !== 'dbgate'"
            @click="pickDbAdmin('phpmyadmin')"
          >
            <div class="text-sm">phpMyAdmin</div>
            <div class="mt-1 text-[11px] text-muted">в zip, через Apache / Nginx</div>
          </button>
          <button
            type="button"
            class="theme-card"
            :aria-pressed="store.snap.config.dbAdmin === 'dbgate'"
            :disabled="store.busy"
            @click="pickDbAdmin('dbgate')"
          >
            <div class="text-sm">DbGate</div>
            <div class="mt-1 text-[11px] text-muted">
              {{ store.snap.dbgateAvailable ? "веб, порт 8030" : "не в zip · скачать ~350 МБ" }}
            </div>
          </button>
        </div>
        <p class="mt-2 text-[11px] text-muted">
          phpMyAdmin едет в релизе. DbGate ставится кнопкой выше, один раз, после установки или обновления.
        </p>
        <button
          v-if="!store.snap.dbgateAvailable"
          type="button"
          class="btn-accent mt-3 rounded-lg px-4 py-2 text-sm font-medium"
          :disabled="store.busy"
          @click="store.installDbGate()"
        >
          Скачать DbGate
        </button>
      </div>
      <div class="grid grid-cols-1 gap-3 sm:grid-cols-3">
        <label class="text-xs text-muted">Apache<input v-model.number="form.apachePort" type="number" class="field mt-1" /></label>
        <label class="text-xs text-muted">Nginx<input v-model.number="form.nginxPort" type="number" class="field mt-1" /></label>
        <label class="text-xs text-muted">MySQL<input v-model.number="form.mysqlPort" type="number" class="field mt-1" /></label>
      </div>
      <div>
        <div class="mb-2 eyebrow">кнопка «Запустить все»</div>
        <p class="mb-3 text-[11px] text-muted">только отмеченные службы. phpMyAdmin идёт через веб-сервер, отдельный процесс не нужен.</p>
        <div class="space-y-2">
          <label class="flex items-center gap-2 text-sm">
            <input v-model="form.startWeb" type="checkbox" class="accent-[var(--accent)]" />
            Веб-сервер ({{ form.webServer === "nginx" ? "Nginx" : "Apache" }} + PHP)
          </label>
          <label class="flex items-center gap-2 text-sm">
            <input v-model="form.mysqlEnabled" type="checkbox" class="accent-[var(--accent)]" />
            MariaDB
          </label>
          <label class="flex items-center gap-2 text-sm">
            <input v-model="form.startMailpit" type="checkbox" class="accent-[var(--accent)]" />
            Mailpit
          </label>
          <label class="flex items-center gap-2 text-sm" :class="{ 'opacity-50': !store.snap.dbgateAvailable }">
            <input
              v-model="form.startDbGate"
              type="checkbox"
              class="accent-[var(--accent)]"
              :disabled="!store.snap.dbgateAvailable"
            />
            DbGate
          </label>
        </div>
        <p v-if="form.dbAdmin !== 'dbgate' && form.startDbGate" class="mt-2 text-[11px] text-muted">
          Панель баз — phpMyAdmin, но DbGate всё равно поднимется. Сними галку, если не нужен.
        </p>
      </div>
      <label class="flex items-center gap-2 text-sm">
        <input v-model="form.autoStart" type="checkbox" class="accent-[var(--accent)]" />
        При открытии LaX сразу жать «Запустить все»
      </label>
      <button class="btn-accent rounded-lg px-4 py-2 text-sm font-medium" :disabled="store.busy">Сохранить</button>
    </form>

    <div class="space-y-3">
      <div class="eyebrow">конфиги</div>
      <div class="surface divide-y divide-line overflow-hidden">
        <button type="button" class="flex w-full items-center justify-between px-5 py-3.5 text-left text-sm hover:bg-lift" @click="store.openIni('php')">
          php.ini <span class="text-xs text-muted">PHP</span>
        </button>
        <button type="button" class="flex w-full items-center justify-between px-5 py-3.5 text-left text-sm hover:bg-lift" @click="store.openIni('mysql')">
          my.ini <span class="text-xs text-muted">MariaDB</span>
        </button>
        <button type="button" class="flex w-full items-center justify-between px-5 py-3.5 text-left text-sm hover:bg-lift" @click="store.openIni('apache')">
          httpd.conf <span class="text-xs text-muted">Apache</span>
        </button>
        <button type="button" class="flex w-full items-center justify-between px-5 py-3.5 text-left text-sm hover:bg-lift" @click="store.openIni('nginx')">
          nginx.conf <span class="text-xs text-muted">Nginx</span>
        </button>
      </div>
    </div>

    <section class="surface space-y-4 p-4 sm:p-5 xl:col-span-2">
      <div class="flex flex-wrap items-end justify-between gap-3">
        <div>
          <div class="eyebrow">о программе</div>
          <div class="mt-1 text-lg font-medium">LaX v{{ store.snap.appVersion }}</div>
          <p class="mt-1 text-sm text-muted">портативный локальный стек</p>
        </div>
        <button class="btn-ghost rounded-lg px-4 py-2 text-sm" :disabled="store.busy" @click="store.checkUpdate()">
          Проверить обновления
        </button>
      </div>
      <p v-if="store.snap.update" class="text-sm">
        Есть v{{ store.snap.update.version }}. Можно поставить сверху, www и базы не затрутся.
      </p>
      <div class="flex flex-wrap gap-2">
        <button class="btn-ghost rounded-lg px-4 py-2 text-sm" @click="store.openUrl(store.snap.repoUrl)">репозиторий</button>
        <button class="btn-ghost rounded-lg px-4 py-2 text-sm" @click="store.openUrl(store.snap.issuesUrl)">задачи</button>
        <button class="btn-accent rounded-lg px-4 py-2 text-sm" @click="store.openUrl(store.snap.feedbackUrl)">фидбек / баг</button>
      </div>
    </section>
  </div>
</template>
