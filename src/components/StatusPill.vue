<script setup lang="ts">
defineProps<{ on?: boolean; label?: string }>();
</script>

<template>
  <span
    class="inline-flex shrink-0 items-center gap-1.5 whitespace-nowrap rounded-full px-2 py-1 text-[11px] font-medium lg:gap-2 lg:px-2.5"
    :class="on ? 'bg-ok/15 text-ok' : 'bg-white/5 text-muted'"
  >
    <span class="h-1.5 w-1.5 rounded-full" :class="on ? 'bg-ok' : 'bg-[#3a3a3e]'" />
    {{ label ?? (on ? "онлайн" : "выкл") }}
  </span>
</template>
