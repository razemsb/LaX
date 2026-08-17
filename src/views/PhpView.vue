<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useAppStore } from "@/stores/app";
import type { PhpExtension, PhpQuickSettings } from "@/types";

const store = useAppStore();
const q = ref("");
const exts = ref<PhpExtension[]>([]);
const busyExt = ref("");
const quick = ref<PhpQuickSettings | null>(null);
const busyQuick = ref("");

const memoryOpts = ["128M", "256M", "512M", "1G"];
const uploadOpts = ["8M", "32M", "64M", "128M"];

const filtered = computed(() => {
  const s = q.value.trim().toLowerCase();
  if (!s) return exts.value;
  return exts.value.filter((e) => e.name.toLowerCase().includes(s));
});

const onCount = computed(() => exts.value.filter((e) => e.enabled).length);

const memoryChoices = computed(() => {
  const cur = quick.value?.memoryLimit;
  const list = [...memoryOpts];
  if (cur && !list.some((x) => x.toLowerCase() === cur.toLowerCase())) list.unshift(cur);
  return list;
});

const uploadChoices = computed(() => {
  const cur = quick.value?.uploadMaxFilesize;
  const list = [...uploadOpts];
  if (cur && !list.some((x) => x.toLowerCase() === cur.toLowerCase())) list.unshift(cur);
  return list;
});

async function load() {
  const [list, settings] = await Promise.all([
    store.listPhpExtensions(),
    store.phpQuickSettings(),
  ]);
  exts.value = list;
  quick.value = settings;
}

async function toggle(ext: PhpExtension) {
  busyExt.value = ext.name;
  try {
    await store.setPhpExtension(ext.name, !ext.enabled);
    await load();
  } finally {
    busyExt.value = "";
  }
}

async function patchQuick(key: string, value: boolean | string) {
  busyQuick.value = key;
  try {
    quick.value = await store.setPhpQuickSettings({ [key]: value });
    if (key === "xdebug") await load();
  } finally {
    busyQuick.value = "";
  }
}

onMounted(load);
watch(() => store.snap?.config.phpVersion, load);
</script>

