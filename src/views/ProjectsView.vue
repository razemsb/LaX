<script setup lang="ts">
import { computed, ref } from "vue";
import { useAppStore } from "@/stores/app";
import type { ProjectInfo } from "@/types";
import { projectCommands, projectContextItems } from "@/lib/projectCommands";
import { showContextMenu } from "@/composables/contextMenu";

const store = useAppStore();
const name = ref("");
const kind = ref("php");
const q = ref("");

const templates = [
  { id: "php", label: "PHP" },
  { id: "laravel", label: "Laravel" },
  { id: "vite", label: "Vite" },
  { id: "wordpress", label: "WordPress" },
];

const kindLabel: Record<string, string> = {
  vite: "Vite",
  laravel: "Laravel",
  node: "Node",
  wordpress: "WP",
  php: "PHP",
};

const hint = computed(() => {
  switch (kind.value) {
    case "laravel":
      return "в терминале: composer create-project laravel/laravel";
    case "vite":
      return "в терминале: npm create vite — шаблон Vue";
    case "wordpress":
      return "скачает latest.zip с wordpress.org в www";
    default:
      return "пустой index.php в www/имя · ПКМ по карточке — npm / composer";
  }
});

const createLabel = computed(() => {
  if (!store.busy) return "Создать";
  return kind.value === "wordpress" ? "качаю…" : "создаю…";
});

const visible = computed(() => {
  const list = store.snap?.projects ?? [];
  const s = q.value.trim().toLowerCase();
  if (!s) return list;
  return list.filter((p) => {
    const blob = `${p.name} ${p.kind} ${p.url} ${p.scripts.join(" ")}`.toLowerCase();
    return blob.includes(s);
  });
});

function acts() {
  return {
    openUrl: (url: string) => store.openUrl(url),
    openPath: (path: string) => store.openPath(path),
    openTerminal: (path: string) => store.openTerminal(path),
    openVscode: (path: string) => store.openVscode(path),
    run: (path: string, action: string) => store.runProjectAction(path, action),
  };
}

function onCardMenu(e: MouseEvent, p: ProjectInfo) {
  showContextMenu(e, projectContextItems(p, acts()));
}

async function create() {
  if (!name.value.trim()) return;
  await store.createProject(name.value.trim(), kind.value);
  name.value = "";
}
</script>

<template>
  <div v-if="store.snap" class="space-y-5">
    <form class="surface space-y-3 p-4" @submit.prevent="create">
      <div class="flex flex-wrap gap-2">
        <button
          v-for="t in templates"
          :key="t.id"
          type="button"
          class="rounded-lg border px-3 py-1.5 text-sm transition"
          :class="kind === t.id
            ? 'border-accent/40 bg-accent/10 text-text'
            : 'border-line text-muted hover:border-[#3f3f46] hover:text-text'"
          @click="kind = t.id"
        >
          {{ t.label }}
        </button>
      </div>
      <div class="flex min-w-0 gap-2">
        <input v-model="name" placeholder="имя-проекта" class="field min-w-0 flex-1" />
        <button class="btn-accent shrink-0 rounded-lg px-4 text-sm font-medium lg:px-5" :disabled="store.busy || !name.trim()">
          {{ createLabel }}
        </button>
      </div>
      <p class="text-xs text-muted">{{ hint }}</p>
    </form>

    <input
      v-if="store.snap.projects.length"
      v-model="q"
      data-page-search
      type="search"
      placeholder="поиск проекта, kind, скрипт…  Ctrl+F"
      class="field"
    />

    <p v-if="!store.snap.projects.length" class="text-sm text-muted">
      Папка www пустая. Проект откроется как http://localhost/имя/
    </p>
    <p v-else-if="!visible.length" class="text-sm text-muted">Ничего не найдено.</p>
    <p v-else-if="!store.snap.nodeAvailable" class="text-sm text-muted">
      Свой Node не найден в bin/node — npm-кнопки возьмут системный, если он есть.
      Положи туда node.exe или запусти <span class="text-text">npm run fetch-tools</span>.
    </p>

    <div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
      <article
        v-for="p in visible"
        :key="p.name"
        data-ctx
        class="surface min-w-0 p-4 lg:p-5"
        @contextmenu="onCardMenu($event, p)"
      >
        <div class="mb-4 flex flex-wrap items-start justify-between gap-3">
          <div class="min-w-0">
            <h3 class="truncate text-base font-medium">{{ p.name }}</h3>
            <div class="mt-1 truncate text-xs text-muted">{{ p.url.replace("http://", "") }}</div>
          </div>
          <div class="flex shrink-0 gap-1">
            <span class="rounded-md bg-white/5 px-2 py-1 text-[10px] uppercase tracking-wider text-muted">{{ kindLabel[p.kind] ?? p.kind }}</span>
            <span v-if="p.hasPublic" class="rounded-md bg-white/5 px-2 py-1 text-[10px] text-muted">public/</span>
          </div>
        </div>

        <div class="flex flex-wrap gap-2">
          <button class="btn-ghost rounded-lg px-3 py-1.5 text-xs" @click="store.openUrl(p.url)">сайт</button>
          <button v-if="p.kind === 'vite'" class="btn-ghost rounded-lg px-3 py-1.5 text-xs" @click="store.openUrl('http://localhost:5173/')">:5173</button>
          <button class="btn-ghost rounded-lg px-3 py-1.5 text-xs" @click="store.openPath(p.path)">папка</button>
          <button class="btn-ghost rounded-lg px-3 py-1.5 text-xs" @click="store.openTerminal(p.path)">терминал</button>
          <button class="btn-ghost rounded-lg px-3 py-1.5 text-xs" @click="store.openVscode(p.path)">VS Code</button>
        </div>

        <p v-if="projectCommands(p).length" class="mt-3 text-[11px] text-muted">
          ПКМ — {{ projectCommands(p).length }} команд (npm / composer)
        </p>
      </article>
    </div>
  </div>
</template>
