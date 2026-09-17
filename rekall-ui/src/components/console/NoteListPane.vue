<script setup lang="ts">
import { computed, ref } from 'vue'
import { storeToRefs } from 'pinia'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import DescriptionCard from '@/components/console/DescriptionCard.vue'
import TimeLogDialog from '@/components/console/TimeLogDialog.vue'
import StepsCard from '@/components/console/StepsCard.vue'
import TimerCard from '@/components/console/TimerCard.vue'
import WrapupCard from '@/components/console/WrapupCard.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useToastStore } from '@/stores/toast.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { excerpt } from '@/common/format/excerpt'
import type { RekallDocument } from '@/model/catalog'
import type { DocumentId } from '@/model/branded'

const store = useConsoleStore()
const {
  selectedTask,
  selectedDocId,
  taskDocuments,
  paneFocus,
  selectedTaskSteps,
  selectedWrapup,
  wrapupIsBehind,
  wrapupMissesSteps,
  selectedTaskEntries,
  runningEntries,
  isLoading
} = storeToRefs(store)
const { run } = useAsyncAction()
const toast = useToastStore()

const showTimeLog = ref(false)
/** The note whose removal is in flight, so a second click on it does nothing. */
const leaving = ref<DocumentId | null>(null)
/** The only-here note whose removal is waiting on the delete confirm, or null. */
const confirmingDelete = ref<RekallDocument | null>(null)
const isRunningHere = computed(() =>
  runningEntries.value.some((entry) => entry.taskId === selectedTask.value?.id)
)

async function startTimer(): Promise<void> {
  if (!selectedTask.value) return
  await run(() => store.startTimer(selectedTask.value!.id))
}

async function pauseTimer(): Promise<void> {
  if (!selectedTask.value) return
  await run(() => store.pauseTimer(selectedTask.value!.id))
}

/** A note is on at least one task, so taking one off the only task it is on is deleting it. */
function onlyHere(note: RekallDocument): boolean {
  return note.tasks.length === 1
}

/**
 * The card's one control. A note that lives elsewhere too comes off this task at once; a note
 * that is only here has nowhere to go, so the same control asks to delete it instead.
 */
function takeOff(note: RekallDocument): void {
  if (leaving.value !== null) return
  if (onlyHere(note)) {
    confirmingDelete.value = note
    return
  }
  void detach(note)
}

/**
 * Takes the note off the task in view without asking: the toast that follows carries an Undo,
 * which is quicker than a confirm and just as safe. Undo puts the note back and, if it was the
 * one in the editor, opens it again.
 */
async function detach(note: RekallDocument): Promise<void> {
  const task = selectedTask.value
  if (!task || leaving.value !== null) return
  const wasOpen = selectedDocId.value === note.id
  leaving.value = note.id
  try {
    const removed = await run(async () => {
      await store.detachNoteFromTask(note.id, task.id)
      return true
    })
    if (!removed) return
  } finally {
    leaving.value = null
  }
  toast.notify(`${note.title} is off this task.`, {
    label: 'Undo',
    run: () =>
      void run(async () => {
        await store.attachNoteToTask(note.id, task.id)
        if (wasOpen) store.selectDocument(note.id)
      })
  })
}

/** Deleting is not undone by a toast: the confirm is the one gate, and it names the blast. */
async function confirmDelete(): Promise<void> {
  const note = confirmingDelete.value
  confirmingDelete.value = null
  if (!note || leaving.value !== null) return
  leaving.value = note.id
  try {
    await run(() => store.deleteNote(note.id), `Deleted ${note.title}.`)
  } finally {
    leaving.value = null
  }
}
</script>

