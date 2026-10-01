<script setup lang="ts">
import { computed, ref } from 'vue'
import { storeToRefs } from 'pinia'
import DescriptionCard from '@/components/console/DescriptionCard.vue'
import NoteAttachPicker from '@/components/console/NoteAttachPicker.vue'
import TimeLogDialog from '@/components/console/TimeLogDialog.vue'
import StepsCard from '@/components/console/StepsCard.vue'
import TimerCard from '@/components/console/TimerCard.vue'
import WrapupCard from '@/components/console/WrapupCard.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useToastStore } from '@/stores/toast.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { excerpt } from '@/common/format/excerpt'
import type { RekallDocument } from '@/model/catalog'
import { scopeAdmits } from '@/model/note-scope'
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
  documents,
  projects,
  isLoading
} = storeToRefs(store)
const { run } = useAsyncAction()
const toast = useToastStore()

const showTimeLog = ref(false)
const isAttaching = ref(false)
const attachButton = ref<HTMLElement | null>(null)
/** Notes this task's scope admits that are not on it yet: what the attach shortcut can offer. */
const attachableCount = computed(() => {
  const task = selectedTask.value
  const project = projects.value.find((candidate) => candidate.id === task?.projectId)
  if (!task || !project) return 0
  return documents.value.filter(
    (document) => scopeAdmits(document.scope, project) && !document.tasks.some((ref) => ref.id === task.id)
  ).length
})
/** The note whose removal is in flight, so a second click on it does nothing. */
const leaving = ref<DocumentId | null>(null)
const { run: runCreate, isRunning: creatingNote } = useAsyncAction()
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

/** The note in the main pane, which is the one its leaf lights for. */
function isOpen(note: RekallDocument): boolean {
  return note.id === selectedDocId.value && paneFocus.value === 'note'
}

/** Whether the rail's hollow leaf needs its one-line legend: only while one is on screen. */
const hasReferenceNote = computed(() =>
  taskDocuments.value.some((note) => note.contextMode === 'REFERENCE')
)

/** The card's one control: the note comes off this task and stays in its scope. */
function takeOff(note: RekallDocument): void {
  if (leaving.value !== null) return
  void detach(note)
}

