<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useAppStore } from "@/stores/app";

const store = useAppStore();
const which = ref("apache");
const body = ref("");
const tabs = [
  { id: "apache", label: "Apache" },
  { id: "nginx", label: "Nginx" },
  { id: "mariadb", label: "MariaDB" },
  { id: "php", label: "PHP" },
  { id: "mailpit", label: "Mailpit" },
];

const lines = computed(() => body.value.split(/\r?\n/));

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
        class="rounded-lg px-3 py-1.5 text-sm"
        :class="which === tab.id ? 'nav-active' : 'text-muted hover:text-text'"
        @click="which = tab.id"
      >
        {{ tab.label }}
      </button>
      <button class="btn-ghost ml-auto rounded-lg px-3 py-1.5 text-xs" @click="load">Обновить</button>
    </div>
    <pre class="scrollbar min-h-0 flex-1 overflow-auto rounded-xl border border-line bg-[#0a0a0b] p-5 text-xs leading-6"><span v-for="(line, i) in lines" :key="i" :class="kind(line)" class="block whitespace-pre-wrap">{{ line }}</span></pre>
  </div>
</template>
