<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import NoteAssignmentDialog from '@/components/console/NoteAssignmentDialog.vue'
import NotePlacementRow from '@/components/console/NotePlacementRow.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { identityHue } from '@/common/identity'
import { groupByProject } from '@/common/catalog/task-search'
import type { Task } from '@/model/catalog'
import type { TaskId } from '@/model/branded'

/**
 * The middle column while browsing notes: where the selected note is, and the way to put it
 * somewhere else. It replaces the task's own column there, so picking a note never drags a task
 * into view. The list is the note's tasks grouped by project; each row carries its own remove
 * control, and nothing else on the row takes the note anywhere. Putting the note on another task
 * goes through the same picker the task side uses, `NoteAssignmentDialog`, opened from the
 * button at the top: one place to learn, whichever side the note is reached from.
 * Removals go through `store.detachNoteFromTask`, which keeps the rule that a note is on at
 * least one task. On the last task that control deletes the note instead, through
 * `store.deleteNote`, after `AppConfirm` has said so.
 */
const store = useConsoleStore()
const { selectedDocument, tasks, companies } = storeToRefs(store)
const { run } = useAsyncAction()

interface PlacementRow {
  readonly task: Task
  /** True when this is the only task the note is on: taking it off is deleting the note. */
  readonly onlyHere: boolean
}

const busy = ref<TaskId | null>(null)
const showDone = ref(false)
const isAssigning = ref(false)
/** The last-task row whose removal is waiting on the confirm, or null. */
const confirmingDelete = ref<PlacementRow | null>(null)

const manyCompanies = computed(() => companies.value.length > 1)

const attachedIds = computed(
  () => new Set(selectedDocument.value?.tasks.map((ref) => ref.id) ?? [])
)

const attachedTasks = computed(() =>
  tasks.value.filter((task) => attachedIds.value.has(task.id))
)

const liveAttached = computed(() => attachedTasks.value.filter((task) => task.status !== 'DONE'))
const doneAttached = computed(() => attachedTasks.value.filter((task) => task.status === 'DONE'))

/** The tasks it is on; finished ones only when asked for, or when there is nothing else to show. */
const rows = computed<PlacementRow[]>(() => {
  const shown = showDone.value || !liveAttached.value.length ? attachedTasks.value : liveAttached.value
  return shown.map((task) => ({ task, onlyHere: attachedIds.value.size === 1 }))
})

const groups = computed(() => groupByProject(rows.value, (row) => row.task))

/** The flat order the groups render in, which is what a row's index refers to. */
const order = computed(() => groups.value.flatMap((group) => group.rows))

watch(selectedDocument, () => {
  showDone.value = false
  confirmingDelete.value = null
})

function orderIndex(taskId: TaskId): number {
  return order.value.findIndex((row) => row.task.id === taskId)
}

/**
 * The row's remove control. With other tasks to stay on, the note comes off this one at once.
 * On its last task there is nothing to fall back to, so the same control asks to delete the
 * note, and only the confirm does it.
 */
async function remove(row: PlacementRow): Promise<void> {
  if (busy.value !== null) return
  if (row.onlyHere) {
    confirmingDelete.value = row
    return
  }
  const document = selectedDocument.value
  if (!document) return
  busy.value = row.task.id
  try {
    await run(
      () => store.detachNoteFromTask(document.id, row.task.id),
      `Taken off ${row.task.projectLabel}/${row.task.label}.`
    )
  } finally {
    busy.value = null
  }
}