/** The shortcut under the last card: a note in this task's project, already on this task. */
async function createHere(): Promise<void> {
  const task = selectedTask.value
  if (!task || creatingNote.value) return
  await runCreate(() => store.createNote({ scope: store.projectScopeOf(task.id), taskIds: [task.id] }), 'Note created')
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
  toast.notify(`${note.title} is off this task. It stays in its scope.`, {
    label: 'Undo',
    run: () =>
      void run(async () => {
        await store.attachNoteToTask(note.id, task.id)
        if (wasOpen) store.selectDocument(note.id)
      })
  })
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
        <div class="dossier" data-testid="dossier">
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
            class="dossier-seam"
            :steps="selectedTaskSteps"
            :selected="paneFocus === 'steps'"
            @open="store.openSteps()"
          />
          <WrapupCard
            class="dossier-seam"
            :wrapup="selectedWrapup"
            :selected="paneFocus === 'wrapup'"
            :behind="wrapupIsBehind"
            :misses-steps="wrapupMissesSteps"
            @open="store.openWrapup()"
          />

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
            <span class="text-[11.5px] font-semibold tracking-[0.005em] text-text-muted">Notes</span>
            <span
              v-if="taskDocuments.length"
              class="font-mono text-[10.5px] tabular-nums text-text-subtle"
            >
              {{ taskDocuments.length }}
            </span>
            <span
              v-if="hasReferenceNote"
              class="ml-auto flex items-center gap-1.5 text-[10.5px] text-text-subtle"
              data-testid="dossier-reference-legend"
            >
              <span
                class="size-[7px] rounded-full border-[1.5px] border-text-subtle"
                aria-hidden="true"
              />
              on request
            </span>
          </div>

          <p
            v-if="!taskDocuments.length"
            class="relative z-[1] pb-3.5 pl-[38px] pr-3 text-[12px] leading-relaxed text-text-subtle"
          >
            No note on this task yet. Press
            <kbd class="rounded border border-border px-1 font-mono text-[10px]">N</kbd>
            or the button below to write the first one.
          </p>

          <TransitionGroup
            tag="div"
            class="pb-1"
            leave-active-class="transition duration-150 ease-in"
            leave-to-class="-translate-x-2 opacity-0"
          >
            <div
              v-for="document in taskDocuments"
              :key="document.id"
              class="group/card relative"
              :class="leaving === document.id && 'pointer-events-none opacity-60'"
            >
              <button
                data-testid="note-card"
                class="dossier-section py-2"
                :class="isOpen(document) && 'dossier-section-selected'"
                :aria-current="isOpen(document)"
                @click="store.selectDocument(document.id)"
                @keydown.delete.prevent="takeOff(document)"
              >
                <span
                  class="dossier-leaf"
                  :class="
                    isOpen(document)
                      ? 'dossier-leaf-lit'
                      : document.contextMode === 'REFERENCE' && 'dossier-leaf-reference'
                  "
                  aria-hidden="true"
                />
                <span class="flex items-center gap-2 pr-8">
                  <span
                    class="min-w-0 flex-1 truncate text-[13px] font-medium transition-colors"
                    :class="isOpen(document) ? 'text-text' : 'text-text/90'"
                  >
                    {{ document.title }}
                  </span>
                  <span class="shrink-0 font-mono text-[9.5px] text-text-subtle">
                    {{ document.kind }}
                  </span>
                </span>
                <span class="mt-0.5 line-clamp-2 block text-[11.5px] leading-relaxed text-text-subtle">
                  <span v-if="document.contextMode === 'REFERENCE'" class="sr-only">Loaded on request. </span>
                  {{ excerpt(document.bodyMarkdown) }}
                </span>
              </button>

              <!--
                The card's right end is the note's membership. At rest it says where else the note
                sits; under the pointer or the keyboard it is the one way off this task.
              -->
              <span class="absolute right-2.5 top-2 z-[2] flex h-[18px] items-center" data-testid="note-card-membership">
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
                  :title="`Take ${document.title} off this task`"
                  :aria-label="`Take ${document.title} off this task`"
                  data-testid="note-card-off"
                  @click.stop="takeOff(document)"
                >
                  <svg class="size-3" viewBox="0 0 12 12" fill="none" aria-hidden="true">
                    <path d="M3 3l6 6M9 3l-6 6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
                  </svg>
                </button>
              </span>
            </div>
          </TransitionGroup>

          <button
            type="button"
            class="focus-ring relative z-[1] mb-2 ml-[38px] mr-3 mt-0.5 flex w-[calc(100%-50px)] items-center gap-2 rounded-[var(--radius-control)] border border-dashed border-border-strong px-2.5 py-1.5 text-left text-[12px] text-text-muted transition-colors hover:border-solid hover:border-accent hover:bg-accent-soft hover:text-text disabled:opacity-60"
            :disabled="creatingNote"
            :title="`A new note in ${selectedTask.projectTitle}, on this task`"
            data-testid="note-create-here"
            @click="createHere"
          >
            <span class="text-accent" aria-hidden="true">+</span>
            <span class="min-w-0 flex-1 truncate">New note</span>
            <span class="shrink-0 truncate text-[10.5px] text-text-subtle">in {{ selectedTask.projectTitle }}</span>
          </button>

          <button
            ref="attachButton"
            type="button"
            class="focus-ring relative z-[1] mb-2 ml-[38px] mr-3 flex w-[calc(100%-50px)] items-center gap-2 rounded-[var(--radius-control)] border border-dashed border-border px-2.5 py-1.5 text-left text-[12px] text-text-muted transition-colors hover:border-solid hover:border-accent hover:bg-accent-soft hover:text-text disabled:cursor-not-allowed disabled:opacity-50 disabled:hover:border-dashed disabled:hover:bg-transparent"
            :class="isAttaching && 'border-solid border-accent bg-accent-soft text-text'"
            :disabled="attachableCount === 0"
            :aria-expanded="isAttaching"
            aria-haspopup="dialog"
            :title="attachableCount === 0 ? 'Every note this task can carry is already on it' : 'Put a note that already exists on this task'"
            data-testid="note-attach-existing"
            @click="isAttaching = !isAttaching"
          >
            <svg class="size-3 shrink-0 text-accent" viewBox="0 0 12 12" fill="none" aria-hidden="true">
              <path d="M5 7l2-2M4.2 5.6L3 6.8a2 2 0 0 0 2.8 2.8L7 8.4M7.8 6.4L9 5.2A2 2 0 0 0 6.2 2.4L5 3.6" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" />
            </svg>
            <span class="min-w-0 flex-1 truncate">Add existing note</span>
            <span class="shrink-0 font-mono text-[10.5px] tabular-nums text-text-subtle" data-testid="note-attach-available">
              {{ attachableCount }}
            </span>
          </button>
        </div>
      </template>
    </div>

    <NoteAttachPicker
      v-if="isAttaching && selectedTask && attachButton"
      :task-id="selectedTask.id"
      :anchor="attachButton"
      @close="isAttaching = false"
    />

    <Transition name="dialog">
      <TimeLogDialog
        v-if="showTimeLog && selectedTask"
        :task="selectedTask"
        :entries="selectedTaskEntries"
        @close="showTimeLog = false"
      />
    </Transition>

  </section>
</template>
