<script setup lang="ts">
import { X } from "lucide-vue-next";

withDefaults(
  defineProps<{
    tone?: "neutral" | "accent";
  }>(),
  { tone: "neutral" },
);

const emit = defineEmits<{ close: [] }>();
</script>

<template>
  <div
    class="mx-3 mt-3 flex flex-wrap items-start gap-3 rounded-xl border px-3 py-3 text-sm sm:mx-4 sm:px-4 lg:mx-8 lg:mt-4"
    :class="tone === 'accent'
      ? 'border-accent/30 bg-accent/10 text-accent'
      : 'border-line bg-panel/80 text-text'"
  >
    <div class="min-w-0 flex-1 pt-0.5">
      <slot />
    </div>
    <div class="flex min-w-0 flex-wrap items-center justify-end gap-2">
      <slot name="actions" />
      <button
        type="button"
        class="inline-flex h-7 w-7 items-center justify-center rounded-lg text-muted hover:bg-panel-2 hover:text-text"
        title="закрыть"
        @click="emit('close')"
      >
        <X :size="14" />
      </button>
    </div>
  </div>
</template>