<template>
  <section
    class="flex min-h-0 w-(--spacing-notelist) shrink-0 flex-col border-r border-border bg-surface"
    aria-label="Notes on the selected task"
  >
    <header class="flex h-(--spacing-header) shrink-0 items-center gap-2 border-b border-border px-3.5">
      <span class="min-w-0 flex-1">
        <span class="block truncate text-[13.5px] font-semibold text-text">
          {{ selectedTask?.title ?? 'Notes' }}
        </span>
        <span
          v-if="selectedTask"
          class="block truncate font-mono text-[10.5px] text-anchor/80"
          :title="selectedTask.anchor"
        >
          {{ selectedTask.anchor }}
        </span>
      </span>
      <span v-if="taskDocuments.length" class="font-mono text-[11px] text-text-subtle">
        {{ taskDocuments.length }}
      </span>
    </header>

    <div class="min-h-0 flex-1 overflow-y-auto p-2">
      <div v-if="isLoading" class="flex flex-col gap-2" aria-hidden="true">
        <div class="skeleton h-14 rounded-[var(--radius-control)]" />
        <div v-for="row in 3" :key="row" class="skeleton h-16 rounded-[var(--radius-control)]" />
      </div>

      <p v-else-if="!selectedTask" class="px-2 py-2 text-[12.5px] leading-relaxed text-text-subtle">
        Pick a task to see its notes.
      </p>

      <template v-else>
        <TimerCard
          :entries="selectedTaskEntries"
          :is-running="isRunningHere"
          @start="startTimer"
          @pause="pauseTimer"
          @open-log="showTimeLog = true"
        />
        <DescriptionCard
          :description="selectedTask.description"
          :selected="paneFocus === 'description'"
          :review-state="
            selectedTask.reviewActive && selectedTask.stepCount === 0
              ? selectedTask.reviewState
              : null
          "
          @open="store.openDescription()"
        />
        <StepsCard
          :steps="selectedTaskSteps"
          :selected="paneFocus === 'steps'"
          @open="store.openSteps()"
        />
        <WrapupCard
          :wrapup="selectedWrapup"
          :selected="paneFocus === 'wrapup'"
          :behind="wrapupIsBehind"
          :misses-steps="wrapupMissesSteps"
          @open="store.openWrapup()"
        />

        <p
          v-if="!taskDocuments.length"
          class="px-2 py-2 text-[12.5px] leading-relaxed text-text-subtle"
        >
          No note on this task yet. Press
          <kbd class="rounded border border-border px-1 font-mono text-[10px]">N</kbd>
          to write the first one.
        </p>
      </template>

      <TransitionGroup
        tag="div"
        leave-active-class="transition duration-150 ease-in"
        leave-to-class="-translate-x-2 opacity-0"
      >
        <div
          v-for="document in taskDocuments"
          :key="document.id"
          class="group/card relative mb-1"
          :class="leaving === document.id && 'pointer-events-none opacity-60'"
        >
          <button
            data-testid="note-card"
            class="focus-ring block w-full rounded-[var(--radius-control)] border p-2.5 text-left transition-all"
            :class="
              document.id === selectedDocId && paneFocus === 'note'
                ? 'border-accent bg-surface-raised shadow-lift'
                : 'border-transparent hover:border-border-strong hover:bg-surface-raised'
            "
            :aria-current="document.id === selectedDocId && paneFocus === 'note'"
            @click="store.selectDocument(document.id)"
            @keydown.delete.prevent="takeOff(document)"
          >
            <span class="flex items-center gap-2 pr-8">
              <span class="min-w-0 flex-1 truncate text-[13px] font-medium text-text">
                {{ document.title }}
              </span>
              <span
                class="shrink-0 rounded border border-border bg-surface-raised px-1.5 py-px font-mono text-[9.5px] text-text-muted"
              >
                {{ document.kind }}
              </span>
            </span>
            <span class="mt-1 line-clamp-2 block text-[11.5px] leading-relaxed text-text-subtle">
              {{ excerpt(document.bodyMarkdown) }}
            </span>
          </button>

          <!--
            The card's right end is the note's membership. At rest it says where else the note
            lives; under the pointer or the keyboard it is the one way off this task. When this
            is the only task the note is on, that way off is deleting the note, and the control
            is a bin rather than a cross so the card says so before the confirm does.
          -->
          <span class="absolute right-2.5 top-2.5 flex h-[18px] items-center" data-testid="note-card-membership">
            <span
              v-if="document.tasks.length > 1"
              class="anchor-chip px-1.5 py-px text-[9.5px] transition-opacity duration-100 group-hover/card:opacity-0 group-focus-within/card:opacity-0"
              :title="`On ${document.tasks.length} tasks`"
              data-testid="note-card-elsewhere"
            >
              &#8942; {{ document.tasks.length }}
            </span>
            <button
              type="button"
              class="focus-ring absolute right-0 top-1/2 grid size-6 -translate-y-1/2 place-items-center rounded-md text-text-subtle opacity-0 transition-[opacity,color,background-color] duration-100 hover:bg-danger-soft hover:text-danger focus-visible:opacity-100 group-hover/card:opacity-100 group-focus-within/card:opacity-100"
              :title="
                onlyHere(document)
                  ? `This is the only task ${document.title} is on: taking it off deletes the note`
                  : `Take ${document.title} off this task`
              "
              :aria-label="
                onlyHere(document)
                  ? `Delete ${document.title}, this is its only task`
                  : `Take ${document.title} off this task`
              "
              :data-testid="onlyHere(document) ? 'note-card-delete' : 'note-card-off'"
              @click.stop="takeOff(document)"
            >
              <svg v-if="onlyHere(document)" class="size-3" viewBox="0 0 12 12" fill="none" aria-hidden="true">
                <path d="M2.5 3.5h7M4.5 3.5V2.5h3v1M3.5 3.5l.5 6h4l.5-6" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round" />
              </svg>
              <svg v-else class="size-3" viewBox="0 0 12 12" fill="none" aria-hidden="true">
                <path d="M3 3l6 6M9 3l-6 6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
              </svg>
            </button>
          </span>
        </div>
      </TransitionGroup>
    </div>

    <Transition name="dialog">
      <TimeLogDialog
        v-if="showTimeLog && selectedTask"
        :task="selectedTask"
        :entries="selectedTaskEntries"
        @close="showTimeLog = false"
      />
    </Transition>

    <Transition name="dialog">
      <AppConfirm
        v-if="confirmingDelete"
        :title="`Delete ${confirmingDelete.title}?`"
        body="This is the only task the note is on. Taking it off deletes the note and its content. Put it on another task first to keep it."
        blast="deletes the note · not recoverable"
        confirm-label="Delete note"
        @cancel="confirmingDelete = null"
        @confirm="confirmDelete"
      />
    </Transition>
  </section>
</template>
