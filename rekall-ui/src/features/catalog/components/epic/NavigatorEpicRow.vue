<script setup lang="ts">
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import EpicGlyph from '@/features/catalog/components/epic/EpicGlyph.vue'
import EpicPageLink from '@/features/catalog/components/epic/EpicPageLink.vue'
import StatusMixBar from '@/components/ui/StatusMixBar.vue'
import { tasksOfEpic } from '@/features/catalog/model/epic'
import type { Task } from '@/features/catalog/model/catalog'
import { useConsoleStore } from '@/stores/console.store'

/**
 * An epic in the navigator: its title over the status mix of the tasks it gathers. The row selects
 * it like a task, so its description and notes open in the console; the end cap opens its dashboard.
 */
const props = withDefaults(defineProps<{ epic: Task; selected: boolean; filed?: boolean }>(), { filed: false })

defineEmits<{ select: [] }>()

const { tasks } = storeToRefs(useConsoleStore())
const children = computed(() => tasksOfEpic(props.epic.id, tasks.value))
const done = computed(() => children.value.filter((task) => task.status === 'DONE').length)
</script>

<template>
  <div class="relative" :class="filed && 'filed-row'" data-testid="epic-row">
    <button
      class="epic-row focus-ring flex w-full items-start gap-2.5 rounded-control py-2 pl-2.5 pr-10 text-left transition-colors"
      :class="selected ? 'selected-row text-text' : 'text-text-muted hover:text-text'"
      :style="selected ? { '--select-tint': 'var(--color-epic)', '--select-tint-soft': 'var(--color-epic-soft)' } : undefined"
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
        <span class="mt-1.5 flex items-center gap-2">
          <StatusMixBar v-if="children.length" class="min-w-0 flex-1" :tasks="children" />
          <span v-else class="min-w-0 flex-1 truncate text-micro text-text-subtle">No task belongs to it yet</span>
          <span
            class="shrink-0 font-mono text-micro tabular-nums"
            :class="children.length && done === children.length ? 'text-safe' : 'text-text-subtle'"
            :title="`${done} of ${children.length} tasks done`"
            data-testid="epic-progress"
          >
            {{ done }}/{{ children.length }}
          </span>
        </span>
      </span>
    </button>
    <EpicPageLink class="absolute right-1.5 top-1.5" :epic-id="epic.id" :epic-title="epic.title" />
  </div>
</template>

<style scoped>
.epic-row:not([aria-current='true']) {
  background: linear-gradient(90deg, var(--color-epic-soft), transparent 70%);
}

.epic-row:not([aria-current='true']):hover {
  background: linear-gradient(90deg, var(--color-epic-soft), var(--color-surface-raised) 70%);
}
</style>
