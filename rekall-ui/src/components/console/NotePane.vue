<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import AppButton from '@/components/ui/AppButton.vue'
import AppMarkdownEditor from '@/components/ui/AppMarkdownEditor.vue'
import AppSelect from '@/components/ui/AppSelect.vue'
import AttachTasksDialog from '@/components/console/AttachTasksDialog.vue'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { DOCUMENT_KINDS } from '@/model/catalog'
import type { TaskId } from '@/model/branded'

const store = useConsoleStore()
const { selectedDocument, selectedTaskId, recentDocuments, isLoading } = storeToRefs(store)
const { run } = useAsyncAction()

const KIND_OPTIONS = DOCUMENT_KINDS.map((kind) => ({ value: kind, label: kind }))

/** The landing screen doubles as the only place the shortcuts are written down. */
const SHORTCUTS = [
  { keys: '⌘K', does: 'search by anchor, from anywhere' },
  { keys: 'T', does: 'new task' },
  { keys: 'E', does: 'edit the task in view' },
  { keys: 'N', does: 'new note on it' },
  { keys: 'W', does: 'its wrapup: what it currently is' },
  { keys: 'B', does: 'switch between tasks and notes' },
  { keys: 'J K', does: 'walk the list' },
  { keys: '1-4', does: 'set the status' }
] as const

const mode = ref<'write' | 'read'>('write')
const isAttaching = ref(false)
const isConfirmingDelete = ref(false)

/** The local copy being typed into, flushed to the server on a pause rather than on a button. */
const draft = ref({ title: '', kind: 'notes', bodyMarkdown: '' })
let saveTimer: ReturnType<typeof setTimeout> | null = null

watch(
  selectedDocument,
  (document) => {
    if (saveTimer) clearTimeout(saveTimer)
    if (!document) return
    draft.value = {
      title: document.title,
      kind: document.kind,
      bodyMarkdown: document.bodyMarkdown
    }
  },
  { immediate: true }
)

/**
 * Autosave. Typing marks the note unsaved at once so the state is honest, and the write goes
 * out when you stop for a moment. A Save button on a notes application is a way to lose work.
 */
function scheduleSave(): void {
  const document = selectedDocument.value
  if (!document) return
  store.saveState = 'unsaved'
  if (saveTimer) clearTimeout(saveTimer)
  saveTimer = setTimeout(() => {
    void run(() => store.saveNote(document.id, { ...draft.value }))
  }, 700)
}

const anchor = computed(() => {
  const document = selectedDocument.value
  if (!document) return ''
  const onCurrent = document.tasks.find((task) => task.id === selectedTaskId.value)
  return (onCurrent ?? document.tasks[0])?.anchor ?? ''
})

/** The tasks this note serves besides the one in view. */
const alsoOn = computed(
  () => selectedDocument.value?.tasks.filter((task) => task.id !== selectedTaskId.value) ?? []
)

/** The copy is confirmed on the chip itself, where the eye already is. */
const justCopied = ref(false)

async function copyAnchor(): Promise<void> {
  await navigator.clipboard?.writeText(anchor.value)
  justCopied.value = true
  setTimeout(() => (justCopied.value = false), 1400)
}

async function detachFrom(taskId: TaskId): Promise<void> {
  const document = selectedDocument.value
  if (!document || document.tasks.length === 1) return
  await run(
    () =>
      store.saveNote(document.id, {
        taskIds: document.tasks.filter((task) => task.id !== taskId).map((task) => task.id)
      }),
    'Removed from that task.'
  )
}

async function confirmDelete(): Promise<void> {
  const document = selectedDocument.value
  if (!document) return
  await run(() => store.deleteNote(document.id), `Deleted ${document.title}`)
  isConfirmingDelete.value = false
}
</script>

