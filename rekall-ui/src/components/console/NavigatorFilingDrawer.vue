<script setup lang="ts">
import { useId } from 'vue'
import EpicGlyph from '@/features/catalog/components/epic/EpicGlyph.vue'

/**
 * A shelf beside a card's live rows. `epics` holds the epics on a violet rail; `filed` holds finished
 * tasks on a verdigris one; `backlog` parked tasks and `loose` notes on no task, on a dashed grey rail.
 */
type ShelfKind = 'epics' | 'filed' | 'backlog' | 'loose'

const SHELF_CLASS: Readonly<Record<ShelfKind, string>> = {
  epics: 'epic-shelf',
  filed: 'filed-shelf',
  backlog: 'backlog-shelf',
  loose: 'backlog-shelf'
}

withDefaults(defineProps<{ count: number; open: boolean; kind?: ShelfKind }>(), {
  kind: 'filed'
})

const SHELF_LABEL: Readonly<Record<ShelfKind, string>> = {
  epics: 'epics',
  filed: 'filed',
  backlog: 'in backlog',
  loose: 'on no task'
}
defineEmits<{ toggle: [] }>()

const bodyId = useId()
</script>

<template>
  <div>
    <button
      class="focus-ring group/filed flex w-full items-center gap-2 rounded-control px-1.5 py-1.5 text-left text-text-subtle transition-colors hover:bg-surface-raised"
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

      <span v-if="kind === 'epics'" class="grid size-3.5 shrink-0 place-items-center" aria-hidden="true">
        <EpicGlyph :size="13" />
      </span>
      <svg
        v-else-if="kind === 'filed'"
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
      <svg
        v-else-if="kind === 'loose'"
        class="size-3.5 shrink-0 text-text-muted"
        viewBox="0 0 24 24"
        fill="none"
        aria-hidden="true"
      >
        <path
          d="M7 3.5h7l4 4V20a.5.5 0 0 1-.5.5h-10.5a.5.5 0 0 1-.5-.5V4a.5.5 0 0 1 .5-.5Z"
          stroke="currentColor"
          stroke-width="1.7"
          stroke-dasharray="2.6 2.4"
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

      <span class="min-w-0 flex-1 truncate text-caption font-semibold">
        {{ count }} {{ SHELF_LABEL[kind] }}
      </span>

      <span
        class="shrink-0 font-mono text-micro text-text-subtle"
        :class="kind === 'filed' ? 'group-hover/filed:text-filed' : 'group-hover/filed:text-text-muted'"
      >
        {{ open ? 'hide' : 'show' }}
      </span>
    </button>

    <div
      v-if="open"
      :id="bodyId"
      class="drawer-open mt-0.5 flex flex-col gap-0.5"
      :class="SHELF_CLASS[kind]"
    >
      <slot />
    </div>
  </div>
</template>
