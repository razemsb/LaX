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
  phpVersion: "php-dlya-debilov",
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
  <div v-if="store.snap" class="grid max-w-4xl grid-cols-2 gap-6">
    <form class="surface space-y-5 p-6" @submit.prevent="save">
      <label class="block">
        <div class="mb-2 text-[11px] uppercase tracking-wider text-muted">корень сайтов</div>
        <input v-model="form.documentRoot" class="field" />
      </label>
      <label class="block">
        <div class="mb-2 text-[11px] uppercase tracking-wider text-muted">веб-сервер</div>
        <select v-model="form.webServer" class="field">
          <option value="apache">Apache</option>
          <option value="nginx">Nginx</option>
        </select>
      </label>
      <div class="grid grid-cols-3 gap-3">
        <label class="text-xs text-muted">Apache<input v-model.number="form.apachePort" type="number" class="field mt-1" /></label>
        <label class="text-xs text-muted">Nginx<input v-model.number="form.nginxPort" type="number" class="field mt-1" /></label>
        <label class="text-xs text-muted">MySQL<input v-model.number="form.mysqlPort" type="number" class="field mt-1" /></label>
      </div>
      <label class="flex items-center gap-2 text-sm">
        <input v-model="form.mysqlEnabled" type="checkbox" class="accent-[#e02430]" />
        Поднимать MariaDB вместе со стеком
      </label>
      <button class="btn-accent rounded-lg px-4 py-2 text-sm font-medium" :disabled="store.busy">Сохранить</button>
    </form>

    <div class="space-y-3">
      <div class="text-[11px] uppercase tracking-wider text-muted">конфиги</div>
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
  </div>
</template>
