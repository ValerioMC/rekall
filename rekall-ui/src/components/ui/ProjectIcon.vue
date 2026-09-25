<script setup lang="ts">
import { computed } from 'vue'
import { projectIconKey } from '@/model/project-icon'

const props = withDefaults(defineProps<{ icon: string; size?: number }>(), { size: 14 })

const key = computed(() => projectIconKey(props.icon))
</script>

<template>
  <svg
    class="shrink-0"
    :width="size"
    :height="size"
    viewBox="0 0 24 24"
    fill="none"
    aria-hidden="true"
  >
    <path
      v-if="key === 'folder'"
      d="M4 6.5A1.5 1.5 0 0 1 5.5 5h4.1a1.5 1.5 0 0 1 1.2.6l1 1.4h7.7A1.5 1.5 0 0 1 21 8.5v10A1.5 1.5 0 0 1 19.5 20h-14A1.5 1.5 0 0 1 4 18.5v-12z"
      fill="currentColor"
    />

    <template v-else-if="key === 'rocket'">
      <path
        d="M12 2c2.6 2 4 5.2 4 8.6 0 2.1-.5 4-1.5 5.6l-.5.8v3l-2-1.4-2 1.4v-3l-.5-.8C8.5 14.6 8 12.7 8 10.6 8 7.2 9.4 4 12 2z"
        fill="currentColor"
      />
      <circle cx="12" cy="10" r="1.6" fill="none" stroke="currentColor" stroke-width="1.4" />
    </template>

    <polygon
      v-else-if="key === 'star'"
      points="12,3 14.12,9.09 20.56,9.22 15.42,13.11 17.29,19.28 12,15.6 6.71,19.28 8.58,13.11 3.44,9.22 9.88,9.09"
      fill="currentColor"
    />

    <polygon
      v-else-if="key === 'bolt'"
      points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"
      fill="currentColor"
    />

    <path
      v-else-if="key === 'flame'"
      d="M12 21c-4.4 0-7-2.8-7-6.5C5 10 8 7 9 3c.3 3 2 4.5 3.5 6 1.8 1.8 2.5 3.5 2.5 5.5 0 .9-.2 1.7-.6 2.4.9-.5 1.6-1.4 1.6-2.9 0-1-.4-2-1-3 2 1 3 3.3 3 5.5 0 3.7-2.6 6-6 6Z"
      fill="currentColor"
    />

    <path
      v-else-if="key === 'shield'"
      d="M12 2 4 5v6c0 5 3.4 9 8 11 4.6-2 8-6 8-11V5l-8-3Z"
      fill="currentColor"
    />

    <polygon v-else-if="key === 'gem'" points="12 2 19 9 12 22 5 9" fill="currentColor" />

    <path
      v-else-if="key === 'heart'"
      d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 1 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"
      fill="currentColor"
    />

    <template v-else-if="key === 'target'">
      <circle cx="12" cy="12" r="9" fill="none" stroke="currentColor" stroke-width="1.7" />
      <circle cx="12" cy="12" r="5" fill="none" stroke="currentColor" stroke-width="1.7" />
      <circle cx="12" cy="12" r="1.8" fill="currentColor" />
    </template>

    <template v-else-if="key === 'compass'">
      <circle cx="12" cy="12" r="9" fill="none" stroke="currentColor" stroke-width="1.7" />
      <polygon points="15.2 8.8 13 13 8.8 15.2 11 11 15.2 8.8" fill="currentColor" />
    </template>

    <template v-else-if="key === 'flag'">
      <line x1="6" y1="3" x2="6" y2="21" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" />
      <path d="M6 4c3-1.5 5 1.5 8 0v8c-3 1.5-5-1.5-8 0V4Z" fill="currentColor" />
    </template>

    <polygon
      v-else-if="key === 'crown'"
      points="3 19 3 9 7.5 13 12 6 16.5 13 21 9 21 19"
      fill="currentColor"
    />

    <template v-else-if="key === 'box'">
      <polygon points="12,2.5 20,7 20,17 12,21.5 4,17 4,7" fill="currentColor" />
      <polygon points="12,2.5 20,7 12,11.5 4,7" fill="currentColor" opacity="0.35" />
    </template>

    <template v-else-if="key === 'briefcase'">
      <rect x="3" y="8" width="18" height="11" rx="2" fill="currentColor" />
      <path d="M9 8V6.5A2.5 2.5 0 0 1 11.5 4h1A2.5 2.5 0 0 1 15 6.5V8" fill="none" stroke="currentColor" stroke-width="1.8" />
    </template>

    <path
      v-else-if="key === 'beaker'"
      d="M9.5 2.5h5v5.3l4.4 8.3a2 2 0 0 1-1.8 2.9H6.9a2 2 0 0 1-1.8-2.9l4.4-8.3z"
      fill="currentColor"
    />

    <template v-else-if="key === 'book'">
      <path d="M3 5.5c2.3-1 4.8-1.3 7-.6v13.6c-2.2-.7-4.7-.4-7 .6z" fill="currentColor" />
      <path d="M21 5.5c-2.3-1-4.8-1.3-7-.6v13.6c2.2-.7 4.7-.4 7 .6z" fill="currentColor" />
    </template>

    <template v-else-if="key === 'bulb'">
      <circle cx="12" cy="9" r="6" fill="currentColor" />
      <rect x="9.5" y="15" width="5" height="2.2" rx="1" fill="currentColor" />
      <rect x="10" y="18" width="4" height="1.8" rx="0.9" fill="currentColor" />
    </template>

    <template v-else-if="key === 'camera'">
      <path
        d="M9 4.5 7.8 6.5H5A2 2 0 0 0 3 8.5v9A2 2 0 0 0 5 19.5h14a2 2 0 0 0 2-2v-9a2 2 0 0 0-2-2h-2.8L15 4.5z"
        fill="currentColor"
      />
      <circle cx="12" cy="13" r="3.4" fill="none" stroke="currentColor" stroke-width="1.8" />
    </template>

    <template v-else-if="key === 'chart'">
      <rect x="4" y="12" width="3.5" height="8" rx="0.8" fill="currentColor" />
      <rect x="10.2" y="7" width="3.5" height="13" rx="0.8" fill="currentColor" />
      <rect x="16.4" y="3" width="3.5" height="17" rx="0.8" fill="currentColor" />
    </template>

    <path
      v-else-if="key === 'cloud'"
      d="M7 18a4.5 4.5 0 0 1-.6-8.96 5.5 5.5 0 0 1 10.7-1.6A4 4 0 0 1 17.5 18z"
      fill="currentColor"
    />

    <template v-else-if="key === 'code'">
      <path d="M9 6 3.5 12 9 18" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
      <path d="M15 6l5.5 6-5.5 6" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
    </template>

    <template v-else-if="key === 'cube'">
      <polygon points="12,3 19,7 12,11 5,7" fill="currentColor" />
      <polygon points="5,7 12,11 12,20 5,16" fill="currentColor" opacity="0.55" />
      <polygon points="19,7 12,11 12,20 19,16" fill="currentColor" opacity="0.8" />
    </template>

    <template v-else-if="key === 'database'">
      <ellipse cx="12" cy="5.5" rx="7.5" ry="2.7" fill="currentColor" />
      <path d="M4.5 5.5v13c0 1.5 3.4 2.7 7.5 2.7s7.5-1.2 7.5-2.7v-13" fill="none" stroke="currentColor" stroke-width="1.8" />
      <path d="M4.5 11.8c0 1.5 3.4 2.7 7.5 2.7s7.5-1.2 7.5-2.7" fill="none" stroke="currentColor" stroke-width="1.8" />
    </template>

    <template v-else-if="key === 'gear'">
      <circle cx="12" cy="12" r="3.2" fill="currentColor" />
      <circle cx="12" cy="12" r="7.2" fill="none" stroke="currentColor" stroke-width="2" stroke-dasharray="2.3 2.1" />
    </template>

    <template v-else-if="key === 'globe'">
      <circle cx="12" cy="12" r="8.5" fill="none" stroke="currentColor" stroke-width="1.7" />
      <ellipse cx="12" cy="12" rx="3.6" ry="8.5" fill="none" stroke="currentColor" stroke-width="1.7" />
      <line x1="3.5" y1="12" x2="20.5" y2="12" stroke="currentColor" stroke-width="1.7" />
    </template>

    <template v-else-if="key === 'key'">
      <circle cx="7.5" cy="12" r="4.2" fill="none" stroke="currentColor" stroke-width="2" />
      <path d="M11 12h10M17.5 12v3M20.5 12v2.5" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
    </template>

    <template v-else-if="key === 'layers'">
      <polygon points="12,3 21,8 12,13 3,8" fill="currentColor" />
      <polyline points="3,12 12,17 21,12" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" />
      <polyline points="3,16 12,21 21,16" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" />
    </template>

    <path
      v-else-if="key === 'map'"
      d="M3 6.5 9 4l6 2.5 6-2.5v15l-6 2.5-6-2.5-6 2.5z"
      fill="currentColor"
    />

    <path
      v-else-if="key === 'palette'"
      d="M12 3a9 9 0 1 0 0 18c1.1 0 2-.9 2-2 0-.5-.2-1-.5-1.3-.3-.3-.5-.8-.5-1.3 0-1.1.9-2 2-2h2A4 4 0 0 0 21 10.4 9 9 0 0 0 12 3z"
      fill="currentColor"
    />

    <template v-else-if="key === 'grid'">
      <rect x="4" y="4" width="7.5" height="7.5" rx="1.6" fill="currentColor" />
      <rect x="12.5" y="4" width="7.5" height="7.5" rx="1.6" fill="currentColor" opacity="0.8" />
      <rect x="4" y="12.5" width="7.5" height="7.5" rx="1.6" fill="currentColor" opacity="0.55" />
      <rect x="12.5" y="12.5" width="7.5" height="7.5" rx="1.6" fill="currentColor" opacity="0.35" />
    </template>

    <template v-else-if="key === 'terminal'">
      <rect x="3" y="4" width="18" height="16" rx="2" fill="none" stroke="currentColor" stroke-width="1.8" />
      <path d="M7 9.5 10.5 12 7 14.5M12.5 15h4.5" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" fill="none" />
    </template>

    <path
      v-else-if="key === 'wrench'"
      d="M14.7 3.3a5 5 0 0 0-6.7 6l-6 6a2 2 0 0 0 2.8 2.8l6-6a5 5 0 0 0 6-6.7l-3 3-2-2z"
      fill="currentColor"
    />

    <path
      v-else
      d="M4 6.5A1.5 1.5 0 0 1 5.5 5h4.1a1.5 1.5 0 0 1 1.2.6l1 1.4h7.7A1.5 1.5 0 0 1 21 8.5v10A1.5 1.5 0 0 1 19.5 20h-14A1.5 1.5 0 0 1 4 18.5v-12z"
      fill="currentColor"
    />
  </svg>
</template>
