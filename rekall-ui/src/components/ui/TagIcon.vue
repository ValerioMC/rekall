<script setup lang="ts">
import { computed } from 'vue'
import { tagColor } from '@/model/tag'

const props = withDefaults(
  defineProps<{
    icon: string
    color: string
    size?: number
    glow?: boolean
  }>(),
  { size: 16, glow: true }
)

const style = computed(() => {
  const palette = tagColor(props.color)
  return { '--tag-tint': palette.base, color: palette.base }
})
</script>

<template>
  <svg
    class="shrink-0"
    :class="glow && 'tag-glow'"
    :style="style"
    :width="size"
    :height="size"
    viewBox="0 0 24 24"
    aria-hidden="true"
  >
    <polygon
      v-if="icon === 'star'"
      points="12,3 14.12,9.09 20.56,9.22 15.42,13.11 17.29,19.28 12,15.6 6.71,19.28 8.58,13.11 3.44,9.22 9.88,9.09"
      fill="currentColor"
    />
    <polygon
      v-else-if="icon === 'bolt'"
      points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"
      fill="currentColor"
    />
    <path
      v-else-if="icon === 'flame'"
      d="M12 21c-4.4 0-7-2.8-7-6.5C5 10 8 7 9 3c.3 3 2 4.5 3.5 6 1.8 1.8 2.5 3.5 2.5 5.5 0 .9-.2 1.7-.6 2.4.9-.5 1.6-1.4 1.6-2.9 0-1-.4-2-1-3 2 1 3 3.3 3 5.5 0 3.7-2.6 6-6 6Z"
      fill="currentColor"
    />
    <path
      v-else-if="icon === 'shield'"
      d="M12 2 4 5v6c0 5 3.4 9 8 11 4.6-2 8-6 8-11V5l-8-3Z"
      fill="currentColor"
    />
    <polygon v-else-if="icon === 'gem'" points="12 2 19 9 12 22 5 9" fill="currentColor" />
    <path
      v-else-if="icon === 'heart'"
      d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 1 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"
      fill="currentColor"
    />
    <template v-else-if="icon === 'target'">
      <circle cx="12" cy="12" r="9" fill="none" stroke="currentColor" stroke-width="1.7" />
      <circle cx="12" cy="12" r="5" fill="none" stroke="currentColor" stroke-width="1.7" />
      <circle cx="12" cy="12" r="1.8" fill="currentColor" />
    </template>
    <template v-else-if="icon === 'compass'">
      <circle cx="12" cy="12" r="9" fill="none" stroke="currentColor" stroke-width="1.7" />
      <polygon points="15.2 8.8 13 13 8.8 15.2 11 11 15.2 8.8" fill="currentColor" />
    </template>
    <template v-else-if="icon === 'flag'">
      <line x1="6" y1="3" x2="6" y2="21" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" />
      <path d="M6 4c3-1.5 5 1.5 8 0v8c-3 1.5-5-1.5-8 0V4Z" fill="currentColor" />
    </template>
    <polygon
      v-else-if="icon === 'crown'"
      points="3 19 3 9 7.5 13 12 6 16.5 13 21 9 21 19"
      fill="currentColor"
    />
    <polygon
      v-else
      points="12,3 14.12,9.09 20.56,9.22 15.42,13.11 17.29,19.28 12,15.6 6.71,19.28 8.58,13.11 3.44,9.22 9.88,9.09"
      fill="currentColor"
    />
  </svg>
</template>
