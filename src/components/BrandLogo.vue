<script setup lang="ts">
import logo from "@/assets/logo.svg";

defineProps<{ size?: string }>();
</script>

<template>
  <img
    :src="logo"
    alt="LaX"
    :class="size ?? 'h-10 w-10'"
  />
</template>
