<script setup lang="ts">
import { computed, nextTick, onUnmounted, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import AppInput from '@/components/ui/AppInput.vue'
import AppMarkdownEditor from '@/components/ui/AppMarkdownEditor.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { identityHue } from '@/common/identity'
import { relativeTime } from '@/common/format/relative-time'
import { rkCommand } from '@/common/format/rk-command'
import type { TaskStep } from '@/model/catalog'
import type { TaskStepId } from '@/model/branded'

/**
 * What is left on the task in view, in the order it is meant to be worked.
 *
 * This is the pane the other three could not be. The description is the brief and grows as the
 * work is redefined; the wrapup is what the implementation became; a note is what you learned.
 * None of them says which parts are finished, and working that out by reading the brief against
 * the wrapup is slow by hand and a guess for a model. Here it is a node on a line.
 *
 * The line is the point of the layout. A checklist is not a set of boxes, it is an order, and
 * the row that matters is the first one still open: it carries the ring, it is expanded when
 * the pane opens, and ticking it moves the ring to the next. The rest of the list is context
 * for that one row, which is why the finished ones recede rather than disappear.
 *
 * A step is ticked here and nowhere else. Claude reads the checklist with every `/rk` and can
 * never close a box on it: the point of the list is that a person looked at the work and said
 * it was done, and a session marking its own homework would be worth nothing.
 */
const store = useConsoleStore()
const { selectedTask, selectedTaskSteps } = storeToRefs(store)
const { run } = useAsyncAction()

const hue = computed(() => identityHue(selectedTask.value?.projectId ?? ''))

const done = computed(() => selectedTaskSteps.value.filter((step) => step.done).length)

/** The first step still open: the work this task is actually on. */
const currentId = computed(
  () => selectedTaskSteps.value.find((step) => !step.done)?.id ?? null
)

const hideDone = ref(false)
const visibleSteps = computed(() =>
  hideDone.value ? selectedTaskSteps.value.filter((step) => !step.done) : selectedTaskSteps.value
)

// ------------------------------------------------------------------ adding

const newTitle = ref('')

/**
 * Adds the step and stays where it is, ready for the next one.
 *
 * A checklist is written in a burst, five items at a time, and a form that had to be reopened
 * between them is a form that gets three of the five.
 */
async function add(): Promise<void> {
  const title = newTitle.value.trim()
  if (!title || !selectedTask.value) return
  newTitle.value = ''
  await run(() => store.addStep(selectedTask.value!.id, title))
}

async function focusAdd(): Promise<void> {
  await nextTick()
  document.getElementById('new-step')?.focus()
}

// ------------------------------------------------------------------ the open row

const expandedId = ref<TaskStepId | null>(null)
const mode = ref<'write' | 'read'>('read')
const draftTitle = ref('')
const draftBody = ref('')
let saveTimer: ReturnType<typeof setTimeout> | null = null

/** Writes what is pending now, for the step it was typed on rather than the one now open. */
function flush(): void {
  if (!saveTimer) return
  clearTimeout(saveTimer)
  saveTimer = null
  writeDraft()
}

function writeDraft(): void {
  const id = expandedId.value
  const title = draftTitle.value.trim()
  if (!id || !title) return
  const step = selectedTaskSteps.value.find((candidate) => candidate.id === id)
  if (!step || (step.title === title && (step.bodyMarkdown ?? '') === draftBody.value)) return
  void run(() => store.saveStep(id, { title, bodyMarkdown: draftBody.value }))
}

/** Autosave on the rhythm the notes and the description use: unsaved as you type, written when
 *  you pause. */
function scheduleSave(): void {
  store.saveState = 'unsaved'
  if (saveTimer) clearTimeout(saveTimer)
  saveTimer = setTimeout(() => {
    saveTimer = null
    writeDraft()
  }, 700)
}

function open(step: TaskStep): void {
  expandedId.value = step.id
  draftTitle.value = step.title
  draftBody.value = step.bodyMarkdown ?? ''
  // Reading is the default on a step that says something: the detail is read every time the
  // work is picked up and rewritten far less often.
  mode.value = step.bodyMarkdown?.trim() ? 'read' : 'write'
}

function toggleExpanded(step: TaskStep): void {
  flush()
  if (expandedId.value === step.id) {
    expandedId.value = null
    return
  }
  open(step)
}

/**
 * The pane opens on the work, not on the list.
 *
 * Landing here means asking what to do next, and the answer is the first open step with its
 * detail already on screen. Opening on a list of titles would make the first click the same
 * one every time.
 */
watch(
  () => selectedTask.value?.id ?? null,
  () => {
    flush()
    hideDone.value = false
    expandedId.value = null
    const next = selectedTaskSteps.value.find((step) => !step.done)
    if (next) open(next)
  },
  { immediate: true }
)

/**
 * Ticking the row that was open moves to the next one.
 *
 * Finishing a step and being left staring at what you just finished is a click wasted every
 * time. The ring, the expansion and the detail all move together, so the pane always shows the
 * work rather than the record of it.
 */
async function toggle(step: TaskStep): Promise<void> {
  const wasOpenHere = expandedId.value === step.id
  flush()
  await run(() => store.toggleStep(step.id))
  if (!wasOpenHere) return

  const next = selectedTaskSteps.value.find((candidate) => !candidate.done)
  if (next && next.id !== step.id) open(next)
  else expandedId.value = null
}

async function move(step: TaskStep, by: number): Promise<void> {
  flush()
  await run(() => store.moveStep(step.id, step.position + by))
}

/**
 * Removing a step, and saying what it takes with it.
 *
 * The detail written under a step is the only copy of it, so the confirmation names it instead
 * of asking whether you are sure. A step with nothing under it is still confirmed: the row is
 * one click from the checkbox next to it.
 */
const deleting = ref<TaskStep | null>(null)

const deletingBlast = computed(() =>
  deleting.value?.bodyMarkdown?.trim()
    ? 'the step and the detail written under it · not recoverable'
    : 'one step · not recoverable'
)

async function remove(): Promise<void> {
  const step = deleting.value
  deleting.value = null
  if (!step) return
  if (expandedId.value === step.id) expandedId.value = null
  await run(() => store.removeStep(step.id))
}

// ------------------------------------------------------------------ the rail

/**
 * The connector between two nodes, trimmed at the ends of the list.
 *
 * A line running past the first and last node reads as a list that continues somewhere off
 * screen, which is the one thing a checklist must not suggest.
 */
function railStyle(index: number): Record<string, string> {
  const isFirst = index === 0
  const isLast = index === visibleSteps.value.length - 1
  if (isFirst && isLast) return { display: 'none' }
  if (isFirst) return { top: '18px', bottom: '0' }
  if (isLast) return { top: '0', height: '18px' }
  return { top: '0', bottom: '0' }
}

const copied = ref(false)

async function copyAnchor(): Promise<void> {
  if (!selectedTask.value) return
  await navigator.clipboard?.writeText(rkCommand(selectedTask.value.anchor))
  copied.value = true
  setTimeout(() => (copied.value = false), 1400)
}

onUnmounted(flush)
</script>

<template>
  <section class="flex min-h-0 w-full min-w-0 flex-1 flex-col bg-canvas" aria-label="Steps">
    <p v-if="!selectedTask" class="px-9 py-12 text-[13px] text-text-muted">
      Pick a task to see what is left on it.
    </p>

    <template v-else>
      <div class="relative shrink-0 overflow-hidden border-b border-border">
        <div
          class="texture-grid pointer-events-none absolute inset-0 opacity-60"
          :style="{ '--texture-tint': hue.base }"
          aria-hidden="true"
        />
        <div
          class="pointer-events-none absolute inset-x-0 top-0 h-px"
          :style="{ background: hue.line }"
          aria-hidden="true"
        />

        <header class="relative flex items-start gap-4 px-5 py-3.5">
          <div class="min-w-0 flex-1">
            <p class="text-[10px] font-semibold uppercase tracking-[0.09em] text-text-subtle">
              Steps
            </p>
            <h2 class="truncate text-[19px] font-semibold tracking-[-0.015em] text-text">
              {{ selectedTask.title }}
            </h2>
            <button
              class="anchor-chip focus-ring mt-1.5 inline-flex items-center gap-2 px-2.5 py-1 text-[11.5px] transition-colors hover:border-anchor"
              :class="copied && 'flash'"
              data-testid="copy-steps-anchor"
              @click="copyAnchor"
            >
              <span class="opacity-60">/rk</span>
              <span>{{ selectedTask.anchor }}</span>
              <span class="opacity-70">{{ copied ? 'copied' : 'copy' }}</span>
            </button>
          </div>

          <!-- The count, and the shape of it. The number says how much; the segments say where
               the gaps are, which is what tells you whether the work is nearly done or merely
               started at both ends. -->
          <div v-if="selectedTaskSteps.length" class="shrink-0 text-right">
            <p
              class="texture-scan inline-block rounded-[6px] px-1.5 py-0.5 font-mono text-[22px] font-semibold leading-none tabular-nums"
              data-testid="steps-count"
            >
              <span :class="done === selectedTaskSteps.length ? 'text-safe' : 'text-accent'">
                {{ done }}
              </span>
              <span class="text-text-subtle">/{{ selectedTaskSteps.length }}</span>
            </p>
            <span class="mt-2 flex h-[5px] w-[164px] gap-[3px]" aria-hidden="true">
              <span
                v-for="step in selectedTaskSteps"
                :key="step.id"
                class="h-full flex-1 rounded-full transition-colors duration-300"
                :class="
                  step.done
                    ? 'bg-accent'
                    : step.id === currentId
                      ? 'bg-accent/40'
                      : 'bg-border-strong'
                "
              />
            </span>
            <button
              v-if="done > 0"
              class="focus-ring mt-2 text-[11px] text-text-subtle transition-colors hover:text-text"
              :aria-pressed="hideDone"
              data-testid="toggle-hide-done"
              @click="hideDone = !hideDone"
            >
              {{ hideDone ? 'Show done' : 'Hide done' }}
            </button>
          </div>
        </header>
      </div>

      <div
        class="flex shrink-0 flex-wrap items-center gap-x-3 gap-y-1.5 border-b border-border bg-surface px-5 py-2.5"
      >
        <span class="text-[11.5px] text-text-muted">
          What is left, in order. Every
          <code class="text-anchor/80">/rk {{ selectedTask.anchor }}</code>
          hands Claude the open ones in full and the done ones by name alone.
        </span>
      </div>

      <div class="min-h-0 min-w-0 flex-1 overflow-y-auto px-5 py-4">
        <!-- Nothing yet. The shape of what would be here is drawn rather than described: three
             ghost nodes on the same line the real ones will sit on. -->
        <div v-if="!selectedTaskSteps.length" class="max-w-[560px] py-4">
          <div class="relative mb-7" aria-hidden="true">
            <span
              class="absolute bottom-[18px] left-[11px] top-[18px] w-px -translate-x-1/2 bg-border-strong"
            />
            <!-- The nodes are placed against this block rather than against each row, for the
                 same reason the real ones are: padding does not move an absolute origin, so a
                 node inside a padded row lands where the text starts instead of on the line. -->
            <div v-for="ghost in 3" :key="ghost" class="relative flex h-9 items-center pl-9">
              <span
                class="absolute left-0 top-1/2 size-[22px] -translate-y-1/2 rounded-full border border-dashed border-border-strong bg-canvas"
              />
              <span
                class="h-[7px] rounded-full bg-border"
                :style="{ width: `${[62, 45, 54][ghost - 1]}%` }"
              />
            </div>
          </div>

          <h3 class="mb-1.5 text-[21px] font-semibold leading-tight tracking-[-0.02em] text-text">
            Nothing says what is left
          </h3>
          <p class="text-[13px] leading-relaxed text-text-muted">
            The description says what the task is and the wrapup says what it became. Neither says
            which parts are finished, and reading that out of the two is guesswork. Break the work
            into steps, write the detail of each in markdown, and tick them as you review them: the
            next session opens on the ones still open, in full.
          </p>
          <button
            class="focus-ring mt-5 rounded-[var(--radius-control)] border border-accent bg-accent-soft px-3.5 py-2 text-[12.5px] font-medium text-accent transition-colors hover:bg-accent hover:text-accent-ink"
            data-testid="write-first-step"
            @click="focusAdd"
          >
            Write the first step
          </button>
        </div>

        <p v-else-if="!visibleSteps.length" class="py-6 text-[12.5px] text-text-subtle">
          Every step is done. Show them again to correct one.
        </p>

        <ol v-else class="relative min-w-0">
          <li
            v-for="(step, index) in visibleSteps"
            :key="step.id"
            class="group/step relative min-w-0 pb-1.5 pl-9"
            data-testid="step-row"
          >
            <span
              class="absolute left-[11px] w-px -translate-x-1/2 bg-border-strong"
              :style="railStyle(index)"
              aria-hidden="true"
            />

            <!-- The node, and the whole of what the console writes about a step's state. The
                 canvas fill is what makes the line pass behind it rather than through it. -->
            <button
              class="focus-ring absolute left-0 top-[7px] z-10 grid size-[22px] place-items-center rounded-full border transition-all duration-200"
              :class="
                step.done
                  ? 'border-accent bg-accent text-accent-ink'
                  : step.id === currentId
                    ? 'border-accent bg-canvas text-accent shadow-[0_0_0_4px_var(--color-accent-soft)]'
                    : 'border-border-strong bg-canvas text-transparent hover:border-accent'
              "
              role="checkbox"
              :aria-checked="step.done"
              :aria-label="step.done ? `Reopen ${step.title}` : `Mark ${step.title} done`"
              data-testid="step-checkbox"
              @click="toggle(step)"
            >
              <svg
                v-if="step.done"
                class="size-3"
                viewBox="0 0 24 24"
                fill="none"
                aria-hidden="true"
              >
                <path
                  d="M5 12.5l4.5 4.5L19 7.5"
                  stroke="currentColor"
                  stroke-width="2.8"
                  stroke-linecap="round"
                />
              </svg>
              <span
                v-else-if="step.id === currentId"
                class="size-[7px] rounded-full bg-accent"
                aria-hidden="true"
              />
            </button>

            <div
              class="min-w-0 rounded-[var(--radius-card)] border px-3 py-2 transition-all"
              :class="[
                step.id === currentId
                  ? 'border-accent/30 bg-surface-raised'
                  : 'border-transparent group-hover/step:border-border',
                step.done && 'opacity-60 hover:opacity-100'
              ]"
            >
              <div class="flex items-start gap-2">
                <button
                  class="focus-ring min-w-0 flex-1 rounded text-left"
                  :aria-expanded="expandedId === step.id"
                  data-testid="step-title"
                  @click="toggleExpanded(step)"
                >
                  <span
                    class="block text-[13.5px] leading-snug transition-colors"
                    :class="
                      step.done
                        ? 'text-text-subtle line-through decoration-text-subtle/50'
                        : 'text-text'
                    "
                  >
                    {{ step.title }}
                  </span>
                  <span class="mt-1 flex flex-wrap items-center gap-2 text-[10.5px]">
                    <span
                      v-if="step.id === currentId"
                      class="rounded-full bg-accent-soft px-1.5 py-px font-semibold uppercase tracking-[0.08em] text-accent"
                    >
                      Next
                    </span>
                    <span v-if="step.done && step.doneAt" class="text-text-subtle">
                      done {{ relativeTime(step.doneAt) }}
                    </span>
                    <span
                      v-if="step.bodyMarkdown?.trim()"
                      class="flex items-center gap-1 text-text-subtle"
                      data-testid="step-has-detail"
                    >
                      <svg class="size-2.5" viewBox="0 0 12 12" fill="none" aria-hidden="true">
                        <path
                          d="M1.6 3h8.8M1.6 6h8.8M1.6 9h5.4"
                          stroke="currentColor"
                          stroke-width="1.2"
                          stroke-linecap="round"
                        />
                      </svg>
                      detail
                    </span>
                    <span v-else-if="!step.done" class="text-text-subtle">no detail yet</span>
                  </span>
                </button>

                <div
                  class="flex shrink-0 items-center gap-0.5 opacity-0 transition-opacity focus-within:opacity-100 group-hover/step:opacity-100"
                >
                  <button
                    class="focus-ring grid size-6 place-items-center rounded text-text-subtle transition-colors hover:bg-surface-hover hover:text-text disabled:opacity-30"
                    :disabled="step.position === 0"
                    :aria-label="`Move ${step.title} up`"
                    data-testid="step-up"
                    @click="move(step, -1)"
                  >
                    <svg class="size-3" viewBox="0 0 12 12" fill="none" aria-hidden="true">
                      <path
                        d="M6 9.5v-7M2.8 5.4 6 2.2l3.2 3.2"
                        stroke="currentColor"
                        stroke-width="1.3"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                      />
                    </svg>
                  </button>
                  <button
                    class="focus-ring grid size-6 place-items-center rounded text-text-subtle transition-colors hover:bg-surface-hover hover:text-text disabled:opacity-30"
                    :disabled="step.position === selectedTaskSteps.length - 1"
                    :aria-label="`Move ${step.title} down`"
                    data-testid="step-down"
                    @click="move(step, 1)"
                  >
                    <svg class="size-3" viewBox="0 0 12 12" fill="none" aria-hidden="true">
                      <path
                        d="M6 2.5v7M2.8 6.6 6 9.8l3.2-3.2"
                        stroke="currentColor"
                        stroke-width="1.3"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                      />
                    </svg>
                  </button>
                  <button
                    class="focus-ring grid size-6 place-items-center rounded text-text-subtle transition-colors hover:bg-surface-hover hover:text-danger"
                    :aria-label="`Delete ${step.title}`"
                    data-testid="step-delete"
                    @click="deleting = step"
                  >
                    <svg class="size-3" viewBox="0 0 12 12" fill="none" aria-hidden="true">
                      <path
                        d="M2.6 2.6l6.8 6.8M9.4 2.6 2.6 9.4"
                        stroke="currentColor"
                        stroke-width="1.3"
                        stroke-linecap="round"
                      />
                    </svg>
                  </button>
                </div>
              </div>

              <!-- The detail of this one piece, in the same markdown surface the description and
                   the notes are written on. It is what Claude receives while the step is open,
                   so it is written and read as a document rather than as a field. -->
              <div v-if="expandedId === step.id" class="mt-2.5" data-testid="step-detail">
                <div class="mb-2 flex items-center gap-2">
                  <span
                    class="text-[9.5px] font-semibold uppercase tracking-[0.09em] text-text-subtle"
                  >
                    Detail
                  </span>
                  <span class="h-px flex-1 bg-border" aria-hidden="true" />
                  <div class="flex gap-0.5 rounded-[7px] bg-surface p-0.5">
                    <button
                      v-for="option in (['write', 'read'] as const)"
                      :key="option"
                      class="focus-ring h-6 rounded-[5px] px-2.5 text-[11.5px] capitalize transition-colors"
                      :class="
                        mode === option
                          ? 'bg-surface-hover text-text'
                          : 'text-text-subtle hover:text-text'
                      "
                      :aria-pressed="mode === option"
                      :data-testid="`step-detail-${option}`"
                      @click="mode = option"
                    >
                      {{ option }}
                    </button>
                  </div>
                </div>

                <template v-if="mode === 'write'">
                  <AppInput
                    v-model="draftTitle"
                    :aria-label="`Title of ${step.title}`"
                    data-testid="step-title-field"
                    @update:model-value="scheduleSave"
                  />
                  <div class="mt-2 h-[300px] min-w-0 overflow-hidden">
                    <AppMarkdownEditor
                      v-model="draftBody"
                      height="100%"
                      :show-preview="false"
                      placeholder="What this step has to do. Markdown, and Claude gets it while the step is open."
                      data-testid="step-detail-field"
                      @update:model-value="scheduleSave"
                    />
                  </div>
                </template>

                <div v-else-if="draftBody.trim()" class="step-detail min-w-0 overflow-x-auto">
                  <AppMarkdownEditor
                    :model-value="draftBody"
                    readonly
                    data-testid="step-detail-read"
                  />
                </div>

                <p v-else class="py-1 text-[12px] text-text-subtle">
                  Nothing written under this step yet.
                  <button
                    class="focus-ring text-accent underline-offset-2 hover:underline"
                    @click="mode = 'write'"
                  >
                    Write the detail
                  </button>
                </p>
              </div>
            </div>
          </li>
        </ol>
      </div>

      <!-- The end of the list, wherever the list has scrolled to. A step is always appended, so
           the field that adds one sits where the next one will land. -->
      <form
        class="flex shrink-0 items-center gap-2 border-t border-border bg-surface px-5 py-3"
        @submit.prevent="add"
      >
        <span
          class="grid size-[22px] shrink-0 place-items-center rounded-full border border-dashed border-border-strong text-text-subtle"
          aria-hidden="true"
        >
          <svg class="size-3" viewBox="0 0 12 12" fill="none">
            <path d="M6 2.4v7.2M2.4 6h7.2" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
          </svg>
        </span>
        <AppInput
          id="new-step"
          v-model="newTitle"
          placeholder="What is the next piece of this task?"
          data-testid="new-step"
        />
        <button
          type="submit"
          class="focus-ring h-(--spacing-control) shrink-0 rounded-[var(--radius-control)] border border-accent bg-accent-soft px-3.5 text-[12.5px] font-medium text-accent transition-colors hover:bg-accent hover:text-accent-ink disabled:cursor-not-allowed disabled:opacity-40"
          :disabled="!newTitle.trim()"
          data-testid="add-step"
        >
          Add
        </button>
      </form>
    </template>

    <AppConfirm
      v-if="deleting"
      title="Delete this step?"
      :body="`Removes &quot;${deleting.title}&quot; from this task's checklist.`"
      :blast="deletingBlast"
      confirm-label="Delete step"
      @cancel="deleting = null"
      @confirm="remove"
    />
  </section>
</template>

<style scoped>
/**
 * The detail of one step is read at the size of the row it belongs to.
 *
 * The shared markdown preview is built for a pane: a heading in it is the title of a document.
 * Here the title is the step above it, so the headings step down to what they are, subdivisions
 * of a paragraph, and the block loses the leading a standalone document earns.
 */
.step-detail :deep(.md-editor-preview) {
  font-size: 12.5px;
}

.step-detail :deep(.md-editor-preview) h1,
.step-detail :deep(.md-editor-preview) h2 {
  font-size: 14px;
  margin-top: 0.6em;
}

.step-detail :deep(.md-editor-preview) h3,
.step-detail :deep(.md-editor-preview) h4 {
  font-size: 13px;
  margin-top: 0.6em;
}

.step-detail :deep(.md-editor-preview) > :first-child {
  margin-top: 0;
}

.step-detail :deep(.md-editor-preview) p,
.step-detail :deep(.md-editor-preview) ul,
.step-detail :deep(.md-editor-preview) ol {
  margin-block: 0.5em;
}
</style>
