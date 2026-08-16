<script setup lang="ts">
defineProps<{ on?: boolean; label?: string }>();
</script>

<template>
  <span class="inline-flex items-center gap-2 text-xs" :class="on ? 'text-ok' : 'text-muted'">
    <span class="h-1.5 w-1.5 rounded-full" :class="on ? 'bg-ok' : 'bg-[#3a3a3e]'" />
    {{ label ?? (on ? "работает" : "остановлен") }}
  </span>
</template>
