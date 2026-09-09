<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import AppInput from '@/components/ui/AppInput.vue'
import AppMarkdownEditor from '@/components/ui/AppMarkdownEditor.vue'
import LaunchClaudeCodeButton from '@/components/claude/LaunchClaudeCodeButton.vue'
import ClaudeSessionLauncher from '@/components/claude/ClaudeSessionLauncher.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { identityHue } from '@/common/identity'
import { relativeTime } from '@/common/format/relative-time'
import { rkCommand } from '@/common/format/rk-command'
import { stepIsComplete, type TaskStep } from '@/model/catalog'
import type { TaskStepId } from '@/model/branded'

const store = useConsoleStore()
const { selectedTask, selectedTaskSteps } = storeToRefs(store)
const { run } = useAsyncAction()

const hue = computed(() => identityHue(selectedTask.value?.projectId ?? ''))

const done = computed(() => selectedTaskSteps.value.filter((step) => step.done).length)
const claimed = computed(
  () => selectedTaskSteps.value.filter((step) => step.state === 'CLAIMED').length
)

const currentId = computed(
  () => selectedTaskSteps.value.find((step) => !stepIsComplete(step.state))?.id ?? null
)

const runningStep = computed(
  () => selectedTaskSteps.value.find((step) => step.state === 'RUNNING') ?? null
)

const hideDone = ref(false)
const visibleSteps = computed(() =>
  hideDone.value ? selectedTaskSteps.value.filter((step) => !step.done) : selectedTaskSteps.value
)

function railKind(index: number): 'spent' | 'pending' {
  const step = visibleSteps.value[index]
  return step && stepIsComplete(step.state) ? 'spent' : 'pending'
}

function checkboxLabel(step: TaskStep): string {
  if (step.state === 'DONE') return `${step.title} is done. Reopen it from its detail.`
  if (step.state === 'CLAIMED') return `Accept ${step.title}`
  return `Mark ${step.title} done`
}

const firstClaimed = computed(
  () => selectedTaskSteps.value.find((step) => step.state === 'CLAIMED') ?? null
)

const newTitle = ref('')

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

const expandedId = ref<TaskStepId | null>(null)
const reopenArmed = ref<TaskStepId | null>(null)
const mode = ref<'write' | 'read'>('read')
const draftTitle = ref('')
const draftBody = ref('')
let saveTimer: ReturnType<typeof setTimeout> | null = null

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

watch(expandedId, () => {
  reopenArmed.value = null
})

watch(
  () => selectedTask.value?.id ?? null,
  () => {
    flush()
    hideDone.value = false
    expandedId.value = null
    const next = selectedTaskSteps.value.find((step) => !stepIsComplete(step.state))
    if (next) open(next)
  },
  { immediate: true }
)

async function markForward(step: TaskStep): Promise<void> {
  if (step.state === 'DONE') return
  const wasOpenHere = expandedId.value === step.id
  flush()
  await run(() => (step.state === 'CLAIMED' ? store.acceptStep(step.id) : store.toggleStep(step.id)))
  if (!wasOpenHere) return

  const next = selectedTaskSteps.value.find((candidate) => !stepIsComplete(candidate.state))
  if (next && next.id !== step.id) open(next)
  else expandedId.value = null
}

async function sendBack(step: TaskStep): Promise<void> {
  flush()
  await run(() => store.reopenStep(step.id))
  open(step)
}

async function reopen(step: TaskStep): Promise<void> {
  if (reopenArmed.value !== step.id) {
    reopenArmed.value = step.id
    return
  }
  reopenArmed.value = null
  flush()
  await run(() => store.reopenStep(step.id))
  open(step)
}

async function reviewFirst(): Promise<void> {
  const step = firstClaimed.value
  if (!step) return
  open(step)
  await nextTick()
  listEl.value
    ?.querySelector<HTMLElement>(`li[data-step-id="${step.id}"]`)
    ?.scrollIntoView({ block: 'center', behavior: 'smooth' })
}

