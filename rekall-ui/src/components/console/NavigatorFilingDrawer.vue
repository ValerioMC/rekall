<script setup lang="ts">
import { useId } from 'vue'

/**
 * The drawer finished tasks are folded into.
 *
 * Closed on every load and reopened only for the session: done work is out of the way by
 * default, one click from view, and back out of the way next time the window opens. The count
 * stays on the row while it is closed, so the pile is never a silent nothing.
 */
defineProps<{ count: number; open: boolean }>()
defineEmits<{ toggle: [] }>()

const bodyId = useId()
</script>

<template>
  <div>
    <button
      class="focus-ring group/filed flex w-full items-center gap-2 rounded-[var(--radius-control)] px-1.5 py-1.5 text-left text-text-subtle transition-colors hover:bg-surface-raised hover:text-filed"
      :aria-expanded="open"
      :aria-controls="bodyId"
      data-testid="filing-drawer-toggle"
      @click="$emit('toggle')"
    >
      <svg
        class="size-3 shrink-0 transition-transform"
        :class="open && 'rotate-90'"
        viewBox="0 0 24 24"
        fill="none"
        aria-hidden="true"
      >
        <path
          d="M9 6l6 6-6 6"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>

      <svg class="size-3.5 shrink-0 text-filed" viewBox="0 0 24 24" fill="none" aria-hidden="true">
        <path
          d="M4 7h16M4 7l1.2 11.2a2 2 0 0 0 2 1.8h9.6a2 2 0 0 0 2-1.8L20 7M9 7V5a2 2 0 0 1 2-2h2a2 2 0 0 1 2 2v2"
          stroke="currentColor"
          stroke-width="1.7"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>

      <span class="min-w-0 flex-1 truncate text-[11.5px] font-semibold">
        {{ count }} filed
      </span>

      <span class="shrink-0 font-mono text-[10px] text-text-subtle group-hover/filed:text-filed">
        {{ open ? 'hide' : 'show' }}
      </span>
    </button>

    <div v-if="open" :id="bodyId" class="drawer-open filed-shelf mt-0.5 flex flex-col gap-0.5">
      <slot />
    </div>
  </div>
</template>
