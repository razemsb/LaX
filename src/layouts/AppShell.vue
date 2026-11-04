<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { RouterLink, RouterView, useRoute, useRouter } from "vue-router";
import {
  Blocks,
  FolderKanban,
  Gauge,
  Globe,
  Loader2,
  MessageSquare,
  ScrollText,
  Settings,
} from "lucide-vue-next";
import { useAppStore } from "@/stores/app";
import BrandLogo from "@/components/BrandLogo.vue";
import Banner from "@/components/Banner.vue";
import CloseDialog from "@/components/CloseDialog.vue";
import ContextMenu from "@/components/ContextMenu.vue";
import MiniView from "@/views/MiniView.vue";
import {
  editMenuItems,
  isEditableTarget,
  showContextMenu,
} from "@/composables/contextMenu";
import { dbAdminLabel, dbAdminUrl } from "@/lib/dbAdmin";
import { t } from "@/lib/i18n";
import type { MsgKey } from "@/lib/i18n";

const store = useAppStore();
const route = useRoute();
const router = useRouter();
let timer: number | undefined;
let unlistenProgress: UnlistenFn | undefined;
let unlistenClose: UnlistenFn | undefined;
const showCloseDialog = ref(false);

const links = computed(() => [
  { to: "/", label: t("nav.overview"), icon: Gauge },
  { to: "/projects", label: t("nav.projects"), icon: FolderKanban },
  { to: "/php", label: t("nav.php"), icon: Blocks },
  { to: "/logs", label: t("nav.logs"), icon: ScrollText },
  { to: "/settings", label: t("nav.settings"), icon: Settings },
]);

const titles: Record<string, MsgKey> = {
  overview: "nav.overview",
  projects: "nav.projects",
  php: "nav.php",
  logs: "nav.logs",
  settings: "nav.settings",
};

const stackLine = computed(() => {
  const web = store.snap?.config.webServer === "nginx" ? "Nginx" : "Apache";
  return `${web} · ${store.snap?.config.phpVersion ?? "PHP"}`;
});

const pageTitle = computed(() => {
  if (mode.value === "mini") return "LaX";
  const key = titles[route.name as string];
  return key ? t(key) : "LaX";
});

const portConflict = computed(() => store.snap?.portConflict ?? null);
const mode = computed(() => store.snap?.config.windowMode ?? "medium");
const showPort = computed(() => {
  const c = portConflict.value;
  if (!c) return false;
  return store.dismissedPort !== `${c.port}:${c.pid}`;
});

function wwwPath() {
  if (!store.snap) return "";
  return `${store.snap.root}\\${store.snap.config.documentRoot}`.replaceAll("/", "\\");
}

function tunnelName(path: string) {
  return path.split(/[\\/]/).filter(Boolean).pop() ?? path;
}

function siteUrl() {
  const port = store.snap?.config.webServer === "nginx"
    ? store.snap.config.nginxPort
    : store.snap?.config.apachePort;
  return port && port !== 80 ? `http://localhost:${port}/` : "http://localhost/";
}

function onRootContext(e: MouseEvent) {
  if ((e.target as HTMLElement | null)?.closest("[data-ctx]")) return;
  if (isEditableTarget(e.target)) {
    showContextMenu(e, editMenuItems(e.target));
    return;
  }
  const www = wwwPath();
  showContextMenu(e, [
    {
      label: store.runningCount ? t("ctx.stopAll") : t("ctx.startAll"),
      accent: !store.runningCount,
      disabled: store.busy,
      run: () => (store.runningCount ? store.stopAll() : store.startAll()),
    },
    { type: "sep" },
    { label: "localhost", run: () => store.openUrl(siteUrl()) },
    {
      label: dbAdminLabel(store.snap?.config),
      run: () => store.openUrl(dbAdminUrl(siteUrl(), store.snap?.config)),
    },
    { label: t("ctx.mailpit"), run: () => store.openUrl("http://localhost:8025") },
    { label: t("ctx.wwwFolder"), disabled: !www, run: () => store.openPath(www) },
    { label: t("ctx.wwwTerminal"), disabled: !www, run: () => store.openTerminal(www) },
    { type: "sep" },
    { label: t("nav.projects"), run: () => router.push("/projects") },
    { label: t("nav.php"), run: () => router.push("/php") },
    { label: t("nav.logs"), run: () => router.push("/logs") },
    { label: t("nav.settings"), run: () => router.push("/settings") },
  ]);
}

function onFind(e: KeyboardEvent) {
  if (!(e.ctrlKey || e.metaKey) || e.code !== "KeyF") return;
  if (isEditableTarget(e.target) && (e.target as HTMLElement).hasAttribute("data-page-search")) return;
  const el = document.querySelector<HTMLInputElement>("[data-page-search]");
  if (!el) return;
  e.preventDefault();
  el.focus();
  el.select();
}

