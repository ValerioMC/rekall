<script setup lang="ts">
/** How sessions on the note's tasks receive it: the whole text, or a line and an anchor they load when they need it. */
import { useConsoleStore } from '@/stores/console.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import type { RekallDocument, DocumentContextMode } from '@/model/catalog'

const props = defineProps<{ note: RekallDocument }>()

const store = useConsoleStore()
const { run } = useAsyncAction()

async function setContextMode(contextMode: DocumentContextMode): Promise<void> {
  const note = props.note
  if (note.contextMode === contextMode) return
  await run(
    () => store.saveNote(note.id, { contextMode }),
    contextMode === 'REFERENCE'
      ? `Sessions now get a line and ${note.anchor}, and load the rest when they need it.`
      : 'Sessions now get this note in full.'
  )
}
</script>

<template>
  <div
    class="inline-flex items-center rounded-control border border-border p-0.5 text-caption"
    role="radiogroup"
    aria-label="How sessions get this note"
    data-testid="note-context-mode"
  >
    <button
      v-for="option in [
        { mode: 'FULL', label: 'In full', hint: 'Every session on its tasks reads the whole note.' },
        {
          mode: 'REFERENCE',
          label: 'By reference',
          hint: `Sessions get the title, the first line and ${note.anchor}, and load the rest only when the work needs it.`
        }
      ] as const"
      :key="option.mode"
      type="button"
      role="radio"
      :aria-checked="note.contextMode === option.mode"
      :title="option.hint"
      class="focus-ring rounded-[calc(var(--radius-control)-2px)] px-2 py-0.5 font-medium transition-colors"
      :class="
        note.contextMode === option.mode
          ? 'bg-accent-soft text-accent'
          : 'text-text-subtle hover:text-text'
      "
      :data-testid="`note-context-mode-${option.mode.toLowerCase()}`"
      @click="setContextMode(option.mode)"
    >
      {{ option.label }}
    </button>
  </div>
</template>
