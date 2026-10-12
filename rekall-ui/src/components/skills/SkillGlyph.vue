<script setup lang="ts">
/** The four-point spark that stands for a skill wherever one is shown. */
withDefaults(defineProps<{ size?: number }>(), { size: 12 })
</script>

<template>
  <svg :width="size" :height="size" viewBox="0 0 12 12" fill="none" aria-hidden="true" class="shrink-0">
    <path
      d="M6 1l1.2 3.8L11 6 7.2 7.2 6 11 4.8 7.2 1 6l3.8-1.2L6 1z"
      fill="currentColor"
      fill-opacity="0.22"
      stroke="currentColor"
      stroke-width="1"
      stroke-linejoin="round"
    />
  </svg>
</template>
