<script setup lang="ts">
/** The hint on a show/hide toggle: an open eye while the value is hidden, a struck-through one while it is shown. */
defineProps<{ shown: boolean }>()
</script>

<template>
  <svg class="size-3.5 shrink-0" viewBox="0 0 16 16" fill="none" aria-hidden="true" data-testid="eye-glyph" :data-shown="shown ? 'true' : 'false'">
    <path
      d="M1.5 8s2.3-4.5 6.5-4.5S14.5 8 14.5 8 12.2 12.5 8 12.5 1.5 8 1.5 8z"
      stroke="currentColor"
      stroke-width="1.3"
      stroke-linejoin="round"
    />
    <circle cx="8" cy="8" r="2" stroke="currentColor" stroke-width="1.3" />
    <path v-if="shown" d="M2.5 13.5l11-11" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
  </svg>
</template>
