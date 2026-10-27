<script setup lang="ts">
/** The epic's mark: three stacked plates, the tasks it gathers. Drawn in the epic tone. */
withDefaults(defineProps<{ size?: number }>(), { size: 14 })
</script>

<template>
  <svg :width="size" :height="size" viewBox="0 0 16 16" fill="none" class="shrink-0 text-epic" aria-hidden="true">
    <path d="M8 1.6 14.2 4.8 8 8 1.8 4.8z" fill="currentColor" fill-opacity="0.32" stroke="currentColor" stroke-width="1.3" stroke-linejoin="round" />
    <path d="M1.8 8 8 11.2 14.2 8" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round" />
    <path d="M1.8 11.2 8 14.4 14.2 11.2" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round" opacity="0.6" />
  </svg>
</template>
