<script setup lang="ts">
/** Move up, move down and delete, as a checklist row and a draft row both offer them. */
defineProps<{
  title: string
  canMoveUp: boolean
  canMoveDown: boolean
  /** `step` or `draft`: the prefix of each button's test id. */
  testidPrefix: string
}>()

defineEmits<{ up: []; down: []; remove: [] }>()

const ICON_BUTTON =
  'focus-ring grid size-6 place-items-center rounded-key text-text-subtle transition-colors hover:bg-surface-hover'
</script>

<template>
  <button
    :class="[ICON_BUTTON, 'hover:text-text disabled:opacity-30']"
    :disabled="!canMoveUp"
    :aria-label="`Move ${title} up`"
    :data-testid="`${testidPrefix}-up`"
    @click="$emit('up')"
  >
    <svg class="size-3" viewBox="0 0 12 12" fill="none" aria-hidden="true">
      <path d="M6 9.5v-7M2.8 5.4 6 2.2l3.2 3.2" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round" />
    </svg>
  </button>
  <button
    :class="[ICON_BUTTON, 'hover:text-text disabled:opacity-30']"
    :disabled="!canMoveDown"
    :aria-label="`Move ${title} down`"
    :data-testid="`${testidPrefix}-down`"
    @click="$emit('down')"
  >
    <svg class="size-3" viewBox="0 0 12 12" fill="none" aria-hidden="true">
      <path d="M6 2.5v7M2.8 6.6 6 9.8l3.2-3.2" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round" />
    </svg>
  </button>
  <button
    :class="ICON_BUTTON"
    :aria-label="`Delete ${title}`"
    :data-testid="`${testidPrefix}-delete`"
    @click="$emit('remove')"
  >
    <svg class="size-3" viewBox="0 0 12 12" fill="none" aria-hidden="true">
      <path d="M2.6 2.6l6.8 6.8M9.4 2.6 2.6 9.4" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" />
    </svg>
  </button>
</template>
