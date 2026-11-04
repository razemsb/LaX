<script setup lang="ts">
import { onMounted, onUnmounted } from "vue";
import { ArrowDownToLine, X } from "lucide-vue-next";
import { t } from "@/lib/i18n";

const emit = defineEmits<{
  tray: [];
  quit: [];
  cancel: [];
}>();

function onKey(e: KeyboardEvent) {
  if (e.key === "Escape") emit("cancel");
}

onMounted(() => window.addEventListener("keydown", onKey));
onUnmounted(() => window.removeEventListener("keydown", onKey));
</script>

<template>
  <Teleport to="body">
    <div
      class="fixed inset-0 z-[90] flex items-center justify-center bg-black/45 p-4"
      @click.self="emit('cancel')"
    >
      <div class="surface w-full max-w-sm p-5" role="dialog" aria-modal="true" :aria-label="t('close.title')">
        <div class="flex items-start justify-between gap-3">
          <div>
            <div class="text-sm font-semibold">{{ t("close.title") }}</div>
            <div class="mt-1 text-xs text-muted">
              {{ t("close.hint") }}
            </div>
          </div>
          <button
            type="button"
            class="inline-flex h-7 w-7 shrink-0 items-center justify-center rounded-lg text-muted hover:bg-lift hover:text-text"
            :title="t('close.cancel')"
            @click="emit('cancel')"
          >
            <X :size="14" />
          </button>
        </div>

        <div class="mt-5 flex flex-col gap-2">
          <button
            type="button"
            class="btn-ghost inline-flex items-center justify-center gap-2 rounded-lg px-3 py-2 text-sm"
            @click="emit('tray')"
          >
            <ArrowDownToLine :size="14" />
            {{ t("close.tray") }}
          </button>
          <button
            type="button"
            class="btn-danger inline-flex items-center justify-center gap-2 rounded-lg px-3 py-2 text-sm"
            @click="emit('quit')"
          >
            <X :size="14" />
            {{ t("close.quit") }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
