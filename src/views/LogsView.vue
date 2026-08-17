<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useAppStore } from "@/stores/app";

const store = useAppStore();
const which = ref("apache");
const body = ref("");
const q = ref("");
const tabs = [
  { id: "apache", label: "Apache" },
  { id: "nginx", label: "Nginx" },
  { id: "mariadb", label: "MariaDB" },
  { id: "php", label: "PHP" },
  { id: "mailpit", label: "Mailpit" },
];

const lines = computed(() => {
  const all = body.value.split(/\r?\n/);
  const s = q.value.trim().toLowerCase();
  if (!s) return all;
  return all.filter((line) => line.toLowerCase().includes(s));
});

function kind(line: string) {
  const l = line.toLowerCase();
  if (/\[(?:error|crit|alert|emerg)\]/.test(l) || /\b(fatal|panic|exception)\b/.test(l)) return "log-err";
  if (/\[warn(?:ing)?\]/.test(l) || /\bwarning\b/.test(l)) return "log-warn";
  if (/\[notice\]/.test(l)) return "log-notice";
  return "log-info";
}

async function load() {
  body.value = await store.readLogs(which.value);
}

onMounted(load);
watch(which, load);
</script>

<template>
  <div class="flex h-full min-h-0 flex-col gap-3 overflow-hidden lg:gap-4">
    <div class="flex flex-wrap items-center gap-1">
      <button
        v-for="tab in tabs"
        :key="tab.id"
        class="rounded-lg px-3 py-1.5 text-sm transition"
        :class="which === tab.id
          ? 'bg-white/8 text-text shadow-[inset_0_-2px_0_#e02430]'
          : 'text-muted hover:bg-white/4 hover:text-text'"
        @click="which = tab.id"
      >
        {{ tab.label }}
      </button>
      <input
        v-model="q"
        data-page-search
        type="search"
        placeholder="поиск в логе  Ctrl+F"
        class="field ml-auto min-w-0 max-w-56 !py-1.5 text-xs"
      />
      <button class="btn-ghost rounded-lg px-3 py-1.5 text-xs" @click="load">Обновить</button>
    </div>
    <pre class="scrollbar min-h-0 flex-1 overflow-auto rounded-xl border border-line bg-[#0a0a0b] p-5 text-xs leading-6"><span v-for="(line, i) in lines" :key="i" :class="kind(line)" class="block whitespace-pre-wrap">{{ line }}</span></pre>
  </div>
</template>
