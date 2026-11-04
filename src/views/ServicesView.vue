<script setup lang="ts">
import { useAppStore } from "@/stores/app";
import StatusPill from "@/components/StatusPill.vue";
import { SERVICE_NAMES } from "@/lib/serviceMeta";
import { t } from "@/lib/i18n";

const store = useAppStore();

async function toggle(id: string, running: boolean) {
  if (running) await store.stopService(id);
  else await store.startService(id);
}
</script>

<template>
  <div v-if="store.snap" class="rise space-y-4">
    <p class="text-sm text-muted">
      {{ t("svc.intro") }}
    </p>
    <div class="overflow-hidden rounded-2xl border border-line">
      <table class="w-full text-left text-sm">
        <thead class="bg-panel-2 text-[11px] tracking-wide text-muted">
          <tr>
            <th class="px-5 py-3 font-medium">{{ t("svc.service") }}</th>
            <th class="px-5 py-3 font-medium">{{ t("svc.version") }}</th>
            <th class="px-5 py-3 font-medium">{{ t("svc.port") }}</th>
            <th class="px-5 py-3 font-medium">{{ t("svc.pid") }}</th>
            <th class="px-5 py-3 font-medium">{{ t("svc.status") }}</th>
            <th class="px-5 py-3 font-medium"></th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="s in store.snap.services"
            :key="s.id"
            class="border-t border-line transition hover:bg-lift"
          >
            <td class="px-5 py-4 font-medium">{{ SERVICE_NAMES[s.id] ?? s.name }}</td>
            <td class="px-5 py-4 text-xs text-muted">{{ s.version }}</td>
            <td class="px-5 py-4 text-xs">{{ s.port ?? "—" }}</td>
            <td class="px-5 py-4 text-xs text-muted">{{ s.pid ?? "—" }}</td>
            <td class="px-5 py-4">
              <StatusPill
                :on="s.running"
                :busy="store.busy && !s.running"
                :label="s.running ? t('pill.running') : t('pill.stopped')"
              />
            </td>
            <td class="px-5 py-4 text-right">
              <button
                class="btn-chip rounded-full px-3 py-1.5 text-xs font-semibold transition"
                :class="s.running ? 'btn-danger' : 'btn-primary'"
                :disabled="store.busy"
                @click="toggle(s.id, s.running)"
              >
                {{ s.running ? t("header.stop") : t("header.start") }}
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>
