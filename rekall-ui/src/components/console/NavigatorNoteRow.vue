<script setup lang="ts">
import { computed } from 'vue'
import NoteKindGlyph from '@/components/ui/NoteKindGlyph.vue'
import type { RekallDocument } from '@/model/catalog'
import type { IdentityHue } from '@/common/identity'

/**
 * One note in the notes navigator, drawn like a task row: its kind where a task shows its mark,
 * the title, and under it the first task it is on with how many more. A loose note has no second
 * line; its shelf already says it is on no task.
 */
const props = defineProps<{
  note: RekallDocument
  selected: boolean
  tint: IdentityHue | null
}>()

defineEmits<{ select: [] }>()

const firstTask = computed(() => props.note.tasks[0] ?? null)
const moreTasks = computed(() => Math.max(0, props.note.tasks.length - 1))
const byReference = computed(() => props.note.contextMode === 'REFERENCE')
</script>

<template>
  <button
    data-testid="note-row"
    class="focus-ring flex w-full items-start gap-2.5 rounded-control px-2.5 py-2 text-left transition-colors"
    :class="selected ? 'selected-row text-text' : 'text-text-muted hover:panel-slate hover:text-text'"
    :style="selected && tint ? { '--select-tint': tint.base, '--select-tint-soft': tint.soft } : undefined"
    :aria-current="selected"
    @click="$emit('select')"
  >
    <NoteKindGlyph class="mt-[3px]" :class="selected ? 'text-accent' : 'text-text-subtle'" :kind="note.kind" />

    <span class="min-w-0 flex-1">
      <span class="block truncate text-body font-medium leading-[1.35]">{{ note.title }}</span>
      <span v-if="firstTask || byReference" class="mt-1 flex min-w-0 items-center gap-1.5">
        <span v-if="firstTask" class="anchor-chip min-w-0 truncate px-1.5 py-px text-micro leading-[15px]">
          {{ firstTask.label }}
        </span>
        <span
          v-if="moreTasks"
          class="shrink-0 font-mono text-micro tabular-nums text-text-subtle"
          :title="`On ${note.tasks.length} tasks`"
        >
          +{{ moreTasks }}
        </span>
        <span
          v-if="byReference"
          class="ml-auto shrink-0 font-mono text-micro text-text-subtle"
          title="Loaded on request: sessions see its title and anchor, not its body"
          data-testid="note-row-reference"
        >
          ref
        </span>
      </span>
    </span>
  </button>
</template>
