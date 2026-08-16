<script setup lang="ts">
import { computed, onMounted, onUnmounted } from "vue";
import { RouterLink, RouterView, useRoute } from "vue-router";
import {
  Blocks,
  FolderKanban,
  Gauge,
  Loader2,
  ScrollText,
  Settings,
} from "lucide-vue-next";
import { useAppStore } from "@/stores/app";
import BrandLogo from "@/components/BrandLogo.vue";

const store = useAppStore();
const route = useRoute();
let timer: number | undefined;

const links = [
  { to: "/", label: "Обзор", icon: Gauge },
  { to: "/projects", label: "Проекты", icon: FolderKanban },
  { to: "/php", label: "PHP", icon: Blocks },
  { to: "/logs", label: "Логи", icon: ScrollText },
  { to: "/settings", label: "Настройки", icon: Settings },
];

const stackLine = computed(() => {
  const web = store.snap?.config.webServer === "nginx" ? "Nginx" : "Apache";
  return `${web} · ${store.snap?.config.phpVersion ?? "PHP"}`;
});

onMounted(async () => {
  await store.refresh();
  timer = window.setInterval(() => store.refreshStatus(), 4000);
});
onUnmounted(() => {
  if (timer) window.clearInterval(timer);
});
</script>

<template>
  <div class="flex h-full min-h-0 bg-ink">
    <aside class="flex w-16 shrink-0 flex-col border-r border-line lg:w-56">
      <div
        class="flex items-center justify-center gap-3 px-2 py-4 lg:justify-start lg:px-5 lg:py-5"
        :title="store.snap ? `LaX v${store.snap.appVersion}` : 'LaX'"
      >
        <BrandLogo size="h-9 w-9 rounded-xl" />
        <div class="hidden min-w-0 lg:block">
          <div class="text-base font-semibold leading-none">LaX</div>
          <div class="mt-1 text-[11px] text-muted">v{{ store.snap?.appVersion ?? "…" }}</div>
        </div>
      </div>

      <nav class="flex flex-1 flex-col gap-0.5 px-2 lg:px-3">
        <RouterLink
          v-for="item in links"
          :key="item.to"
          :to="item.to"
          :title="item.label"
          class="nav-item flex items-center justify-center gap-3 rounded-lg px-0 py-2.5 text-sm lg:justify-start lg:px-3"
          :class="route.path === item.to ? 'nav-active' : ''"
        >
          <component :is="item.icon" :size="16" class="shrink-0" />
          <span class="hidden lg:inline">{{ item.label }}</span>
        </RouterLink>
      </nav>

      <div class="hidden px-3 pb-1 lg:block">
        <button
          class="w-full truncate text-left text-[11px] text-muted hover:text-text"
          @click="store.snap && store.openUrl(store.snap.feedbackUrl)"
        >
          фидбек / баг
        </button>
      </div>

      <div class="m-2 rounded-xl border border-line px-2 py-2 text-center lg:m-3 lg:px-3 lg:py-3 lg:text-left" :title="stackLine">
        <div class="hidden text-[10px] uppercase tracking-wider text-muted lg:block">активно</div>
        <div class="hidden truncate text-xs lg:mt-1 lg:block">{{ stackLine }}</div>
        <div class="text-[11px] text-muted lg:mt-1">{{ store.runningCount }}/{{ store.snap?.services.length ?? 0 }}</div>
      </div>
    </aside>

    <div class="flex min-h-0 min-w-0 flex-1 flex-col">
      <header class="flex items-center justify-between gap-3 border-b border-line px-4 py-3 lg:px-8 lg:py-4">
        <div class="min-w-0">
          <div class="truncate text-base font-semibold lg:text-lg">{{ route.name }}</div>
          <div class="truncate text-[11px] text-muted">{{ store.snap?.root ?? "" }}</div>
        </div>
        <button
          class="inline-flex h-9 shrink-0 items-center gap-2 rounded-lg px-3 text-sm font-medium lg:px-4"
          :class="store.runningCount ? 'btn-ghost' : 'btn-accent'"
          :disabled="store.busy"
          @click="store.runningCount ? store.stopAll() : store.startAll()"
        >
          <Loader2 v-if="store.busy" :size="14" class="spin" />
          {{ store.runningCount ? "Закрыть все" : "Запустить все" }}
        </button>
      </header>

      <div
        v-if="store.snap?.update"
        class="mx-4 mt-3 flex flex-wrap items-center justify-between gap-3 rounded-lg border border-line px-4 py-3 text-sm lg:mx-8 lg:mt-4"
      >
        <div class="min-w-0">
          Доступна <span class="font-medium">v{{ store.snap.update.version }}</span>
          <span v-if="store.snap.update.size" class="text-muted">
            · {{ Math.round(store.snap.update.size / 1024 / 1024) }} МБ
          </span>
          <div v-if="store.snap.update.notes" class="mt-1 truncate text-xs text-muted">{{ store.snap.update.notes }}</div>
        </div>
        <div class="flex shrink-0 flex-wrap gap-2">
          <button class="btn-ghost rounded-lg px-3 py-1.5 text-sm" @click="store.openUrl(store.snap.update.url)">релиз</button>
          <button
            v-if="store.snap.update.downloadUrl"
            class="btn-accent rounded-lg px-3 py-1.5 text-sm"
            :disabled="store.busy"
            @click="store.applyUpdate()"
          >
            Обновить
          </button>
          <button class="btn-ghost rounded-lg px-3 py-1.5 text-sm" @click="store.dismissUpdate()">скрыть</button>
        </div>
      </div>
      <div
        v-if="store.snap?.portConflict"
        class="mx-4 mt-3 flex flex-wrap items-center justify-between gap-3 rounded-lg border border-accent/30 bg-accent/10 px-4 py-3 text-sm lg:mx-8 lg:mt-4"
      >
        <div>
          Порт <span class="font-medium">:{{ store.snap.portConflict.port }}</span> занят
          <span class="font-medium">{{ store.snap.portConflict.process }}</span>
          <span v-if="store.snap.portConflict.pid"> (PID {{ store.snap.portConflict.pid }})</span>
        </div>
        <button
          v-if="store.snap.portConflict.port !== 8080"
          class="btn-accent shrink-0 rounded-lg px-3 py-1.5 text-sm"
          :disabled="store.busy"
          @click="store.switchWebPort(8080)"
        >
          Сменить на 8080
        </button>
      </div>
      <div v-if="store.error" class="mx-4 mt-3 rounded-lg border border-accent/30 bg-accent/10 px-4 py-3 text-sm text-accent lg:mx-8 lg:mt-4">
        {{ store.error }}
      </div>
      <div v-else-if="store.snap?.message && !store.snap.update" class="mx-4 mt-3 rounded-lg border border-line px-4 py-3 text-sm text-muted lg:mx-8 lg:mt-4">
        {{ store.snap.message }}
      </div>

      <main class="scrollbar flex min-h-0 flex-1 flex-col overflow-auto px-4 py-4 lg:px-8 lg:py-6">
        <RouterView class="min-h-0 flex-1" />
      </main>
    </div>
  </div>
</template>
