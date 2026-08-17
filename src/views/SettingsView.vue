<script setup lang="ts">
import { reactive, watch } from "vue";
import { useAppStore } from "@/stores/app";
import type { LaxConfig } from "@/types";

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
});

watch(
  () => store.snap?.config,
  (cfg) => {
    if (cfg) Object.assign(form, cfg);
  },
  { immediate: true },
);

async function save() {
  await store.saveConfig({ ...form });
}
</script>

<template>
  <div v-if="store.snap" class="grid max-w-4xl grid-cols-1 gap-6 lg:grid-cols-2">
    <form class="surface space-y-5 p-4 lg:p-6" @submit.prevent="save">
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
      <div class="grid grid-cols-1 gap-3 sm:grid-cols-3">
        <label class="text-xs text-muted">Apache<input v-model.number="form.apachePort" type="number" class="field mt-1" /></label>
        <label class="text-xs text-muted">Nginx<input v-model.number="form.nginxPort" type="number" class="field mt-1" /></label>
        <label class="text-xs text-muted">MySQL<input v-model.number="form.mysqlPort" type="number" class="field mt-1" /></label>
      </div>
      <label class="flex items-center gap-2 text-sm">
        <input v-model="form.mysqlEnabled" type="checkbox" class="accent-[#e02430]" />
        Поднимать MariaDB вместе со стеком
      </label>
      <label class="flex items-center gap-2 text-sm">
        <input v-model="form.autoStart" type="checkbox" class="accent-[#e02430]" />
        При открытии LaX сразу запускать Apache и MariaDB
      </label>
      <button class="btn-accent rounded-lg px-4 py-2 text-sm font-medium" :disabled="store.busy">Сохранить</button>
    </form>

    <div class="space-y-3">
      <div class="eyebrow">конфиги</div>
      <div class="surface divide-y divide-line overflow-hidden">
        <button type="button" class="flex w-full items-center justify-between px-5 py-3.5 text-left text-sm hover:bg-panel-2" @click="store.openIni('php')">
          php.ini <span class="text-xs text-muted">PHP</span>
        </button>
        <button type="button" class="flex w-full items-center justify-between px-5 py-3.5 text-left text-sm hover:bg-panel-2" @click="store.openIni('mysql')">
          my.ini <span class="text-xs text-muted">MariaDB</span>
        </button>
        <button type="button" class="flex w-full items-center justify-between px-5 py-3.5 text-left text-sm hover:bg-panel-2" @click="store.openIni('apache')">
          httpd.conf <span class="text-xs text-muted">Apache</span>
        </button>
        <button type="button" class="flex w-full items-center justify-between px-5 py-3.5 text-left text-sm hover:bg-panel-2" @click="store.openIni('nginx')">
          nginx.conf <span class="text-xs text-muted">Nginx</span>
        </button>
      </div>
    </div>

    <section class="surface space-y-4 p-4 lg:col-span-2 lg:p-6">
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
