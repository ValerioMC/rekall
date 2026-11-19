<script setup lang="ts">
import { computed, ref } from 'vue'
import { storeToRefs } from 'pinia'
import AppButton from '@/components/ui/AppButton.vue'
import NoteKindGlyph from '@/components/ui/NoteKindGlyph.vue'
import NoteDeleteConfirm from '@/features/notes/components/NoteDeleteConfirm.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useConsoleStore } from '@/stores/console.store'
import type { Task } from '@/features/catalog/model/catalog'
import type { RekallDocument } from '@/features/notes/model/document'

/** The task's notes as a list beside the editor: pick one, add information, delete one. A note on request wears an open ring. */
const props = defineProps<{ task: Task; current: RekallDocument | null }>()

const emit = defineEmits<{ select: [note: RekallDocument] }>()

const store = useConsoleStore()
const { taskDocuments } = storeToRefs(store)
const { run } = useAsyncAction()
const { run: runCreate, isRunning: isCreating } = useAsyncAction()

const asking = ref<RekallDocument | null>(null)
const hasReferenceNote = computed(() => taskDocuments.value.some((note) => note.contextMode === 'REFERENCE'))

async function add(): Promise<void> {
  if (isCreating.value) return
  await runCreate(() => store.createNote(props.task.id), 'Information added')
}

async function confirmDelete(): Promise<void> {
  const note = asking.value
  if (!note) return
  asking.value = null
  await run(() => store.deleteNote(note.id), `Deleted ${note.title}`)
}
</script>

<template>
  <nav class="flex min-h-0 w-64 shrink-0 flex-col gap-0.5 overflow-y-auto border-r border-border bg-surface p-2" aria-label="Notes" data-testid="task-note-list">
    <div
      v-for="note in taskDocuments"
      :key="note.id"
      class="group/note relative"
    >
      <button
        type="button"
        class="focus-ring flex h-9 w-full cursor-pointer items-center gap-2 rounded-control px-2.5 pr-9 text-left text-small transition-colors duration-(--motion-quick)"
        :class="current?.id === note.id ? 'bg-surface-raised text-text' : 'text-text-muted hover:bg-surface-hover hover:text-text'"
        :aria-current="current?.id === note.id ? 'true' : undefined"
        data-testid="task-note"
        @click="emit('select', note)"
        @keydown.delete.prevent="asking = note"
      >
        <NoteKindGlyph :kind="note.kind" class="shrink-0 text-text-subtle" />
        <span class="min-w-0 flex-1 truncate">{{ note.title }}</span>
        <span
          class="size-[7px] shrink-0 rounded-full"
          :class="note.contextMode === 'REFERENCE' ? 'border-[1.5px] border-text-subtle' : 'bg-text-subtle'"
          :title="note.contextMode === 'REFERENCE' ? 'On request' : 'In full'"
          :data-load-mode="note.contextMode === 'REFERENCE' ? 'on-request' : 'in-full'"
          data-testid="task-note-marker"
        />
      </button>
      <button
        type="button"
        class="focus-ring absolute right-1.5 top-1/2 grid size-6 -translate-y-1/2 place-items-center rounded-control text-text-subtle opacity-0 transition-[opacity,color,background-color] duration-(--motion-quick) hover:bg-danger-soft hover:text-danger focus-visible:opacity-100 group-focus-within/note:opacity-100 group-hover/note:opacity-100"
        :aria-label="`Delete ${note.title}`"
        data-testid="task-note-delete"
        @click="asking = note"
      >
        <svg class="size-3" viewBox="0 0 12 12" fill="none" aria-hidden="true">
          <path d="M3 3l6 6M9 3l-6 6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
        </svg>
      </button>
    </div>

    <p v-if="hasReferenceNote" class="flex items-center gap-1.5 px-2.5 py-1 text-micro text-text-subtle" data-testid="note-reference-legend">
      <span class="size-[7px] rounded-full border-[1.5px] border-text-subtle" aria-hidden="true" />
      on request
    </p>

    <AppButton variant="quiet" size="sm" class="mt-2 justify-start" :disabled="isCreating" data-testid="information-add" @click="add">
      + Add information
    </AppButton>

    <Transition name="dialog">
      <NoteDeleteConfirm v-if="asking" :note="asking" @cancel="asking = null" @confirm="confirmDelete" />
    </Transition>
  </nav>
</template>