<template>
  <div v-if="store.snap" class="space-y-6">
    <section>
      <div class="mb-3 text-[11px] uppercase tracking-wider text-muted">версия</div>
      <div class="flex flex-wrap gap-2">
        <button
          v-for="ver in store.snap.phpVersions"
          :key="ver"
          class="rounded-lg border px-4 py-2 text-sm transition"
          :class="ver === store.snap.config.phpVersion
            ? 'border-accent/40 bg-accent/10 text-text'
            : 'border-line text-muted hover:border-[#3f3f46] hover:text-text'"
          :disabled="store.busy"
          @click="store.switchPhp(ver)"
        >
          {{ ver }}
        </button>
      </div>
    </section>

    <section v-if="quick" class="surface space-y-4 p-4 sm:p-5">
      <div>
        <div class="eyebrow">быстрый php.ini</div>
        <p class="mt-1 text-xs text-muted">пишется сразу; если стек онлайн — PHP перезапустится</p>
      </div>

      <div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
        <button
          class="flex items-center justify-between rounded-xl border border-line px-4 py-3 text-left"
          :disabled="!!busyQuick"
          @click="patchQuick('displayErrors', !quick.displayErrors)"
        >
          <div>
            <div class="text-sm">display_errors</div>
            <div class="mt-0.5 text-[11px] text-muted">ошибки в браузере</div>
          </div>
          <span
            class="relative h-5 w-9 rounded-full border border-line"
            :class="quick.displayErrors ? 'bg-[#1e3d2e]' : 'bg-ink'"
          >
            <span
              class="absolute top-0.5 h-4 w-4 rounded-full"
              :class="quick.displayErrors ? 'right-0.5 bg-ok' : 'left-0.5 bg-[#3a3a3e]'"
            />
          </span>
        </button>

        <button
          class="flex items-center justify-between rounded-xl border border-line px-4 py-3 text-left disabled:opacity-50"
          :disabled="!!busyQuick || (!quick.xdebugAvailable && !quick.xdebug)"
          :title="quick.xdebugAvailable ? '' : 'нет php_xdebug в ext/'"
          @click="patchQuick('xdebug', !quick.xdebug)"
        >
          <div>
            <div class="text-sm">Xdebug</div>
            <div class="mt-0.5 text-[11px] text-muted">
              {{ quick.xdebugAvailable ? `IDE слушай :${quick.xdebugPort} · CGI :9003` : "dll не найдена в ext/" }}
            </div>
          </div>
          <span
            class="relative h-5 w-9 rounded-full border border-line"
            :class="quick.xdebug ? 'bg-[#1e3d2e]' : 'bg-ink'"
          >
            <span
              class="absolute top-0.5 h-4 w-4 rounded-full"
              :class="quick.xdebug ? 'right-0.5 bg-ok' : 'left-0.5 bg-[#3a3a3e]'"
            />
          </span>
        </button>
      </div>

      <div>
        <div class="mb-2 text-xs text-muted">memory_limit</div>
        <div class="flex flex-wrap gap-2">
          <button
            v-for="opt in memoryChoices"
            :key="opt"
            class="rounded-lg border px-3 py-1.5 text-sm"
            :class="opt.toLowerCase() === quick.memoryLimit.toLowerCase()
              ? 'border-accent/40 bg-accent/10 text-text'
              : 'border-line text-muted hover:text-text'"
            :disabled="!!busyQuick"
            @click="patchQuick('memoryLimit', opt)"
          >
            {{ opt }}
          </button>
        </div>
      </div>

      <div>
        <div class="mb-2 text-xs text-muted">upload_max_filesize <span v-if="quick.postMaxSize" class="text-muted/80">· post {{ quick.postMaxSize }}</span></div>
        <div class="flex flex-wrap gap-2">
          <button
            v-for="opt in uploadChoices"
            :key="opt"
            class="rounded-lg border px-3 py-1.5 text-sm"
            :class="opt.toLowerCase() === quick.uploadMaxFilesize.toLowerCase()
              ? 'border-accent/40 bg-accent/10 text-text'
              : 'border-line text-muted hover:text-text'"
            :disabled="!!busyQuick"
            @click="patchQuick('uploadMaxFilesize', opt)"
          >
            {{ opt }}
          </button>
        </div>
      </div>
    </section>

    <section>
      <div class="mb-3 flex flex-col gap-3 lg:flex-row lg:items-end lg:justify-between">
        <div class="min-w-0">
          <div class="text-[11px] uppercase tracking-wider text-muted">расширения</div>
          <p class="mt-1 text-xs text-muted">{{ onCount }} включено · после списка расширений лучше перезапустить стек</p>
        </div>
        <div class="flex min-w-0 gap-2">
          <input v-model="q" data-page-search type="search" placeholder="поиск расширения  Ctrl+F" class="field min-w-0 flex-1 !py-2 lg:w-48 lg:flex-none" />
          <button class="btn-ghost shrink-0 rounded-lg px-3 py-2 text-xs" @click="store.openIni('php')">php.ini</button>
        </div>
      </div>

      <div class="surface grid max-h-[min(420px,calc(100dvh-16rem))] grid-cols-1 gap-x-2 overflow-auto p-2 scrollbar sm:grid-cols-2 lg:grid-cols-3">
        <button
          v-for="ext in filtered"
          :key="ext.name + ext.kind"
          class="flex min-w-0 items-center justify-between rounded-lg px-3 py-2.5 text-left hover:bg-panel-2"
          :disabled="busyExt === ext.name"
          @click="toggle(ext)"
        >
          <span class="min-w-0 truncate text-sm">
            {{ ext.name }}
            <span v-if="ext.kind === 'zend'" class="text-[10px] text-muted">zend</span>
          </span>
          <span
            class="relative h-5 w-9 rounded-full border border-line"
            :class="ext.enabled ? 'bg-[#1e3d2e]' : 'bg-ink'"
          >
            <span
              class="absolute top-0.5 h-4 w-4 rounded-full"
              :class="ext.enabled ? 'right-0.5 bg-ok' : 'left-0.5 bg-[#3a3a3e]'"
            />
          </span>
        </button>
        <div v-if="!filtered.length" class="col-span-full px-3 py-6 text-sm text-muted">ничего не найдено</div>
      </div>
    </section>
  </div>
</template>
