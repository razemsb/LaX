<script setup lang="ts">
import { Maximize2, Server } from "lucide-vue-next";
import { useAppStore } from "@/stores/app";
import StatusPill from "@/components/StatusPill.vue";
import { SERVICE_ICONS, SERVICE_NAMES } from "@/lib/serviceMeta";
import { t } from "@/lib/i18n";

const store = useAppStore();

async function toggle(id: string, running: boolean) {
  if (running) await store.stopService(id);
  else await store.startService(id);
}
</script>

<template>
  <div v-if="store.snap" class="mx-auto flex h-full w-full max-w-md flex-col gap-2 p-3">
    <button
      v-for="s in store.snap.services"
      :key="s.id"
      class="surface svc-card flex min-w-0 items-center gap-3 px-3 py-2.5 text-left"
      :class="s.running ? 'svc-on' : ''"
      :disabled="store.busy"
      @click="toggle(s.id, s.running)"
    >
      <span class="icon-well shrink-0" :class="s.running ? 'text-ok bg-ok/10' : ''">
        <component :is="SERVICE_ICONS[s.id] ?? Server" :size="15" />
      </span>
      <span class="min-w-0 flex-1">
        <span class="block truncate text-sm font-medium">{{ SERVICE_NAMES[s.id] ?? s.name }}</span>
        <span class="mt-0.5 block truncate text-[11px] text-muted">
          {{ s.port ? `:${s.port}` : "—" }} · {{ s.version }}
        </span>
      </span>
      <StatusPill :on="s.running" :busy="store.busy && !s.running" />
    </button>

    <button
      type="button"
      class="btn-ghost btn-chip mt-1 inline-flex items-center justify-center gap-2 rounded-lg px-3 py-2 text-xs"
      @click="store.setWindowMode('medium')"
    >
      <Maximize2 :size="12" />
      {{ t("mode.expand") }}
    </button>
  </div>
</template>
