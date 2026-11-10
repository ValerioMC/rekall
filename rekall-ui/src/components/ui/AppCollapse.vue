<script setup lang="ts">
/**
 * Folds its content away by animating the height to zero. Closed content is inert and hidden from
 * assistive technology once the fold has finished, so it is never reachable while out of sight.
 */
defineProps<{ open: boolean }>()
</script>

<template>
  <div class="app-collapse" :data-open="open" :inert="!open">
    <div class="app-collapse-body">
      <slot />
    </div>
  </div>
</template>

<style scoped>
.app-collapse {
  display: grid;
  grid-template-rows: 0fr;
  opacity: 0;
  visibility: hidden;
  transition:
    grid-template-rows var(--motion-move) cubic-bezier(0.16, 1, 0.3, 1),
    opacity var(--motion-quick) ease,
    visibility 0s linear var(--motion-move);
}

.app-collapse[data-open='true'] {
  grid-template-rows: 1fr;
  opacity: 1;
  visibility: visible;
  transition-delay: 0s;
}

.app-collapse-body {
  min-height: 0;
  overflow: hidden;
  margin: -3px;
  padding: 3px;
}
</style>
