<script setup lang="ts">
import { computed, useId } from 'vue'
import EpicGlyph from '@/features/catalog/components/epic/EpicGlyph.vue'
import StatusMixBar from '@/components/ui/StatusMixBar.vue'
import type { Task } from '@/features/catalog/model/catalog'

/**
 * An epic's head in the navigator: a chevron that opens or closes its tasks, then its title over the
 * status mix of those tasks. The title selects the epic like any task; the chevron only folds.
 */
const props = defineProps<{ epic: Task; tasks: readonly Task[]; selected: boolean; expanded: boolean; controls: string }>()

defineEmits<{ select: []; toggle: [] }>()

const done = computed(() => props.tasks.filter((task) => task.status === 'DONE').length)
const hasChildren = computed(() => props.tasks.length > 0)
const toggleLabel = computed(() => `${props.expanded ? 'Close' : 'Open'} the tasks of ${props.epic.title}`)
const headId = useId()
</script>

<template>
  <div
    :id="headId"
    class="epic-head flex items-start gap-0.5 rounded-control transition-colors"
    :class="selected ? 'selected-row text-text' : 'text-text-muted hover:text-text'"
    :style="selected ? { '--select-tint': 'var(--color-epic)', '--select-tint-soft': 'var(--color-epic-soft)' } : undefined"
    data-testid="epic-row"
  >
    <button
      v-if="hasChildren"
      type="button"
      class="focus-ring mt-[5px] grid size-6 shrink-0 place-items-center rounded-key text-text-subtle transition-colors hover:text-epic"
      :aria-expanded="expanded"
      :aria-controls="controls"
      :aria-label="toggleLabel"
      data-testid="epic-toggle"
      @click="$emit('toggle')"
    >
      <svg
        class="size-3 transition-transform duration-(--motion-move) ease-out"
        :class="expanded && 'rotate-90'"
        viewBox="0 0 24 24"
        fill="none"
        aria-hidden="true"
      >
        <path d="M9 6l6 6-6 6" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
      </svg>
    </button>
    <span v-else class="size-6 shrink-0" aria-hidden="true" />

    <button
      type="button"
      class="epic-title focus-ring flex min-w-0 flex-1 items-start gap-2.5 rounded-control py-2 pl-1 pr-2.5 text-left"
      :aria-current="selected"
      @click="$emit('select')"
    >
      <span class="mt-px grid size-[18px] shrink-0 place-items-center rounded-key border border-epic-line bg-epic-soft">
        <EpicGlyph :size="12" />
      </span>
      <span class="min-w-0 flex-1">
        <span class="flex items-center gap-1.5">
          <span class="min-w-0 truncate text-body font-semibold leading-[1.35]">{{ epic.title }}</span>
          <span class="sr-only">Epic.</span>
        </span>
        <span class="mt-1 block truncate font-mono text-micro text-text-subtle" data-testid="epic-code">{{ epic.code }}</span>
        <span class="mt-1.5 flex items-center gap-2">
          <StatusMixBar v-if="hasChildren" class="min-w-0 flex-1" :tasks="tasks" />
          <span v-else class="min-w-0 flex-1 truncate text-micro text-text-subtle">No task belongs to it yet</span>
          <span
            class="shrink-0 font-mono text-micro tabular-nums"
            :class="hasChildren && done === tasks.length ? 'text-safe' : 'text-text-subtle'"
            :title="`${done} of ${tasks.length} tasks done`"
            data-testid="epic-progress"
          >
            {{ done }}/{{ tasks.length }}
          </span>
        </span>
      </span>
    </button>
  </div>
</template>

<style scoped>
.epic-head:not(.selected-row) {
  background: linear-gradient(90deg, var(--color-epic-soft), transparent 70%);
}

.epic-head:not(.selected-row):hover {
  background: linear-gradient(90deg, var(--color-epic-soft), var(--color-surface-raised) 70%);
}
</style>
