<script setup lang="ts">
import { onMounted, onUnmounted, watch } from "vue";
import { Globe, X } from "lucide-vue-next";
import { useAppStore } from "@/stores/app";
import { shareDialog } from "@/composables/share";
import { t } from "@/lib/i18n";

const store = useAppStore();

function close() {
  shareDialog.open = false;
  shareDialog.path = "";
}

async function confirm() {
  const path = shareDialog.path;
  close();
  if (path) await store.shareProject(path);
}

function onKey(e: KeyboardEvent) {
  if (e.key === "Escape" && shareDialog.open) close();
}

watch(
  () => shareDialog.open,
  (open) => {
    if (!open) shareDialog.path = "";
  },
);

onMounted(() => window.addEventListener("keydown", onKey));
onUnmounted(() => window.removeEventListener("keydown", onKey));
</script>

<template>
  <Teleport to="body">
    <div
      v-if="shareDialog.open"
      class="fixed inset-0 z-[90] flex items-center justify-center bg-black/45 p-4"
      @click.self="close"
    >
      <div class="surface w-full max-w-sm p-5" role="dialog" aria-modal="true" :aria-label="t('share.dialogTitle')">
        <div class="flex items-start gap-3">
          <span class="icon-well shrink-0 text-warn"><Globe :size="15" /></span>
          <div class="min-w-0 flex-1">
            <div class="text-sm font-semibold">{{ t("share.dialogTitle") }}</div>
            <p class="mt-1 text-xs leading-relaxed text-muted">{{ t("share.dialogHint") }}</p>
          </div>
          <button
            type="button"
            class="inline-flex h-7 w-7 shrink-0 items-center justify-center rounded-lg text-muted hover:bg-lift hover:text-text"
            :title="t('close.cancel')"
            @click="close"
          >
            <X :size="14" />
          </button>
        </div>

        <div class="mt-5 flex justify-end gap-2">
          <button type="button" class="btn-ghost rounded-lg px-4 py-2 text-sm" @click="close">
            {{ t("close.cancel") }}
          </button>
          <button
            type="button"
            class="btn-danger rounded-lg px-4 py-2 text-sm font-medium"
            :disabled="store.busy"
            @click="confirm"
          >
            {{ t("share.confirm") }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
