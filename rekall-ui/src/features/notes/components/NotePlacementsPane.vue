<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import NoteAssignmentDialog from '@/features/notes/components/NoteAssignmentDialog.vue'
import NotePlacementCard from '@/features/notes/components/NotePlacementCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import { useToastStore } from '@/stores/toast.store'
import { useConsoleStore } from '@/stores/console.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { groupByProject } from '@/features/catalog/lib/task-search'
import { scopeName } from '@/features/notes/model/note-scope'
import type { Task } from '@/features/catalog/model/catalog'
import type { TaskId } from '@/model/branded'

/**
 * The middle column while browsing notes: the tasks the selected note is on, one card per
 * project, so picking a note never drags a task into view. Detaching is one click with an Undo
 * toast, since the note stays in its scope. Placing it elsewhere opens `NoteAssignmentDialog`,
 * the task side's own picker.
 */
const store = useConsoleStore()
const { selectedDocument, tasks, companies, projects } = storeToRefs(store)
const { run } = useAsyncAction()
const toast = useToastStore()

const busy = ref<TaskId | null>(null)
const showDone = ref(false)
const isAssigning = ref(false)

const scopeLabel = computed(() =>
  selectedDocument.value ? scopeName(selectedDocument.value.scope, projects.value, companies.value) : ''
)

const manyCompanies = computed(() => companies.value.length > 1)

const attachedIds = computed(
  () => new Set(selectedDocument.value?.tasks.map((ref) => ref.id) ?? [])
)

/** The note's own title and kind are in the editor beside this column; here only where it is. */
const placementsTitle = computed(() => {
  if (!selectedDocument.value) return 'Placements'
  const count = attachedIds.value.size
  if (count === 0) return 'On no task'
  return `On ${count} task${count === 1 ? '' : 's'}`
})

const attachedTasks = computed(() =>
  tasks.value.filter((task) => attachedIds.value.has(task.id))
)

const liveAttached = computed(() => attachedTasks.value.filter((task) => task.status !== 'DONE'))
const doneAttached = computed(() => attachedTasks.value.filter((task) => task.status === 'DONE'))

/** The tasks it is on; finished ones only when asked for, or when there is nothing else to show. */
const shownTasks = computed(() =>
  showDone.value || !liveAttached.value.length ? attachedTasks.value : liveAttached.value
)

const groups = computed(() => groupByProject(shownTasks.value, (task) => task))

watch(selectedDocument, () => {
  showDone.value = false
})

/** The note comes off this task at once; the toast's Undo puts it back. */
async function detach(task: Task): Promise<void> {
  if (busy.value !== null) return
  const document = selectedDocument.value
  if (!document) return
  busy.value = task.id
  try {
    const detached = await run(() => store.detachNoteFromTask(document.id, task.id).then(() => true))
    if (detached) {
      toast.notify(`Detached from ${task.projectLabel}/${task.label}.`, {
        label: 'Undo',
        run: () => void run(() => store.attachNoteToTask(document.id, task.id))
      })
    }
  } finally {
    busy.value = null
  }
}

/** The one deliberate way out: the task opens on the Tasks side, with this note still in view. */
function openTask(task: Task): void {
  store.setNavMode('tasks')
  store.selectTask(task.id)
  if (selectedDocument.value) store.selectDocument(selectedDocument.value.id)
}
</script>

<template>
  <section
    class="flex min-h-0 w-(--spacing-notelist) shrink-0 flex-col border-r border-border bg-surface"
    aria-label="Tasks the selected note is on"
    data-testid="note-placements"
  >
    <header class="flex h-(--spacing-header) shrink-0 items-center gap-2 border-b border-border px-3.5">
      <span class="min-w-0 flex-1">
        <span class="block truncate text-body font-semibold text-text" data-testid="note-placements-count">
          {{ placementsTitle }}
        </span>
        <span v-if="selectedDocument" class="block truncate text-micro text-text-subtle">
          Lives in {{ scopeLabel }}
        </span>
      </span>
      <AppButton
        v-if="selectedDocument && attachedIds.size"
        variant="quiet"
        size="sm"
        title="Choose the tasks this note is on"
        data-testid="note-placements-open"
        @click="isAssigning = true"
      >
        <svg class="size-3" viewBox="0 0 12 12" fill="none" aria-hidden="true">
          <path d="M6 2v8M2 6h8" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
        </svg>
        Put it on a task
      </AppButton>
    </header>

    <div v-if="!selectedDocument" class="px-4 py-4 text-small leading-relaxed text-text-subtle">
      <p>Pick a note to see which tasks it is on.</p>
      <p class="mt-2">
        Or press
        <kbd class="rounded-key border border-border px-1 font-mono text-micro">N</kbd>
        to start a new one.
      </p>
    </div>

    <div v-else-if="attachedIds.size === 0" class="p-3">
      <div class="flex flex-col items-center rounded-card border border-dashed border-border-strong px-5 py-8 text-center">
        <span class="mb-3 grid size-10 place-items-center rounded-card bg-surface-raised text-text-subtle">
          <svg class="size-5" viewBox="0 0 16 16" fill="none" aria-hidden="true">
            <path
              d="M6.5 4.5l1-1a2.83 2.83 0 0 1 4 4l-1 1M9.5 11.5l-1 1a2.83 2.83 0 0 1-4-4l1-1M3 3l1.5 1.5M13 13l-1.5-1.5"
              stroke="currentColor"
              stroke-width="1.3"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          </svg>
        </span>
        <p class="text-body font-semibold text-text">On no task yet</p>
        <p class="mt-1 text-small leading-relaxed text-text-subtle">
          It waits in {{ scopeLabel }} until it is put on one. Sessions load it from a task.
        </p>
        <AppButton class="mt-4" variant="primary" size="sm" data-testid="note-placements-open" @click="isAssigning = true">
          Put it on a task
        </AppButton>
      </div>
    </div>

    <div v-else class="flex min-h-0 flex-1 flex-col gap-2.5 overflow-y-auto p-2.5" data-testid="note-placements-list">
      <NotePlacementCard
        v-for="group in groups"
        :key="group.projectId"
        :group="group"
        :show-company="manyCompanies"
        :busy-task-id="busy"
        @detach="detach"
        @open="openTask"
      />

      <button
        v-if="doneAttached.length && liveAttached.length"
        type="button"
        class="focus-ring inline-flex items-center gap-1.5 self-start rounded-full border border-border-strong bg-surface-raised py-0.5 pl-2 pr-2.5 font-mono text-caption text-text-muted transition-colors hover:text-text"
        :aria-expanded="showDone"
        data-testid="note-placements-done-toggle"
        @click="showDone = !showDone"
      >
        <svg
          class="size-2.5 transition-transform"
          :class="showDone && 'rotate-90'"
          viewBox="0 0 24 24"
          fill="none"
          aria-hidden="true"
        >
          <path d="M9 6l6 6-6 6" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
        {{ doneAttached.length }} done
      </button>
    </div>

    <Transition name="dialog">
      <NoteAssignmentDialog v-if="isAssigning" @close="isAssigning = false" />
    </Transition>
  </section>
</template>
