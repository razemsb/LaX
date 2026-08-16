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
  <div class="flex h-full bg-ink">
    <aside class="flex w-56 shrink-0 flex-col border-r border-line">
      <div class="flex items-center gap-3 px-5 py-5">
        <BrandLogo size="h-9 w-9 rounded-xl" />
        <div>
          <div class="text-base font-semibold leading-none">LaX</div>
          <div class="mt-1 text-[11px] text-muted">локальный стек</div>
        </div>
      </div>

      <nav class="flex flex-1 flex-col gap-0.5 px-3">
        <RouterLink
          v-for="item in links"
          :key="item.to"
          :to="item.to"
          class="nav-item flex items-center gap-3 rounded-lg px-3 py-2.5 text-sm"
          :class="route.path === item.to ? 'nav-active' : ''"
        >
          <component :is="item.icon" :size="16" />
          {{ item.label }}
        </RouterLink>
      </nav>

      <div class="m-3 rounded-xl border border-line px-3 py-3">
        <div class="text-[10px] uppercase tracking-wider text-muted">активно</div>
        <div class="mt-1 truncate text-xs">{{ stackLine }}</div>
        <div class="mt-1 text-[11px] text-muted">{{ store.runningCount }}/4 сервиса</div>
      </div>
    </aside>

    <div class="flex min-w-0 flex-1 flex-col">
      <header class="flex items-center justify-between border-b border-line px-8 py-4">
        <div>
          <div class="text-lg font-semibold">{{ route.name }}</div>
          <div class="truncate text-[11px] text-muted">{{ store.snap?.root ?? "" }}</div>
        </div>
        <div class="flex items-center gap-2">
          <button
            class="btn-accent inline-flex h-9 items-center gap-2 rounded-lg px-4 text-sm font-medium"
            :disabled="store.busy"
            @click="store.startAll()"
          >
            <Loader2 v-if="store.busy" :size="14" class="spin" />
            Start All
          </button>
          <button
            class="btn-ghost h-9 rounded-lg px-4 text-sm"
            :disabled="store.busy"
            @click="store.stopAll()"
          >
            Stop
          </button>
        </div>
      </header>

      <div v-if="store.error" class="mx-8 mt-4 rounded-lg border border-accent/30 bg-accent/10 px-4 py-3 text-sm text-accent">
        {{ store.error }}
      </div>
      <div v-else-if="store.snap?.message" class="mx-8 mt-4 rounded-lg border border-line px-4 py-3 text-sm text-muted">
        {{ store.snap.message }}
      </div>

      <main class="scrollbar min-h-0 flex-1 overflow-auto px-8 py-6">
        <RouterView />
      </main>
    </div>
  </div>
</template>
