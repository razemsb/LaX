<script setup lang="ts">
import { nextTick, onMounted, onUnmounted, ref, watch } from "vue";
import { contextMenuState, hideContextMenu } from "@/composables/contextMenu";

const menu = contextMenuState();
const box = ref<HTMLElement | null>(null);

function place() {
  const el = box.value;
  if (!el || !menu.open) return;
  const pad = 8;
  const r = el.getBoundingClientRect();
  let x = menu.x;
  let y = menu.y;
  if (x + r.width > window.innerWidth - pad) x = Math.max(pad, window.innerWidth - r.width - pad);
  if (y + r.height > window.innerHeight - pad) y = Math.max(pad, window.innerHeight - r.height - pad);
  menu.x = x;
  menu.y = y;
}

watch(
  () => [menu.open, menu.items],
  () => nextTick(place),
);

function onKey(e: KeyboardEvent) {
  if (e.key === "Escape") hideContextMenu();
}

function onPointer(e: PointerEvent) {
  if (!menu.open) return;
  const t = e.target as Node | null;
  if (box.value && t && box.value.contains(t)) return;
  hideContextMenu();
}

onMounted(() => {
  window.addEventListener("keydown", onKey);
  window.addEventListener("pointerdown", onPointer, true);
  window.addEventListener("resize", hideContextMenu);
  window.addEventListener("blur", hideContextMenu);
});
onUnmounted(() => {
  window.removeEventListener("keydown", onKey);
  window.removeEventListener("pointerdown", onPointer, true);
  window.removeEventListener("resize", hideContextMenu);
  window.removeEventListener("blur", hideContextMenu);
});

function pick(run: () => void) {
  hideContextMenu();
  run();
}
</script>

<template>
  <Teleport to="body">
    <div
      v-if="menu.open"
      ref="box"
      class="ctx-menu scrollbar"
      :style="{ left: menu.x + 'px', top: menu.y + 'px' }"
      @contextmenu.prevent
    >
      <template v-for="(item, i) in menu.items" :key="i">
        <div v-if="'type' in item && item.type === 'sep'" class="ctx-sep" />
        <button
          v-else
          type="button"
          class="ctx-item"
          :class="{ 'ctx-accent': 'accent' in item && item.accent, 'opacity-40': 'disabled' in item && item.disabled }"
          :disabled="'disabled' in item && item.disabled"
          @click="'run' in item && pick(item.run)"
        >
          <span class="min-w-0 truncate">{{ 'label' in item ? item.label : "" }}</span>
        </button>
      </template>
    </div>
  </Teleport>
</template>