async function confirmDelete(): Promise<void> {
  const document = selectedDocument.value
  const row = confirmingDelete.value
  confirmingDelete.value = null
  if (!document || !row) return
  busy.value = row.task.id
  try {
    await run(() => store.deleteNote(document.id), `Deleted ${document.title}.`)
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
        <span class="block truncate text-[13.5px] font-semibold text-text">
          {{ selectedDocument?.title ?? 'Notes' }}
        </span>
        <span v-if="selectedDocument" class="block truncate text-[10.5px] text-text-subtle">
          On {{ attachedIds.size }} task{{ attachedIds.size === 1 ? '' : 's' }}
        </span>
      </span>
      <span
        v-if="selectedDocument"
        class="shrink-0 rounded border border-border bg-surface-raised px-1.5 py-px font-mono text-[9.5px] text-text-muted"
      >
        {{ selectedDocument.kind }}
      </span>
    </header>

    <div v-if="!selectedDocument" class="px-4 py-4 text-[12.5px] leading-relaxed text-text-subtle">
      <p>Pick a note to see which tasks it is on.</p>
      <p class="mt-2">
        Or press
        <kbd class="rounded border border-border px-1 font-mono text-[10px]">N</kbd>
        to start a new one.
      </p>
    </div>

    <template v-else>
      <div class="shrink-0 border-b border-border px-2.5 py-2">
        <button
          type="button"
          class="place-opener focus-ring group/opener flex h-9 w-full items-center gap-2.5 rounded-[var(--radius-control)] border border-dashed border-border-strong bg-canvas pl-2.5 pr-2 text-left text-[12.5px] text-text-muted transition-colors hover:border-solid hover:border-accent hover:bg-accent-soft hover:text-text"
          title="Choose the tasks this note is on"
          data-testid="note-placements-open"
          @click="isAssigning = true"
        >
          <svg
            class="size-4 shrink-0 text-text-subtle transition-colors group-hover/opener:text-accent"
            viewBox="0 0 16 16"
            fill="none"
            aria-hidden="true"
          >
            <rect x="1" y="2.5" width="3.4" height="11" rx="1" stroke="currentColor" stroke-width="1.2" />
            <rect x="6.3" y="2.5" width="3.4" height="11" rx="1" stroke="currentColor" stroke-width="1.2" />
            <rect x="11.6" y="2.5" width="3.4" height="11" rx="1" stroke="currentColor" stroke-width="1.2" />
            <rect x="12.5" y="5" width="1.6" height="2.4" rx="0.5" fill="currentColor" />
          </svg>
          <span class="min-w-0 flex-1 truncate">Put it on a task</span>
          <svg
            class="size-3 shrink-0 text-text-subtle transition-colors group-hover/opener:text-accent"
            viewBox="0 0 12 12"
            fill="none"
            aria-hidden="true"
          >
            <path d="M4.5 3l3 3-3 3" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
        </button>
      </div>

      <div class="min-h-0 flex-1 overflow-y-auto py-1.5" data-testid="note-placements-list">
        <div
          v-for="group in groups"
          :key="group.projectId"
          class="placement-group mx-2 mb-2"
          :style="{ '--identity-color': identityHue(group.projectId).base }"
        >
          <div class="flex items-baseline gap-1.5 px-2 pb-1 pt-1.5">
            <span class="truncate text-[11px] font-medium text-text-muted">{{ group.projectTitle }}</span>
            <span v-if="manyCompanies" class="truncate text-[10px] text-text-subtle">
              {{ group.companyName }}
            </span>
          </div>

          <NotePlacementRow
            v-for="row in group.rows"
            :key="row.task.id"
            :task="row.task"
            attached
            :locked="row.onlyHere"
            :busy="busy === row.task.id"
            :walk-index="orderIndex(row.task.id)"
            openable
            removable
            @remove="remove(row)"
            @open="openTask(row.task)"
          />
        </div>

        <button
          v-if="doneAttached.length && liveAttached.length"
          type="button"
          class="focus-ring mx-4 mt-1 inline-flex items-center gap-1.5 rounded-full border border-border-strong bg-surface-raised py-0.5 pl-2 pr-2.5 font-mono text-[11px] text-text-muted transition-colors hover:text-text"
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
            <path
              d="M9 6l6 6-6 6"
              stroke="currentColor"
              stroke-width="2.4"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          </svg>
          {{ doneAttached.length }} done
        </button>
      </div>

      <div class="shrink-0 border-t border-border bg-canvas px-3.5 py-2 text-[10.5px] text-text-subtle">
        <template v-if="attachedIds.size === 1">
          Its only task: taking it off deletes the note. Put it on another to keep it.
        </template>
        <template v-else>The cross on a row takes the note off that task.</template>
      </div>
    </template>

    <Transition name="dialog">
      <NoteAssignmentDialog v-if="isAssigning" @close="isAssigning = false" />
    </Transition>

    <Transition name="dialog">
      <AppConfirm
        v-if="confirmingDelete && selectedDocument"
        :title="`Delete ${selectedDocument.title}?`"
        :body="`${confirmingDelete.task.title} is the only task this note is on. Taking it off deletes the note and its content. Put it on another task first to keep it.`"
        blast="deletes the note · not recoverable"
        confirm-label="Delete note"
        @cancel="confirmingDelete = null"
        @confirm="confirmDelete"
      />
    </Transition>
  </section>
</template>

<style scoped>
.placement-group {
  border-left: 2px solid var(--identity-color, var(--color-border-strong));
  padding-left: 2px;
}

/* The opener shows a slice of the picker it opens: the three columns, one task ticked. */
.place-opener:focus-visible {
  border-style: solid;
}
</style>
