<script setup lang="ts">
import { useAppStore } from "@/stores/app";
import StatusPill from "@/components/StatusPill.vue";

const store = useAppStore();

const names: Record<string, string> = {
  apache: "Apache",
  nginx: "Nginx",
  mariadb: "MariaDB",
  php: "PHP",
};

async function toggle(id: string, running: boolean) {
  if (running) await store.stopService(id);
  else await store.startService(id);
}
</script>

<template>
  <div v-if="store.snap" class="rise space-y-4">
    <p class="text-sm text-muted">
      Каждый процесс включается отдельно. Apache и Nginx делят порт 80 — одновременно работает только один веб-сервер.
    </p>
    <div class="overflow-hidden rounded-2xl border border-line">
      <table class="w-full text-left text-sm">
        <thead class="bg-panel-2 text-[10px] uppercase tracking-[0.16em] text-muted">
          <tr>
            <th class="px-5 py-3 font-medium">Сервис</th>
            <th class="px-5 py-3 font-medium">Версия</th>
            <th class="px-5 py-3 font-medium">Порт</th>
            <th class="px-5 py-3 font-medium">PID</th>
            <th class="px-5 py-3 font-medium">Статус</th>
            <th class="px-5 py-3 font-medium"></th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="s in store.snap.services"
            :key="s.id"
            class="border-t border-line transition hover:bg-white/[0.03]"
          >
            <td class="px-5 py-4 font-medium">{{ names[s.id] ?? s.name }}</td>
            <td class="px-5 py-4 text-xs text-muted">{{ s.version }}</td>
            <td class="px-5 py-4 text-xs">{{ s.port ?? "—" }}</td>
            <td class="px-5 py-4 text-xs text-muted">{{ s.pid ?? "—" }}</td>
            <td class="px-5 py-4"><StatusPill :on="s.running" :label="s.running ? 'работает' : 'остановлен'" /></td>
            <td class="px-5 py-4 text-right">
              <button
                class="rounded-full px-3 py-1.5 text-xs font-semibold transition"
                :class="s.running ? 'btn-ghost' : 'btn-accent'"
                :disabled="store.busy"
                @click="toggle(s.id, s.running)"
              >
                {{ s.running ? "Стоп" : "Старт" }}
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>
