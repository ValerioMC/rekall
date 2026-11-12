<script setup lang="ts">
import { computed, ref } from 'vue'
import { storeToRefs } from 'pinia'
import NoteKindGlyph from '@/components/ui/NoteKindGlyph.vue'
import NoteDeleteConfirm from '@/features/notes/components/NoteDeleteConfirm.vue'
import { useExcerptReveal } from '@/features/notes/composables/useExcerptReveal'
import { useConsoleStore } from '@/stores/console.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { excerpt } from '@/common/format/excerpt'
import type { RekallDocument } from '@/features/notes/model/document'
import type { DocumentId } from '@/model/branded'

const store = useConsoleStore()
const { selectedTask, selectedDocId, taskDocuments, paneFocus } = storeToRefs(store)
const { run } = useAsyncAction()
const { run: runCreate, isRunning: isCreating } = useAsyncAction()
const { revealedId, reveal, hide } = useExcerptReveal()

const leaving = ref<DocumentId | null>(null)
const asking = ref<RekallDocument | null>(null)

const hasReferenceNote = computed(() =>
  taskDocuments.value.some((note) => note.contextMode === 'REFERENCE')
)

function isOpen(note: RekallDocument): boolean {
  return note.id === selectedDocId.value && paneFocus.value === 'note'
}

function isOnRequest(note: RekallDocument): boolean {
  return note.contextMode === 'REFERENCE'
}

function excerptOf(note: RekallDocument): string {
  return excerpt(note.bodyMarkdown)
}

async function add(): Promise<void> {
  const task = selectedTask.value
  if (!task || isCreating.value) return
  await runCreate(() => store.createNote(task.id), 'Information added')
}

async function confirmDelete(): Promise<void> {
  const note = asking.value
  if (!note || leaving.value !== null) return
  asking.value = null
  leaving.value = note.id
  try {
    await run(() => store.deleteNote(note.id), `Deleted ${note.title}`)
  } finally {
    leaving.value = null
  }
}

defineExpose({ add })
</script>

<template>
  <div data-testid="information-list">
    <div class="dossier-seam relative z-[1] flex items-center gap-2 pb-1.5 pl-[38px] pr-3 pt-2.5">
      <span
        class="dossier-node top-[9px]"
        :class="!taskDocuments.length && 'dossier-node-empty'"
        aria-hidden="true"
      >
        <svg class="size-[9px]" viewBox="0 0 12 12" fill="none">
          <path d="M3 1.5h6v9H3z" stroke="currentColor" stroke-width="1.4" stroke-linejoin="round" />
          <path d="M1.2 3.6v7.2" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
        </svg>
      </span>
      <span class="text-caption font-semibold tracking-[0.005em] text-text-muted">Information</span>
      <span v-if="taskDocuments.length" class="font-mono text-micro tabular-nums text-text-subtle">
        {{ taskDocuments.length }}
      </span>
      <span
        v-if="hasReferenceNote"
        class="ml-auto flex items-center gap-1.5 text-micro text-text-subtle"
        data-testid="dossier-reference-legend"
      >
        <span class="size-[7px] rounded-full border-[1.5px] border-text-subtle" aria-hidden="true" />
        on request
      </span>
    </div>

    <p
      v-if="!taskDocuments.length"
      class="relative z-[1] pb-3.5 pl-[38px] pr-3 text-small leading-relaxed text-text-subtle"
    >
      Nothing here yet. Press
      <kbd class="rounded-key border border-border px-1 font-mono text-micro">N</kbd>
      to add information.
    </p>

    <TransitionGroup
      tag="div"
      leave-active-class="transition duration-(--motion-quick) ease-in"
      leave-to-class="-translate-x-2 opacity-0"
    >
      <div
        v-for="note in taskDocuments"
        :key="note.id"
        class="group/card relative"
        :class="leaving === note.id && 'pointer-events-none opacity-60'"
        @mouseenter="reveal(note.id)"
        @mouseleave="hide"
        @focusin="reveal(note.id)"
        @focusout="hide"
      >
        <button
          data-testid="note-card"
          class="dossier-section h-[30px] py-0"
          :class="isOpen(note) && 'dossier-section-selected'"
          :aria-current="isOpen(note)"
          :aria-describedby="`note-excerpt-${note.id}`"
          @click="store.selectDocument(note.id)"
          @keydown.delete.prevent="asking = note"
        >
          <span class="dossier-leaf top-[11px]" :class="isOpen(note) && 'dossier-leaf-lit'" aria-hidden="true" />
          <span class="flex h-full items-center gap-2 pr-9">
            <NoteKindGlyph :kind="note.kind" class="text-text-subtle" />
            <span
              class="min-w-0 flex-1 truncate text-body font-medium transition-colors"
              :class="isOpen(note) ? 'text-text' : 'text-text/90'"
            >
              {{ note.title }}
            </span>
            <span v-if="isOnRequest(note)" class="sr-only">Loaded on request.</span>
          </span>
        </button>

        <span class="absolute right-3 top-0 z-[2] flex h-[30px] items-center" data-testid="note-card-end">
          <span
            class="size-[7px] rounded-full transition-opacity duration-(--motion-quick) group-hover/card:opacity-0 group-focus-within/card:opacity-0"
            :class="isOnRequest(note) ? 'border-[1.5px] border-text-subtle' : 'bg-text-subtle'"
            :title="isOnRequest(note) ? 'On request' : 'In full'"
            :data-load-mode="isOnRequest(note) ? 'on-request' : 'in-full'"
            data-testid="note-load-marker"
          />
          <button
            type="button"
            class="focus-ring absolute right-0 top-1/2 grid size-6 -translate-y-1/2 place-items-center rounded-control text-text-subtle opacity-0 transition-[opacity,color,background-color] duration-(--motion-quick) hover:bg-danger-soft hover:text-danger focus-visible:opacity-100 group-hover/card:opacity-100 group-focus-within/card:opacity-100"
            :title="`Delete ${note.title}`"
            :aria-label="`Delete ${note.title}`"
            data-testid="note-card-delete"
            @click.stop="asking = note"
          >
            <svg class="size-3" viewBox="0 0 12 12" fill="none" aria-hidden="true">
              <path d="M3 3l6 6M9 3l-6 6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
            </svg>
          </button>
        </span>

        <div
          v-if="revealedId === note.id && excerptOf(note)"
          :id="`note-excerpt-${note.id}`"
          role="tooltip"
          class="pointer-events-none absolute left-[38px] right-3 top-full z-[3] rounded-control border border-border-strong bg-surface-raised px-2.5 py-1.5 text-caption leading-relaxed text-text-muted shadow-lift"
          data-testid="note-excerpt"
        >
          <span class="line-clamp-2 block">{{ excerptOf(note) }}</span>
        </div>
      </div>
    </TransitionGroup>

    <button
      type="button"
      class="add-row focus-ring relative z-[1] mb-1 ml-[38px] mr-3 mt-0.5 w-[calc(100%-50px)]"
      :disabled="isCreating"
      data-testid="information-add"
      @click="add"
    >
      <span class="text-accent" aria-hidden="true">+</span>
      <span class="min-w-0 flex-1 truncate">Add information</span>
      <kbd class="shrink-0 rounded-key border border-border px-1 font-mono text-micro text-text-subtle">N</kbd>
    </button>

    <Transition name="dialog">
      <NoteDeleteConfirm v-if="asking" :note="asking" @cancel="asking = null" @confirm="confirmDelete" />
    </Transition>
  </div>
</template>
