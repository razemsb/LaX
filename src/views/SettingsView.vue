<script setup lang="ts">
import { computed, reactive, watch } from "vue";
import { Maximize2, Minimize2, MoveDiagonal } from "lucide-vue-next";
import { useAppStore } from "@/stores/app";
import type { LaxConfig } from "@/types";
import { THEMES } from "@/lib/themes";
import { LOCALES, t, type Locale } from "@/lib/i18n";

const store = useAppStore();
const windowModes = computed(() => [
  { id: "mini", icon: Minimize2, label: t("mode.mini") },
  { id: "medium", icon: MoveDiagonal, label: t("mode.medium") },
  { id: "large", icon: Maximize2, label: t("mode.large") },
]);
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
  language: "ru",
  windowMode: "medium",
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

function pickLocale(id: Locale) {
  store.setLanguage(id);
}
</script>

<template>
  <div v-if="store.snap" class="grid max-w-4xl grid-cols-1 gap-5 xl:grid-cols-2 xl:gap-6">
    <section class="surface space-y-4 p-4 sm:p-5 xl:col-span-2">
      <div>
        <div class="eyebrow">{{ t("set.language") }}</div>
        <p class="mt-1 text-xs text-muted">{{ t("set.languageHint") }}</p>
      </div>
      <div class="grid grid-cols-3 gap-2">
        <button
          v-for="l in LOCALES"
          :key="l.id"
          type="button"
          class="theme-card"
          :aria-pressed="(store.snap.config.language || 'ru') === l.id"
          @click="pickLocale(l.id)"
        >
          <div class="text-sm">{{ l.native }}</div>
          <div class="mt-1 truncate text-[11px] text-muted">{{ l.label }}</div>
        </button>
      </div>
    </section>

    <section class="surface space-y-4 p-4 sm:p-5 xl:col-span-2">
      <div>
        <div class="eyebrow">{{ t("set.window") }}</div>
        <p class="mt-1 text-xs text-muted">{{ t("set.windowHint") }}</p>
      </div>
      <div class="grid grid-cols-3 gap-2">
        <button
          v-for="m in windowModes"
          :key="m.id"
          type="button"
          class="theme-card flex items-center gap-3"
          :aria-pressed="(store.snap.config.windowMode || 'medium') === m.id"
          @click="store.setWindowMode(m.id)"
        >
          <span class="icon-well shrink-0"><component :is="m.icon" :size="15" /></span>
          <span class="min-w-0 truncate text-sm">{{ m.label }}</span>
        </button>
      </div>
    </section>

    <section class="surface space-y-4 p-4 sm:p-5 xl:col-span-2">
      <div>
        <div class="eyebrow">{{ t("set.theme") }}</div>
        <p class="mt-1 text-xs text-muted">{{ t("set.themeHint") }}</p>
      </div>
      <div class="grid grid-cols-2 gap-2 sm:grid-cols-3">
        <button
          v-for="th in THEMES"
          :key="th.id"
          type="button"
          class="theme-card"
          :aria-pressed="(store.snap.config.theme || 'noir') === th.id"
          @click="store.setTheme(th.id)"
        >
          <div class="mb-3 flex h-9 overflow-hidden rounded-lg">
            <span class="flex-1" :style="{ background: th.swatch[0] }" />
            <span class="flex-1" :style="{ background: th.swatch[1] }" />
            <span class="w-7" :style="{ background: th.swatch[2] }" />
          </div>
          <div class="flex items-center justify-between gap-2">
            <div class="min-w-0">
              <div class="truncate text-sm">{{ th.name }}</div>
              <div class="truncate text-[11px] text-muted">{{ t(`theme.${th.id}`) }}</div>
            </div>
            <span v-if="th.experimental" class="shrink-0 rounded-md bg-lift px-1.5 py-0.5 text-[10px] text-muted">{{ t("set.experimental") }}</span>
          </div>
        </button>
      </div>
    </section>

    <form class="surface space-y-5 p-4 sm:p-5" @submit.prevent="save">
      <label class="block">
        <div class="mb-2 eyebrow">{{ t("set.docRoot") }}</div>
        <input v-model="form.documentRoot" class="field" />
      </label>
      <label class="block">
        <div class="mb-2 eyebrow">{{ t("set.webServer") }}</div>
        <select v-model="form.webServer" class="field">
          <option value="apache">Apache</option>
          <option value="nginx">Nginx</option>
        </select>
      </label>
      <div>
        <label class="flex items-center gap-2 text-sm">
          <input v-model="form.autoVhost" type="checkbox" class="accent-[var(--accent)]" />
          {{ t("set.autoVhost") }}
        </label>
        <p class="mt-1 text-[11px] text-muted">{{ t("set.autoVhostHint") }}</p>
        <div v-if="form.autoVhost" class="mt-3 space-y-2">
          <label class="block">
            <div class="mb-2 eyebrow">{{ t("set.tld") }}</div>
            <input v-model="form.tld" placeholder="test" class="field" />
          </label>
          <p v-if="!store.snap.hostsWritable" class="text-[11px] text-accent">
            {{ t("set.autoVhostHosts") }}
          </p>
        </div>
      </div>
      <div>
        <div class="mb-2 eyebrow">{{ t("set.dbPanel") }}</div>
        <div class="grid grid-cols-2 gap-2">
          <button
            type="button"
            class="theme-card"
            :aria-pressed="(store.snap.config.dbAdmin || 'phpmyadmin') !== 'dbgate'"
            @click="pickDbAdmin('phpmyadmin')"
          >
            <div class="text-sm">phpMyAdmin</div>
            <div class="mt-1 text-[11px] text-muted">{{ t("set.pmaHint") }}</div>
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
              {{ store.snap.dbgateAvailable ? t("set.dbgateHint") : t("set.dbgateMissing") }}
            </div>
          </button>
        </div>
        <p class="mt-2 text-[11px] text-muted">
          {{ t("set.dbNote") }}
        </p>
        <button
          v-if="!store.snap.dbgateAvailable"
          type="button"
          class="btn-primary mt-3 rounded-lg px-4 py-2 text-sm font-medium"
          :disabled="store.busy"
          @click="store.installDbGate()"
        >
          {{ t("set.downloadDbgate") }}
        </button>
      </div>
      <div class="grid grid-cols-1 gap-3 sm:grid-cols-3">
        <label class="text-xs text-muted">Apache<input v-model.number="form.apachePort" type="number" class="field mt-1" /></label>
        <label class="text-xs text-muted">Nginx<input v-model.number="form.nginxPort" type="number" class="field mt-1" /></label>
        <label class="text-xs text-muted">MySQL<input v-model.number="form.mysqlPort" type="number" class="field mt-1" /></label>
      </div>
      <div>
        <div class="mb-2 eyebrow">{{ t("set.startAllBtn") }}</div>
        <p class="mb-3 text-[11px] text-muted">{{ t("set.startAllHint") }}</p>
        <div class="space-y-2">
          <label class="flex items-center gap-2 text-sm">
            <input v-model="form.startWeb" type="checkbox" class="accent-[var(--accent)]" />
            {{ t("set.webPlusPhp", { x: form.webServer === "nginx" ? "Nginx" : "Apache" }) }}
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
          {{ t("set.phpmyadminButDbgate") }}
        </p>
      </div>
      <label class="flex items-center gap-2 text-sm">
        <input v-model="form.autoStart" type="checkbox" class="accent-[var(--accent)]" />
        {{ t("set.autoStart") }}
      </label>
      <button class="btn-primary rounded-lg px-4 py-2 text-sm font-medium" :disabled="store.busy">{{ t("set.save") }}</button>
    </form>

    <div class="space-y-3">
      <div class="eyebrow">{{ t("set.configs") }}</div>
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
          <div class="eyebrow">{{ t("set.about") }}</div>
          <div class="mt-1 text-lg font-medium">LaX v{{ store.snap.appVersion }}</div>
          <p class="mt-1 text-sm text-muted">{{ t("set.tagline") }}</p>
        </div>
        <button class="btn-primary rounded-lg px-4 py-2 text-sm" :disabled="store.busy" @click="store.checkUpdate()">
          {{ t("set.checkUpdates") }}
        </button>
      </div>
      <p v-if="store.snap.update" class="text-sm">
        {{ t("set.updateFound", { version: store.snap.update.version }) }}
      </p>
      <div class="flex flex-wrap gap-2">
        <button class="btn-ghost rounded-lg px-4 py-2 text-sm" @click="store.openUrl(store.snap.repoUrl)">{{ t("set.repo") }}</button>
        <button class="btn-ghost rounded-lg px-4 py-2 text-sm" @click="store.openUrl(store.snap.issuesUrl)">{{ t("set.issues") }}</button>
        <button class="btn-ghost rounded-lg px-4 py-2 text-sm" @click="store.openUrl(store.snap.feedbackUrl)">{{ t("app.feedback") }}</button>
      </div>
    </section>
  </div>
</template>
