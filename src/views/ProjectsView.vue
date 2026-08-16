<script setup lang="ts">
import { ref } from "vue";
import { useAppStore } from "@/stores/app";
import type { ProjectInfo } from "@/types";

const store = useAppStore();
const name = ref("");

const kindLabel: Record<string, string> = {
  vite: "Vite",
  laravel: "Laravel",
  node: "Node",
  wordpress: "WP",
  php: "PHP",
};

const primary = ["dev", "start", "serve", "preview", "build"];

type Cmd = { label: string; action: string; hint: string; accent?: boolean };

function commandsOf(p: ProjectInfo): Cmd[] {
  const out: Cmd[] = [];
  if (p.hasPackage) {
    out.push({
      label: "install",
      action: "npm-install",
      hint: "npm install",
      accent: !p.hasNodeModules,
    });
  }
  if (p.hasComposer) {
    out.push({
      label: "composer",
      action: "composer-install",
      hint: "composer install",
      accent: !p.hasVendor,
    });
  }
  const have = new Set(p.scripts);
  for (const s of primary) {
    if (!have.has(s)) continue;
    out.push({
      label: s,
      action: "npm-run:" + s,
      hint: "npm run " + s,
      accent: s === "dev" || s === "start",
    });
  }
  for (const s of p.scripts) {
    if (primary.includes(s) || s.startsWith("pre") || s.startsWith("post")) continue;
    out.push({ label: s, action: "npm-run:" + s, hint: "npm run " + s });
  }
  return out;
}

async function create() {
  if (!name.value.trim()) return;
  await store.createProject(name.value.trim());
  name.value = "";
}
</script>

<template>
  <div v-if="store.snap" class="space-y-5">
    <form class="flex min-w-0 gap-2" @submit.prevent="create">
      <input v-model="name" placeholder="имя-проекта" class="field min-w-0 flex-1" />
      <button class="btn-accent shrink-0 rounded-lg px-4 text-sm font-medium lg:px-5" :disabled="store.busy">Создать</button>
    </form>

    <p v-if="!store.snap.projects.length" class="text-sm text-muted">
      Папка www пустая. Проект откроется как http://localhost/имя/
    </p>

    <div class="grid grid-cols-1 gap-3 lg:grid-cols-2">
      <article v-for="p in store.snap.projects" :key="p.name" class="surface min-w-0 p-4 lg:p-5">
        <div class="mb-4 flex flex-wrap items-start justify-between gap-3">
          <div>
            <h3 class="text-base font-medium">{{ p.name }}</h3>
            <div class="mt-1 text-xs text-muted">{{ p.url }}</div>
          </div>
          <div class="flex shrink-0 gap-1">
            <span class="rounded-md bg-panel-2 px-2 py-1 text-[10px] text-muted">{{ kindLabel[p.kind] ?? p.kind }}</span>
            <span v-if="p.hasPublic" class="rounded-md bg-panel-2 px-2 py-1 text-[10px] text-muted">public/</span>
          </div>
        </div>

        <div class="flex flex-wrap gap-2">
          <button class="btn-ghost rounded-lg px-3 py-1.5 text-xs" @click="store.openUrl(p.url)">сайт</button>
          <button v-if="p.kind === 'vite'" class="btn-ghost rounded-lg px-3 py-1.5 text-xs" @click="store.openUrl('http://localhost:5173/')">:5173</button>
          <button class="btn-ghost rounded-lg px-3 py-1.5 text-xs" @click="store.openPath(p.path)">папка</button>
          <button class="btn-ghost rounded-lg px-3 py-1.5 text-xs" @click="store.openTerminal(p.path)">терминал</button>
          <button class="btn-ghost rounded-lg px-3 py-1.5 text-xs" @click="store.openVscode(p.path)">VS Code</button>
        </div>

        <div v-if="p.hasPackage || p.hasComposer" class="mt-3 border-t border-line pt-3">
          <div class="mb-2 text-[10px] uppercase tracking-wider text-muted">команды</div>
          <div class="grid max-h-44 grid-cols-2 gap-1.5 overflow-auto scrollbar">
            <button
              v-for="c in commandsOf(p)"
              :key="c.action"
              class="h-8 truncate rounded-lg px-2.5 text-left text-xs"
              :class="c.accent ? 'btn-accent' : 'btn-ghost'"
              :title="c.hint"
              @click="store.runProjectAction(p.path, c.action)"
            >
              {{ c.label }}
            </button>
          </div>
        </div>
      </article>
    </div>
  </div>
</template>
