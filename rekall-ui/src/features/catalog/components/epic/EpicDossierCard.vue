<script setup lang="ts">
import { computed } from 'vue'
import EpicGlyph from '@/features/catalog/components/epic/EpicGlyph.vue'
import StatusMixBar from '@/components/ui/StatusMixBar.vue'
import { excerpt as previewOf } from '@/common/format/excerpt'
import type { Task } from '@/features/catalog/model/catalog'

/** Where a task's dossier shows its description and steps, an epic shows one card: its tasks and its description, opened together. */
const props = defineProps<{ epic: Task; children: readonly Task[]; selected: boolean }>()

const emit = defineEmits<{ open: [] }>()

const done = computed(() => props.children.filter((task) => task.status === 'DONE').length)
const excerpt = computed(() => previewOf(props.epic.description ?? ''))
</script>

<template>
  <button
    data-testid="epic-dossier-card"
    class="dossier-section"
    :class="selected && 'dossier-section-selected'"
    :aria-current="selected"
    @click="emit('open')"
  >
    <span
      class="dossier-node"
      :class="selected ? 'dossier-node-lit' : !children.length && 'dossier-node-empty'"
      aria-hidden="true"
    >
      <EpicGlyph :size="9" />
    </span>
    <span class="flex min-h-[17px] items-center gap-2">
      <span
        class="text-caption font-semibold tracking-[0.005em] transition-colors"
        :class="selected ? 'text-text' : 'text-text-muted'"
      >
        Epic tasks
      </span>
      <span
        class="ml-auto shrink-0 font-mono text-micro tabular-nums"
        :class="children.length && done === children.length ? 'text-safe' : 'text-text-muted'"
        :title="`${done} of ${children.length} tasks done`"
        data-testid="epic-dossier-progress"
      >
        {{ done }}/{{ children.length }}
      </span>
    </span>
    <StatusMixBar v-if="children.length" class="mt-2" :tasks="children" size="md" />
    <span v-else class="mt-1 block text-caption leading-relaxed text-text-muted">No task belongs to it yet.</span>
    <span v-if="excerpt" class="mt-1.5 line-clamp-2 block text-small leading-relaxed text-text-muted">{{ excerpt }}</span>
    <span v-else class="mt-1.5 block text-caption leading-relaxed text-text-muted">
      No description yet. Open it to write the concept its tasks share.
    </span>
  </button>
</template>
