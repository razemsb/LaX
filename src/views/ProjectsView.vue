<script setup lang="ts">
import { computed, ref } from "vue";
import { FolderKanban, Globe } from "lucide-vue-next";
import { useAppStore } from "@/stores/app";
import type { ProjectInfo } from "@/types";
import { projectCommands, projectContextItems } from "@/lib/projectCommands";
import { showContextMenu } from "@/composables/contextMenu";
import { t, type MsgKey } from "@/lib/i18n";

const store = useAppStore();
const name = ref("");
const nameInput = ref<HTMLInputElement | null>(null);
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

const hints: Record<string, MsgKey> = {
  laravel: "proj.hintLaravel",
  vite: "proj.hintVite",
  wordpress: "proj.hintWp",
  php: "proj.hintPhp",
};

const hint = computed(() => t(hints[kind.value] ?? "proj.hintPhp"));

const createLabel = computed(() => {
  if (!store.busy) return t("proj.create");
  return kind.value === "wordpress" ? t("proj.downloading") : t("proj.creating");
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

function tunnelFor(p: ProjectInfo) {
  return store.snap?.tunnels.find((x) => x.path === p.path) ?? null;
}

function acts() {
  return {
    openUrl: (url: string) => store.openUrl(url),
    openPath: (path: string) => store.openPath(path),
    openTerminal: (path: string) => store.openTerminal(path),
    openVscode: (path: string) => store.openVscode(path),
    run: (path: string, action: string) => store.runProjectAction(path, action),
    share: (path: string) => store.shareProject(path),
    unshare: (path: string) => store.unshareProject(path),
  };
}

function onCardMenu(e: MouseEvent, p: ProjectInfo) {
  showContextMenu(e, projectContextItems(p, acts(), tunnelFor(p)));
}

function onPhpChange(ev: Event, p: ProjectInfo) {
  const v = (ev.target as HTMLSelectElement).value;
  store.setProjectPhp(p.path, v || null);
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
          v-for="tmpl in templates"
          :key="tmpl.id"
          type="button"
          class="rounded-lg border px-3 py-1.5 text-sm transition"
          :class="kind === tmpl.id
            ? 'border-accent/40 bg-accent/10 text-text'
            : 'border-line text-muted hover:border-text/40 hover:text-text'"
          @click="kind = tmpl.id"
        >
          {{ tmpl.label }}
        </button>
      </div>
      <div class="flex min-w-0 gap-2">
        <input ref="nameInput" v-model="name" :placeholder="t('proj.name')" class="field min-w-0 flex-1" />
        <button class="btn-primary shrink-0 rounded-lg px-4 text-sm font-medium lg:px-5" :disabled="store.busy || !name.trim()">
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
      :placeholder="t('proj.search')"
      class="field"
    />

    <div v-if="!store.snap.projects.length" class="flex flex-col items-start gap-3">
      <span class="icon-well"><FolderKanban :size="15" /></span>
      <p class="text-sm text-muted">{{ t("proj.wwwEmpty") }}</p>
      <button
        type="button"
        class="btn-primary rounded-lg px-4 py-2 text-sm"
        @click="nameInput?.focus()"
      >
        {{ t("proj.emptyAction") }}
      </button>
    </div>
    <p v-else-if="!visible.length" class="text-sm text-muted">{{ t("proj.notFound") }}</p>
    <p v-else-if="!store.snap.nodeAvailable" class="text-sm text-muted">
      {{ t("proj.nodeMissing") }} <span class="text-text">npm run fetch-tools</span>.
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
            <span v-if="tunnelFor(p)" class="inline-flex items-center gap-1 rounded-md bg-warn/15 px-2 py-1 text-[10px] text-warn">
              <Globe :size="10" />
              {{ tunnelFor(p)?.url ? t("share.public") : t("share.opening") }}
            </span>
            <span class="rounded-md bg-lift px-2 py-1 text-[10px] tracking-wider text-muted">{{ kindLabel[p.kind] ?? p.kind }}</span>
            <span v-if="p.hasPublic" class="rounded-md bg-lift px-2 py-1 text-[10px] text-muted">public/</span>
          </div>
        </div>

        <div class="flex flex-wrap items-center gap-2">
          <select
            class="field !w-auto min-w-0 !py-1.5 text-xs"
            :value="p.phpVersion ?? ''"
            :disabled="store.busy"
            @change="onPhpChange($event, p)"
          >
            <option value="">{{ t("proj.phpDefault") }}</option>
            <option v-for="v in store.snap?.phpVersions ?? []" :key="v" :value="v">{{ v }}</option>
          </select>
          <button class="btn-ghost btn-chip rounded-lg px-3 py-1.5 text-xs" @click="store.openUrl(p.url)">{{ t("proj.site") }}</button>
          <button v-if="p.kind === 'vite'" class="btn-ghost btn-chip rounded-lg px-3 py-1.5 text-xs" @click="store.openUrl('http://localhost:5173/')">:5173</button>
          <button class="btn-ghost btn-chip rounded-lg px-3 py-1.5 text-xs" @click="store.openPath(p.path)">{{ t("proj.folder") }}</button>
          <button class="btn-ghost btn-chip rounded-lg px-3 py-1.5 text-xs" @click="store.openTerminal(p.path)">{{ t("proj.term") }}</button>
          <button class="btn-ghost btn-chip rounded-lg px-3 py-1.5 text-xs" @click="store.openVscode(p.path)">VS Code</button>
        </div>

        <p v-if="projectCommands(p).length" class="mt-3 text-[11px] text-muted">
          {{ t("proj.ctxCommands", { n: projectCommands(p).length }) }}
        </p>
      </article>
    </div>
  </div>
</template>
