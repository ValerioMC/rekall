<script setup lang="ts">
import { useId } from 'vue'

/**
 * A shelf under a project's live tasks. `filed` holds the finished ones on a verdigris rail;
 * `backlog` holds the parked ones on a dashed grey rail, work that exists but is not started.
 */
withDefaults(defineProps<{ count: number; open: boolean; kind?: 'filed' | 'backlog' }>(), {
  kind: 'filed'
})
defineEmits<{ toggle: [] }>()

const bodyId = useId()
</script>

<template>
  <div>
    <button
      class="focus-ring group/filed flex w-full items-center gap-2 rounded-[var(--radius-control)] px-1.5 py-1.5 text-left text-text-subtle transition-colors hover:bg-surface-raised"
      :class="kind === 'filed' ? 'hover:text-filed' : 'hover:text-text-muted'"
      :aria-expanded="open"
      :aria-controls="bodyId"
      :data-kind="kind"
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

      <svg
        v-if="kind === 'filed'"
        class="size-3.5 shrink-0 text-filed"
        viewBox="0 0 24 24"
        fill="none"
        aria-hidden="true"
      >
        <path
          d="M4 7h16M4 7l1.2 11.2a2 2 0 0 0 2 1.8h9.6a2 2 0 0 0 2-1.8L20 7M9 7V5a2 2 0 0 1 2-2h2a2 2 0 0 1 2 2v2"
          stroke="currentColor"
          stroke-width="1.7"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
      <svg v-else class="size-3.5 shrink-0 text-text-muted" viewBox="0 0 24 24" fill="none" aria-hidden="true">
        <path
          d="M4 14h4.5l1.5 2.5h4l1.5-2.5H20M4 14l2.2-7.4A2 2 0 0 1 8.1 5h7.8a2 2 0 0 1 1.9 1.6L20 14v4a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2v-4Z"
          stroke="currentColor"
          stroke-width="1.7"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>

      <span class="min-w-0 flex-1 truncate text-[11.5px] font-semibold">
        {{ count }} {{ kind === 'filed' ? 'filed' : 'in backlog' }}
      </span>

      <span
        class="shrink-0 font-mono text-[10px] text-text-subtle"
        :class="kind === 'filed' ? 'group-hover/filed:text-filed' : 'group-hover/filed:text-text-muted'"
      >
        {{ open ? 'hide' : 'show' }}
      </span>
    </button>

    <div
      v-if="open"
      :id="bodyId"
      class="drawer-open mt-0.5 flex flex-col gap-0.5"
      :class="kind === 'filed' ? 'filed-shelf' : 'backlog-shelf'"
    >
      <slot />
    </div>
  </div>
</template>
