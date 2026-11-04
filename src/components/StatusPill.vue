<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{ on?: boolean; busy?: boolean; label?: string }>();

const state = computed(() => (props.on ? "on" : props.busy ? "busy" : "off"));
const dot = computed(() =>
  state.value === "on"
    ? "bg-ok pulse-dot"
    : state.value === "busy"
      ? "bg-warn pulse-dot"
      : "bg-dot",
);
const tone = computed(() =>
  state.value === "on"
    ? "text-ok"
    : state.value === "busy"
      ? "text-warn"
      : "text-muted",
);
</script>

<template>
  <span class="inline-flex shrink-0 items-center gap-1.5 whitespace-nowrap" :title="label">
    <span class="h-2 w-2 rounded-full" :class="dot" />
    <span v-if="label" class="text-[11px] font-medium" :class="tone">{{ label }}</span>
  </span>
</template>
