<script setup lang="ts">
import { ref } from "vue";
import { useAppStore } from "@/stores/app";

const store = useAppStore();
const name = ref("");

async function create() {
  if (!name.value.trim()) return;
  await store.createProject(name.value.trim());
  name.value = "";
}
</script>

<template>
  <div v-if="store.snap" class="space-y-5">
    <form class="flex gap-2" @submit.prevent="create">
      <input v-model="name" placeholder="имя-проекта" class="field flex-1" />
      <button class="btn-accent rounded-lg px-5 text-sm font-medium" :disabled="store.busy">Создать</button>
    </form>

    <p v-if="!store.snap.projects.length" class="text-sm text-muted">
      Папка www пустая. Проект откроется как http://localhost/имя/
    </p>

    <div class="grid grid-cols-2 gap-3">
      <article v-for="p in store.snap.projects" :key="p.name" class="surface p-5">
        <div class="mb-4 flex items-start justify-between gap-3">
          <div>
            <h3 class="text-base font-medium">{{ p.name }}</h3>
            <div class="mt-1 text-xs text-muted">{{ p.url }}</div>
          </div>
          <span v-if="p.hasPublic" class="rounded-md bg-panel-2 px-2 py-1 text-[10px] text-muted">public/</span>
        </div>
        <div class="flex gap-2">
          <button class="btn-ghost rounded-lg px-3 py-1.5 text-xs" @click="store.openUrl(p.url)">сайт</button>
          <button class="btn-ghost rounded-lg px-3 py-1.5 text-xs" @click="store.openPath(p.path)">папка</button>
          <button class="btn-ghost rounded-lg px-3 py-1.5 text-xs" @click="store.openTerminal(p.path)">терминал</button>
          <button class="btn-ghost rounded-lg px-3 py-1.5 text-xs" @click="store.openVscode(p.path)">VS Code</button>
        </div>
      </article>
    </div>
  </div>
</template>