<template>
  <section class="flex min-h-0 flex-1 flex-col bg-canvas" aria-label="Note">
    <!-- Nothing open: the landing state is what you were last writing, not an empty page. -->
    <div v-if="!selectedDocument" class="min-h-0 flex-1 overflow-y-auto">
      <div class="max-w-[640px] px-9 py-12">
        <p class="text-[10px] font-semibold uppercase tracking-[0.09em] text-text-subtle">
          Continue
        </p>
        <h2 class="mb-1.5 mt-1.5 text-[26px] font-semibold leading-tight tracking-[-0.025em] text-text">
          Pick up where you left off
        </h2>
        <p class="mb-6 text-[13px] leading-relaxed text-text-muted">
          Everything here is one anchor away from a Claude session.
        </p>

        <div v-if="isLoading" class="flex flex-col gap-1.5" aria-hidden="true">
          <div v-for="row in 3" :key="row" class="skeleton h-14 rounded-[var(--radius-control)]" />
        </div>

        <div v-else-if="recentDocuments.length" class="flex flex-col gap-1.5">
          <button
            v-for="document in recentDocuments"
            :key="document.id"
            data-testid="resume-row"
            class="focus-ring group flex w-full items-center gap-3 rounded-[var(--radius-control)] border border-border bg-surface px-3.5 py-3 text-left transition-all hover:-translate-y-px hover:border-border-strong hover:bg-surface-raised hover:shadow-lift"
            @click="store.selectDocument(document.id)"
          >
            <span class="min-w-0 flex-1">
              <span class="block truncate text-[13.5px] font-medium text-text">
                {{ document.title }}
              </span>
              <span class="mt-0.5 block truncate font-mono text-[11px] text-anchor">
                {{ document.tasks[0]?.anchor }}
              </span>
            </span>
            <span
              class="shrink-0 text-[13px] text-text-subtle transition-transform group-hover:translate-x-0.5 group-hover:text-accent"
              aria-hidden="true"
            >
              &#8594;
            </span>
          </button>
        </div>

        <p v-else class="text-[13px] leading-relaxed text-text-subtle">
          Nothing written yet. Create a task on the left, then press
          <kbd class="rounded border border-border px-1 font-mono text-[10px]">N</kbd> to write its
          first note.
        </p>

        <ul class="mt-8 flex flex-col gap-2 text-[12px]">
          <li v-for="hint in SHORTCUTS" :key="hint.keys" class="flex items-center gap-3">
            <kbd
              class="w-9 shrink-0 rounded border border-border bg-surface py-0.5 text-center font-mono text-[10px] text-text-muted"
            >
              {{ hint.keys }}
            </kbd>
            <span class="text-text-subtle">{{ hint.does }}</span>
          </li>
        </ul>
      </div>
    </div>

    <template v-else>
      <header class="flex shrink-0 items-start gap-3.5 border-b border-border px-5 py-3.5">
        <div class="min-w-0 flex-1">
          <input
            v-model="draft.title"
            data-testid="note-title"
            class="focus-ring w-full truncate rounded border-0 bg-transparent text-[19px] font-semibold tracking-[-0.015em] text-text outline-none"
            aria-label="Note title"
            @input="scheduleSave"
          />
          <button
            class="anchor-chip focus-ring mt-1.5 inline-flex items-center gap-2 px-2.5 py-1 text-[11.5px] transition-colors hover:border-anchor"
            :class="justCopied && 'flash'"
            data-testid="copy-anchor"
            @click="copyAnchor"
          >
            <span>{{ anchor }}</span>
            <span class="opacity-70">{{ justCopied ? 'copied' : 'copy' }}</span>
          </button>
        </div>

        <div class="flex shrink-0 items-center gap-2">
          <div class="w-[128px]">
            <AppSelect v-model="draft.kind" :options="KIND_OPTIONS" @change="scheduleSave" />
          </div>
          <AppButton size="sm" @click="isAttaching = true">Attach to task</AppButton>
          <AppButton variant="danger" size="sm" @click="isConfirmingDelete = true">Delete</AppButton>
        </div>
      </header>

      <!-- The many-to-many made visible, and reachable. -->
      <div class="flex shrink-0 flex-wrap items-center gap-2 border-b border-border bg-surface px-5 py-2.5">
        <span class="text-[10px] font-semibold uppercase tracking-[0.09em] text-text-subtle">
          On {{ selectedDocument.tasks.length }}
          task{{ selectedDocument.tasks.length === 1 ? '' : 's' }}
        </span>
        <span
          v-for="task in selectedDocument.tasks"
          :key="task.id"
          class="inline-flex items-center gap-1.5 rounded-full border py-0.5 pl-2.5 pr-1 font-mono text-[11px] transition-colors"
          :class="
            task.id === selectedTaskId
              ? 'border-accent bg-accent-soft text-accent'
              : 'border-border-strong bg-surface-raised text-text-muted'
          "
        >
          <button class="focus-ring rounded" :title="task.title" @click="store.selectTask(task.id)">
            {{ task.projectLabel }}/{{ task.label }}
          </button>
          <button
            v-if="selectedDocument.tasks.length > 1"
            class="focus-ring grid size-4 place-items-center rounded-full text-text-subtle transition-colors hover:bg-danger-soft hover:text-danger"
            :aria-label="`Remove this note from ${task.title}`"
            @click="detachFrom(task.id)"
          >
            &times;
          </button>
        </span>
        <span v-if="alsoOn.length" class="w-full text-[11.5px] text-text-subtle">
          Editing here changes what
          {{ alsoOn.length === 1 ? 'that other task' : 'those other tasks' }} load too.
        </span>
      </div>

      <div class="flex min-h-0 flex-1 flex-col">
        <div class="flex shrink-0 items-center gap-2 border-b border-border px-5 py-1.5">
          <div class="ml-auto flex gap-0.5 rounded-[7px] bg-surface p-0.5">
            <button
              v-for="option in (['write', 'read'] as const)"
              :key="option"
              class="focus-ring h-6 rounded-[5px] px-2.5 text-[11.5px] capitalize transition-colors"
              :class="mode === option ? 'bg-surface-raised text-text' : 'text-text-subtle hover:text-text'"
              :aria-pressed="mode === option"
              @click="mode = option"
            >
              {{ option }}
            </button>
          </div>
        </div>

        <div class="min-h-0 flex-1 overflow-y-auto p-4">
          <AppMarkdownEditor
            v-if="mode === 'write'"
            v-model="draft.bodyMarkdown"
            height="100%"
            @update:model-value="scheduleSave"
          />
          <AppMarkdownEditor v-else :model-value="draft.bodyMarkdown" readonly />
        </div>
      </div>
    </template>

    <AttachTasksDialog v-if="isAttaching" @close="isAttaching = false" />

    <AppConfirm
      v-if="isConfirmingDelete && selectedDocument"
      :title="`Delete ${selectedDocument.title}?`"
      :body="
        selectedDocument.tasks.length > 1
          ? 'This note is on more than one task. Deleting removes it everywhere.'
          : 'This removes the note and its content.'
      "
      :blast="`removes it from ${selectedDocument.tasks.length} task${selectedDocument.tasks.length === 1 ? '' : 's'} · not recoverable`"
      confirm-label="Delete note"
      @cancel="isConfirmingDelete = false"
      @confirm="confirmDelete"
    />
  </section>
</template>