function blockNativeMenu(e: Event) {
  e.preventDefault();
}

onMounted(async () => {
  document.addEventListener("contextmenu", blockNativeMenu, true);
  window.addEventListener("keydown", onFind);
  await store.refresh();
  unlistenProgress = await listen<string>("update-progress", (e) => {
    store.updateProgress = e.payload;
  });
  unlistenClose = await listen("close-requested", () => {
    showCloseDialog.value = true;
  });
  timer = window.setInterval(() => {
    if (!store.busy) store.refreshStatus();
  }, 4000);
});
onUnmounted(() => {
  document.removeEventListener("contextmenu", blockNativeMenu, true);
  window.removeEventListener("keydown", onFind);
  if (timer) window.clearInterval(timer);
  if (unlistenProgress) unlistenProgress();
  if (unlistenClose) unlistenClose();
});
</script>

<template>
  <div class="app-root flex h-full min-h-0" @contextmenu="onRootContext">
    <ContextMenu />
    <CloseDialog
      v-if="showCloseDialog"
      @tray="showCloseDialog = false; store.hideWindow()"
      @quit="showCloseDialog = false; store.quitApp()"
      @cancel="showCloseDialog = false"
    />
    <aside v-if="mode !== 'mini'" class="flex w-14 shrink-0 flex-col border-r border-line md:w-16 xl:w-56">
      <div
        class="flex items-center justify-center gap-3 px-2 py-4 xl:justify-start xl:px-5 xl:py-5"
        :title="store.snap ? `LaX v${store.snap.appVersion}` : 'LaX'"
      >
        <BrandLogo size="h-9 w-9 rounded-xl" />
        <div class="hidden min-w-0 xl:block">
          <div class="text-base font-semibold leading-none">LaX</div>
          <div class="mt-1.5 inline-flex rounded-md bg-lift px-1.5 py-0.5 text-[10px] text-muted">
            v{{ store.snap?.appVersion ?? "…" }}
          </div>
        </div>
      </div>

      <nav class="flex flex-1 flex-col gap-0.5 px-1.5 md:px-2 xl:px-3">
        <RouterLink
          v-for="item in links"
          :key="item.to"
          :to="item.to"
          :title="item.label"
          class="nav-item flex items-center justify-center gap-3 px-0 py-2.5 text-sm xl:justify-start xl:px-3"
          :class="route.path === item.to ? 'nav-active' : ''"
        >
          <component :is="item.icon" :size="16" class="shrink-0" />
          <span class="hidden xl:inline">{{ item.label }}</span>
        </RouterLink>
      </nav>

      <div class="px-1.5 pb-1 md:px-2 xl:px-3 xl:pb-2">
        <button
          class="nav-item flex w-full items-center justify-center gap-3 px-0 py-2.5 text-sm xl:justify-start xl:px-3"
          :title="t('app.feedback')"
          @click="store.snap && store.openUrl(store.snap.feedbackUrl)"
        >
          <MessageSquare :size="16" class="shrink-0" />
          <span class="hidden xl:inline">{{ t("app.feedback") }}</span>
        </button>
      </div>

      <div
        class="m-2 overflow-hidden rounded-xl border px-2 py-2 text-center xl:m-3 xl:px-3 xl:py-3 xl:text-left"
        :class="store.runningCount ? 'border-ok/25 bg-ok/5' : 'border-line'"
        :title="stackLine"
      >
        <div class="hidden items-center gap-1.5 text-[10px] tracking-wider text-muted xl:flex">
          <span
            class="h-1.5 w-1.5 rounded-full"
            :class="store.runningCount ? 'bg-ok pulse-dot' : 'bg-dot'"
          />
          {{ t("app.stack") }}
        </div>
        <div class="hidden truncate text-xs xl:mt-1 xl:block">{{ stackLine }}</div>
        <div class="text-[11px] tabular-nums text-muted xl:mt-1">{{ store.runningCount }}/{{ store.snap?.services.length ?? 0 }}</div>
      </div>
    </aside>

    <div class="flex min-h-0 min-w-0 flex-1 flex-col">
      <header class="flex items-center justify-between gap-3 border-b border-line px-3 py-2.5 sm:px-5 xl:px-8 xl:py-4">
        <div class="min-w-0">
          <div class="truncate text-base font-semibold xl:text-lg">{{ pageTitle }}</div>
          <div class="hidden truncate text-[11px] text-muted sm:block" :title="store.snap?.root">{{ store.snap?.root ?? "" }}</div>
        </div>
        <div class="flex shrink-0 items-center gap-2 sm:gap-3">
          <button
            v-if="mode === 'mini'"
            type="button"
            class="btn-ghost inline-flex h-9 w-9 shrink-0 items-center justify-center rounded-lg"
            :title="t('nav.settings')"
            @click="store.setWindowMode('medium'); router.push('/settings')"
          >
            <Settings :size="15" />
          </button>
          <button
            class="inline-flex h-9 shrink-0 items-center gap-2 rounded-lg px-3 text-sm font-medium lg:px-4"
            :class="store.runningCount ? 'btn-danger' : 'btn-primary'"
            :disabled="store.busy"
            @click="store.runningCount ? store.stopAll() : store.startAll()"
          >
            <Loader2 v-if="store.busy" :size="14" class="spin" />
            <span class="sm:hidden">{{ store.runningCount ? t("header.stop") : t("header.start") }}</span>
            <span class="hidden sm:inline">{{ store.runningCount ? t("header.stopAll") : t("header.startAll") }}</span>
          </button>
        </div>
      </header>

      <Banner v-if="store.snap?.update" @close="store.dismissNotice('update')">
        <div>
          {{ t("banner.updateAvailable", { version: store.snap.update.version }) }}
          <span v-if="store.snap.update.size" class="text-muted">
            · {{ Math.round(store.snap.update.size / 1024 / 1024) }} {{ t("banner.mb") }}
          </span>
          <div class="mt-1 truncate text-xs text-muted">
            {{ store.updateProgress || store.snap.update.notes }}
          </div>
        </div>
        <template #actions>
          <button class="btn-ghost rounded-lg px-3 py-1.5 text-sm" @click="store.openUrl(store.snap.update.url)">{{ t("banner.release") }}</button>
          <button
            v-if="store.snap.update.downloadUrl"
            class="btn-primary rounded-lg px-3 py-1.5 text-sm"
            :disabled="store.busy"
            @click="store.applyUpdate()"
          >
            {{ store.updateProgress || store.busy ? t("banner.updating") : t("banner.update") }}
          </button>
        </template>
      </Banner>

      <Banner v-if="showPort && portConflict" tone="accent" @close="store.dismissNotice('port')">
        <div>
          {{ t("banner.portTaken", { port: portConflict.port }) }}
          <span class="font-medium">{{ portConflict.process }}</span>
          <span v-if="portConflict.pid"> (PID {{ portConflict.pid }})</span>
        </div>
        <template #actions>
          <button
            v-if="portConflict.port !== 8080"
            class="btn-primary rounded-lg px-3 py-1.5 text-sm"
            :disabled="store.busy"
            @click="store.switchWebPort(8080)"
          >
            {{ t("banner.switchTo8080") }}
          </button>
        </template>
      </Banner>

      <Banner v-if="store.snap?.tunnels.length" tone="accent">
        <div class="min-w-0 flex-1">
          {{ t("share.bannerTitle") }}
          <div class="mt-1 flex flex-wrap gap-x-4 gap-y-1 text-xs">
            <span v-for="tun in store.snap.tunnels" :key="tun.path" class="inline-flex min-w-0 items-center gap-2">
              <Globe :size="11" class="shrink-0 text-warn" />
              <span class="truncate text-muted">{{ tunnelName(tun.path) }}</span>
              <button
                v-if="tun.url"
                type="button"
                class="truncate text-text underline decoration-line underline-offset-2"
                :title="tun.url"
                @click="store.openUrl(tun.url!)"
              >
                {{ tun.url.replace("https://", "") }}
              </button>
              <span v-else class="text-muted">{{ t("share.opening") }}</span>
              <button type="button" class="shrink-0 text-muted hover:text-text" @click="store.unshareProject(tun.path)">
                {{ t("share.close") }}
              </button>
            </span>
          </div>
        </div>
      </Banner>

      <Banner v-if="store.updateProgress && !store.snap?.update">
        {{ store.updateProgress }}
      </Banner>

      <Banner v-if="store.error" tone="accent" @close="store.clearError()">
        {{ store.error }}
      </Banner>

      <Banner v-else-if="store.snap?.message && !store.snap.update" @close="store.dismissNotice('message')">
        <span class="text-muted">{{ store.snap.message }}</span>
      </Banner>

      <main class="scrollbar flex min-h-0 flex-1 flex-col overflow-auto px-3 py-3 sm:px-5 sm:py-5 xl:px-8 xl:py-6">
        <MiniView v-if="mode === 'mini'" class="min-h-0 flex-1" />
        <RouterView v-else class="min-h-0 flex-1" />
      </main>
    </div>
  </div>
</template>