async function move(step: TaskStep, by: number): Promise<void> {
  flush()
  await run(() => store.moveStep(step.id, step.position + by))
}

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

function railStyle(index: number): Record<string, string> {
  const isFirst = index === 0
  const isLast = index === visibleSteps.value.length - 1
  const isRunning = visibleSteps.value[index]?.state === 'RUNNING'
  if (isFirst && isLast) return { display: 'none' }
  if (isRunning && !isFirst) return isLast ? { display: 'none' } : { top: '18px', bottom: '0' }
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

const listEl = ref<HTMLElement | null>(null)
const stream = ref<{ top: number; height: number } | null>(null)

const NODE_CENTER_OFFSET = 18

function measureStream(): void {
  const list = listEl.value
  if (!list) {
    stream.value = null
    return
  }
  const rows = Array.from(list.querySelectorAll<HTMLElement>('li[data-testid="step-row"]'))
  const runningIndex = rows.findIndex((row) => row.dataset.stepState === 'RUNNING')
  const startIndex = rows.findIndex(
    (row) => row.dataset.stepState === 'CLAIMED' || row.dataset.stepState === 'DONE'
  )
  const startRow = rows[startIndex]
  const runningRow = rows[runningIndex]
  if (!startRow || !runningRow || runningIndex < 1 || startIndex >= runningIndex) {
    stream.value = null
    return
  }
  const top = startRow.offsetTop + NODE_CENTER_OFFSET
  const height = runningRow.offsetTop + NODE_CENTER_OFFSET - top
  stream.value = height > 4 ? { top, height } : null
}

function scheduleMeasure(): void {
  void nextTick(measureStream)
}

let rowObserver: ResizeObserver | null = null
if (typeof ResizeObserver !== 'undefined') {
  rowObserver = new ResizeObserver(() => measureStream())
}

watch(listEl, (element) => {
  rowObserver?.disconnect()
  if (element) rowObserver?.observe(element)
  scheduleMeasure()
})

watch([visibleSteps, expandedId, mode, () => draftBody.value], scheduleMeasure)

onMounted(scheduleMeasure)
onUnmounted(() => rowObserver?.disconnect())
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
            <p class="eyebrow flex items-center gap-1.5">
              <span class="h-2.5 w-[3px] shrink-0 rounded-full bg-accent" aria-hidden="true" />
              Steps
            </p>
            <h2 class="flex items-center gap-2 truncate text-[19px] font-semibold tracking-[-0.015em] text-text">
              <span class="truncate">{{ selectedTask.title }}</span>
              <span
                v-if="runningStep"
                class="inline-flex shrink-0 items-center gap-1.5 rounded-full bg-accent-soft px-2 py-0.5 text-[10.5px] font-semibold tracking-[0.02em] text-accent"
                data-testid="steps-running-flag"
              >
                <span class="relative grid size-2 place-items-center" aria-hidden="true">
                  <span class="absolute inline-flex size-2 animate-ping rounded-full bg-accent/60" />
                  <span class="relative inline-flex size-1.5 rounded-full bg-accent" />
                </span>
                running
              </span>
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

          <div class="flex shrink-0 items-center gap-1.5">
            <ClaudeSessionLauncher
              :task-id="selectedTask.id"
              :step-id="currentId"
              :folder="selectedTask.projectRepoFolder"
            />
            <LaunchClaudeCodeButton
              :anchors="selectedTask.anchor"
              :folder="selectedTask.projectRepoFolder"
              missing-hint="Set this project's folder on its page to open a session from it"
            />
          </div>

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
                  step.state === 'DONE'
                    ? 'bg-accent'
                    : step.state === 'CLAIMED'
                      ? 'bg-accent/60'
                      : step.state === 'RUNNING'
                        ? 'bg-accent/50 animate-pulse'
                        : step.id === currentId
                          ? 'bg-accent/25'
                          : 'bg-border-strong'
                "
              />
            </span>
            <button
              v-if="claimed > 0"
              class="focus-ring mt-1 text-[10.5px] text-accent underline-offset-2 transition-colors hover:underline"
              data-testid="steps-claimed-count"
              @click="reviewFirst"
            >
              {{ claimed }} awaiting review
            </button>
            <button
              v-if="done > 0 || claimed > 0"
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
          What is left, in order. A session moves a step from open to running to claimed over
          <code class="text-anchor/80">/rk {{ selectedTask.anchor }}</code>; the last tick, done,
          is yours.
        </span>
      </div>

      <div class="min-h-0 min-w-0 flex-1 overflow-y-auto px-5 py-4">
        <div v-if="!selectedTaskSteps.length" class="max-w-[560px] py-4">
          <div class="relative mb-7" aria-hidden="true">
            <span
              class="absolute bottom-[18px] left-[11px] top-[18px] w-px -translate-x-1/2 bg-border-strong"
            />
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

        <ol v-else ref="listEl" class="relative min-w-0">
          <span
            v-if="stream"
            class="energy-stream"
            :style="{ top: `${stream.top}px`, height: `${stream.height}px` }"
            data-testid="energy-stream"
            aria-hidden="true"
          />
          <li
            v-for="(step, index) in visibleSteps"
            :key="step.id"
            class="group/step relative min-w-0 pb-1.5 pl-9"
            data-testid="step-row"
            :data-step-id="step.id"
            :data-step-state="step.state"
          >
            <span
              class="absolute left-[11px] -translate-x-1/2"
              :class="{
                'w-px bg-border-strong': railKind(index) === 'pending',
                'w-px spent-rail': railKind(index) === 'spent'
              }"
              :style="railStyle(index)"
              aria-hidden="true"
            />

            <button
              class="focus-ring absolute left-0 top-[7px] z-10 grid size-[22px] place-items-center rounded-full border transition-all duration-200"
              :class="{
                'border-accent bg-accent text-accent-ink cursor-default': step.state === 'DONE',
                'border-accent bg-accent-soft text-accent': step.state === 'CLAIMED',
                'step-node-running border-accent bg-canvas text-accent shadow-[0_0_0_4px_var(--color-accent-soft)]':
                  step.state === 'RUNNING',
                'border-accent bg-canvas text-accent shadow-[0_0_0_4px_var(--color-accent-soft)]':
                  step.state === 'OPEN' && step.id === currentId,
                'border-border-strong bg-canvas text-transparent hover:border-accent':
                  step.state === 'OPEN' && step.id !== currentId
              }"
              role="checkbox"
              :aria-checked="
                step.state === 'DONE' ? 'true' : step.state === 'CLAIMED' ? 'mixed' : 'false'
              "
              :aria-label="checkboxLabel(step)"
              data-testid="step-checkbox"
              @click="markForward(step)"
            >
              <svg
                v-if="step.state === 'DONE'"
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
                v-else-if="step.state === 'CLAIMED'"
                class="size-2.5 rounded-full border-[1.5px] border-accent"
                aria-hidden="true"
              />
              <span
                v-else-if="step.state === 'RUNNING' || step.id === currentId"
                class="size-[7px] rounded-full bg-accent"
                aria-hidden="true"
              />
            </button>

            <div
              class="min-w-0 rounded-[var(--radius-card)] border px-3 py-2 transition-all"
              :class="[
                step.state === 'RUNNING'
                  ? 'border-accent/60 bg-accent-soft'
                  : step.state === 'CLAIMED'
                    ? 'border-accent/25 bg-accent-soft/40'
                    : step.id === currentId
                      ? 'border-accent/30 bg-surface-raised'
                      : 'border-transparent group-hover/step:border-border',
                step.state === 'DONE' && 'opacity-60 hover:opacity-100'
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
                      step.state === 'DONE'
                        ? 'text-text-subtle line-through decoration-text-subtle/50'
                        : 'text-text'
                    "
                  >
                    {{ step.title }}
                  </span>
                  <span class="mt-1 flex flex-wrap items-center gap-2 text-[10.5px]">
                    <span
                      v-if="step.state === 'RUNNING'"
                      class="inline-flex items-center gap-1 rounded-full bg-accent px-1.5 py-px text-[10px] font-semibold tracking-[0.02em] text-accent-ink"
                      data-testid="step-running-badge"
                    >
                      Running
                    </span>
                    <span
                      v-else-if="step.state === 'CLAIMED'"
                      class="rounded-full border border-accent/40 px-1.5 py-px text-[10px] font-semibold tracking-[0.02em] text-accent"
                      data-testid="step-claimed-badge"
                    >
                      Awaiting review
                    </span>
                    <span
                      v-else-if="step.id === currentId"
                      class="rounded-full bg-accent-soft px-1.5 py-px text-[10px] font-semibold tracking-[0.02em] text-accent"
                    >
                      Next
                    </span>
                    <span v-if="step.state === 'CLAIMED' && step.claimedAt" class="text-text-subtle">
                      claimed {{ relativeTime(step.claimedAt) }}
                    </span>
                    <span v-if="step.state === 'DONE' && step.doneAt" class="text-text-subtle">
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
                    <span v-else-if="!stepIsComplete(step.state)" class="text-text-subtle">
                      no detail yet
                    </span>
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

              <div v-if="expandedId === step.id" class="mt-2.5" data-testid="step-detail">
                <div
                  v-if="step.state === 'CLAIMED'"
                  class="mb-3 flex flex-wrap items-center gap-2 rounded-[var(--radius-card)] border border-accent/25 bg-accent-soft/40 px-3 py-2"
                  data-testid="step-review-bar"
                >
                  <span class="min-w-0 flex-1 text-[11.5px] leading-snug text-text-muted">
                    A session claimed this. Accept it to tick the box, or send it back to reopen it
                    for another pass.
                  </span>
                  <button
                    class="focus-ring h-7 shrink-0 rounded-[var(--radius-control)] border border-accent bg-accent-soft px-3 text-[11.5px] font-medium text-accent transition-colors hover:bg-accent hover:text-accent-ink"
                    data-testid="step-accept"
                    @click="markForward(step)"
                  >
                    Accept
                  </button>
                  <button
                    class="focus-ring h-7 shrink-0 rounded-[var(--radius-control)] border border-border-strong px-3 text-[11.5px] font-medium text-text-subtle transition-colors hover:border-danger hover:text-danger"
                    data-testid="step-send-back"
                    @click="sendBack(step)"
                  >
                    Send back
                  </button>
                </div>

                <div
                  v-else-if="step.state === 'DONE'"
                  class="mb-3 flex flex-wrap items-center gap-2 rounded-[var(--radius-card)] border border-border px-3 py-2"
                  data-testid="step-reopen-bar"
                >
                  <span class="min-w-0 flex-1 text-[11.5px] leading-snug text-text-muted">
                    Accepted. Reopening drops it back to open and clears the review it passed.
                  </span>
                  <button
                    class="focus-ring h-7 shrink-0 rounded-[var(--radius-control)] border px-3 text-[11.5px] font-medium transition-colors"
                    :class="
                      reopenArmed === step.id
                        ? 'border-danger bg-danger-soft text-danger'
                        : 'border-border-strong text-text-subtle hover:border-danger hover:text-danger'
                    "
                    data-testid="step-reopen"
                    @click="reopen(step)"
                  >
                    {{ reopenArmed === step.id ? 'Confirm reopen' : 'Reopen' }}
                  </button>
                </div>

                <div class="mb-2 flex items-center gap-2">
                  <span
                    class="eyebrow text-[9.5px]"
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

      <form
        class="dock-lane-safe flex shrink-0 items-center gap-2 border-t border-border bg-surface py-3 pl-5"
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
