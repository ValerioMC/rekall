<script setup lang="ts">
export type TrackerSyncState = 'pending' | 'syncing' | 'synced' | 'failed'

/**
 * The hint on the sync control: two chasing arrows, a check once the issue holds what the task
 * holds, and a dot on the arrows while the task has changed since. It carries no text of its own;
 * the button it sits in says what pressing does.
 */
defineProps<{ state: TrackerSyncState }>()
</script>

<template>
  <span
    class="relative grid size-3.5 shrink-0 place-items-center"
    :data-state="state"
    data-testid="tracker-sync-glyph"
    aria-hidden="true"
  >
    <Transition name="tracker-sync-glyph" mode="out-in">
      <svg v-if="state === 'synced'" key="check" class="size-3.5" viewBox="0 0 24 24" fill="none">
        <path d="m5 12.5 4.5 4.5L19 7.5" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round" />
      </svg>
      <svg
        v-else
        key="arrows"
        class="size-3.5 motion-reduce:opacity-60"
        :class="state === 'syncing' && 'animate-spin motion-reduce:animate-none'"
        viewBox="0 0 24 24"
        fill="none"
      >
        <path
          d="M3.5 12a8.5 8.5 0 0 1 14.6-5.9L20.5 8.5M20.5 3.5v5h-5M20.5 12a8.5 8.5 0 0 1-14.6 5.9L3.5 15.5M3.5 20.5v-5h5"
          stroke="currentColor"
          stroke-width="2.4"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
    </Transition>
    <span
      v-if="state === 'pending'"
      class="absolute -right-0.5 -top-0.5 size-1.5 rounded-full bg-accent ring-2 ring-canvas"
      data-testid="tracker-sync-changed"
    />
  </span>
</template>

<style scoped>
.tracker-sync-glyph-enter-active,
.tracker-sync-glyph-leave-active {
  transition:
    opacity var(--motion-quick) ease,
    transform var(--motion-quick) ease;
}
.tracker-sync-glyph-enter-from,
.tracker-sync-glyph-leave-to {
  opacity: 0;
  transform: scale(0.55);
}
</style>
