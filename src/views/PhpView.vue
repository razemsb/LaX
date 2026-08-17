<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useAppStore } from "@/stores/app";
import type { PhpExtension } from "@/types";

const store = useAppStore();
const q = ref("");
const exts = ref<PhpExtension[]>([]);
const busyExt = ref("");

const filtered = computed(() => {
  const s = q.value.trim().toLowerCase();
  if (!s) return exts.value;
  return exts.value.filter((e) => e.name.toLowerCase().includes(s));
});

const onCount = computed(() => exts.value.filter((e) => e.enabled).length);

async function load() {
  exts.value = await store.listPhpExtensions();
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

    <section>
      <div class="mb-3 flex flex-col gap-3 lg:flex-row lg:items-end lg:justify-between">
        <div class="min-w-0">
          <div class="text-[11px] uppercase tracking-wider text-muted">расширения</div>
          <p class="mt-1 text-xs text-muted">{{ onCount }} включено · после изменений: Закрыть все → Запустить все</p>
        </div>
        <div class="flex min-w-0 gap-2">
          <input v-model="q" placeholder="поиск" class="field min-w-0 flex-1 !py-2 lg:w-48 lg:flex-none" />
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
