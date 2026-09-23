<script setup lang="ts">
/**
 * Where the run queue is set up and watched: the order on the left, how and when it runs on the
 * right, and one action at the foot that says exactly what will happen.
 *
 * The order is a relay: a numbered line of beads joined by a rail, the part already run drawn
 * solid and the part still to run in hairline, so where the run has got to reads without a word.
 * Only waiting tasks move or come off; the running one is stopped, not removed, and a finished
 * one stays until cleared, as the record of the run.
 *
 * Settings save as they change and apply from the next session opened and the next ceiling
 * check, so they can be adjusted while a queue runs. The start, the schedule and the stop are
 * the only things that wait for a button.
 */
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import AppButton from '@/components/ui/AppButton.vue'
import AppCheckbox from '@/components/ui/AppCheckbox.vue'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import AppSelect from '@/components/ui/AppSelect.vue'
import CloseGlyph from '@/components/ui/CloseGlyph.vue'
import CeilingGauge from '@/components/queue/CeilingGauge.vue'
import QueueBead from '@/components/queue/QueueBead.vue'
import QueueTaskPicker from '@/components/queue/QueueTaskPicker.vue'
import { useRunQueueStore } from '@/stores/runQueue.store'
import { useConsoleStore } from '@/stores/console.store'
import { useTerminalStore } from '@/stores/terminal.store'
import { useToastStore } from '@/stores/toast.store'
import { useModalGate } from '@/composables/useModalGate'
import { useNow } from '@/composables/useNow'
import { trapTabKey } from '@/common/a11y/focus-trap'
import { formatResetIn } from '@/common/format/countdown'
import { fetchClaudeUsage } from '@/api/claude.api'
import {
  CLAUDE_EFFORT_CHOICES,
  CLAUDE_MODEL_CHOICES,
  claudeEffortChoiceLabel,
  claudeModelChoiceLabel,
  type ClaudeUsageLimit
} from '@/model/claude'
import {
  DEFAULT_CEILING_PERCENT,
  clockLabel,
  fromLocalInputValue,
  isSettled,
  suggestedStart,
  toLocalInputValue,
  type RunQueueItem,
  type RunQueueItemState
} from '@/model/runQueue'
import type { TaskId } from '@/model/branded'

const emit = defineEmits<{ close: [] }>()

const SETTINGS_DEBOUNCE_MS = 350

const store = useRunQueueStore()
const consoleStore = useConsoleStore()
const terminals = useTerminalStore()
const toast = useToastStore()
const { queue, progress } = storeToRefs(store)
const { open: openModal, close: closeModal } = useModalGate()
const now = useNow(15_000)

const panel = ref<HTMLElement | null>(null)
const closeButton = ref<HTMLButtonElement | null>(null)
const busy = ref(false)
const confirmingStop = ref(false)

// ---------------------------------------------------------------- settings

const ceilingOn = ref(queue.value.ceilingPercent !== null)
const ceiling = ref(queue.value.ceilingPercent ?? DEFAULT_CEILING_PERCENT)
const skipPermissions = ref(queue.value.skipPermissions)
const model = ref<string>(queue.value.model ?? 'default')
const effort = ref<string>(queue.value.effort ?? 'default')

// Two selects share a 300px column, so 'Account default' is shortened to fit; the heading says whose.
const shortDefault = (value: string, label: string): string => (value === 'default' ? 'Default' : label)
const MODEL_OPTIONS = CLAUDE_MODEL_CHOICES.map((value) => ({ value, label: shortDefault(value, claudeModelChoiceLabel(value)) }))
const EFFORT_OPTIONS = CLAUDE_EFFORT_CHOICES.map((value) => ({ value, label: shortDefault(value, claudeEffortChoiceLabel(value)) }))

let settingsTimer: ReturnType<typeof setTimeout> | undefined
let pendingSave: Promise<void> | null = null

function saveSettingsNow(): Promise<void> {
  clearTimeout(settingsTimer)
  pendingSave = store
    .saveSettings({
      ceilingPercent: ceilingOn.value ? ceiling.value : null,
      skipPermissions: skipPermissions.value,
      model: model.value === 'default' ? null : model.value,
      effort: effort.value === 'default' ? null : effort.value
    })
    .catch((error: unknown) => toast.notifyError(error))
    .finally(() => (pendingSave = null))
  return pendingSave
}

watch([ceilingOn, ceiling, skipPermissions, model, effort], () => {
  clearTimeout(settingsTimer)
  settingsTimer = setTimeout(() => void saveSettingsNow(), SETTINGS_DEBOUNCE_MS)
})

// ---------------------------------------------------------------- usage for the gauge

const usageLimits = ref<readonly ClaudeUsageLimit[]>([])

/** The windows the server checks the ceiling against: session, weekly, and the model's own week. */
const watchedWindows = computed(() =>
  usageLimits.value.filter(
    (limit) =>
      limit.key === 'session' ||
      limit.key === 'weekly_all' ||
      (model.value !== 'default' && limit.key === `weekly_${model.value}`)
  )
)

async function loadUsage(): Promise<void> {
  try {
    const reading = await fetchClaudeUsage()
    usageLimits.value = reading.status === 'OK' ? reading.limits : []
  } catch {
    usageLimits.value = []
  }
}

// ---------------------------------------------------------------- when

const when = ref<'now' | 'later'>(queue.value.startAt ? 'later' : 'now')
const startLocal = ref(toLocalInputValue(queue.value.startAt ?? suggestedStart()))
const startIso = computed(() => fromLocalInputValue(startLocal.value))
const startInPast = computed(
  () => when.value === 'later' && (!startIso.value || Date.parse(startIso.value) <= now.value)
)

// ---------------------------------------------------------------- the order

const items = computed(() => queue.value.items)
const waitingIds = computed(() => items.value.filter((item) => item.state === 'QUEUED').map((item) => item.id))
const excluded = computed<ReadonlySet<TaskId>>(
  () => new Set(items.value.filter((item) => !isSettled(item.state)).map((item) => item.taskId))
)
const hasSettled = computed(() => items.value.some((item) => isSettled(item.state)))

const STATE_WORD: Readonly<Record<RunQueueItemState, string>> = {
  QUEUED: 'Waiting',
  RUNNING: 'Running',
  FINISHED: 'Finished',
  SKIPPED: 'Skipped',
  FAILED: 'Failed'
}

const STATE_TONE: Readonly<Record<RunQueueItemState, string>> = {
  QUEUED: 'text-text-subtle',
  RUNNING: 'text-accent',
  FINISHED: 'text-safe',
  SKIPPED: 'text-filed',
  FAILED: 'text-danger'
}

/** The rail below a bead is solid once the run has passed it: that bead has had its turn. */
function railPassed(item: RunQueueItem): boolean {
  return isSettled(item.state)
}

function waitingIndex(item: RunQueueItem): number {
  return waitingIds.value.indexOf(item.id)
}

async function act(action: () => Promise<void>): Promise<void> {
  if (busy.value) return
  busy.value = true
  try {
    await action()
  } catch (error) {
    toast.notifyError(error)
  } finally {
    busy.value = false
  }
}

function add(taskId: TaskId): void {
  void act(() => store.add(taskId))
}

function moveBy(item: RunQueueItem, delta: number): void {
  const index = waitingIndex(item) + delta
  if (index < 0 || index >= waitingIds.value.length) return
  void act(() => store.move(item.id, index))
}

function remove(item: RunQueueItem): void {
  void act(() => store.remove(item.id))
}

function clearSettled(): void {
  void act(() => store.clearSettled())
}

async function watchSession(item: RunQueueItem): Promise<void> {
  try {
    await terminals.load()
  } catch (error) {
    toast.notifyError(error)
    return
  }
  const terminal = terminals.terminalForTask(item.taskId)
  if (terminal) terminals.select(terminal.id)
  consoleStore.selectTask(item.taskId)
  consoleStore.openTerminal()
  emit('close')
}

// ---------------------------------------------------------------- start and stop

const armed = computed(() => queue.value.state !== 'IDLE')
const running = computed(() => queue.value.state === 'RUNNING' || queue.value.state === 'HOLDING')

const startLabel = computed(() => {
  if (when.value === 'now') return queue.value.state === 'SCHEDULED' ? 'Start now instead' : 'Start now'
  return `Schedule for ${clockLabel(startIso.value, now.value) || '…'}`
})

const startBlocked = computed(
  () => progress.value.waiting === 0 || running.value || (when.value === 'later' && startInPast.value)
)

function start(): void {
  void act(async () => {
    if (pendingSave) await pendingSave
    else await saveSettingsNow()
    await store.start(when.value === 'now' ? null : startIso.value)
    toast.notify(when.value === 'now' ? 'Queue started' : `Queue scheduled for ${clockLabel(startIso.value)}`)
  })
}

function requestStop(): void {
  if (running.value && progress.value.running) {
    confirmingStop.value = true
    return
  }
  const wasScheduled = queue.value.state === 'SCHEDULED'
  void act(async () => {
    await store.stop()
    toast.notify(wasScheduled ? 'Schedule cancelled' : 'Queue stopped')
  })
}

function confirmStop(): void {
  confirmingStop.value = false
  void act(async () => {
    await store.stop()
    toast.notify('Queue stopped')
  })
}

const stopBlast = computed(() => {
  const item = progress.value.running
  return item
    ? `Closes the Claude session working on ${item.taskTitle}. Claimed steps stay claimed; the one it was on goes back to open.`
    : ''
})

// ---------------------------------------------------------------- the status line

const status = computed(() => {
  const current = queue.value
  switch (current.state) {
    case 'SCHEDULED':
      return {
        tone: 'scheduled' as const,
        title: `Starts at ${clockLabel(current.startAt, now.value)}`,
        detail: `In ${formatResetIn(current.startAt, now.value)}. Keep Rekall open: the queue runs inside it.`
      }
    case 'RUNNING':
      return {
        tone: 'running' as const,
        title: progress.value.running ? `Running ${progress.value.running.taskTitle}` : 'Starting the next task',
        detail: `${progress.value.settled} of ${progress.value.total} done, ${progress.value.waiting} waiting.`
      }
    case 'HOLDING':
      return {
        tone: 'holding' as const,
        title: `Holding until ${clockLabel(current.holdUntil, now.value)}`,
        detail: `${current.holdReason ?? ''} Resumes in ${formatResetIn(current.holdUntil, now.value)}.`
      }
    default:
      return null
  }
})

// ---------------------------------------------------------------- dialog

function onKeydown(event: KeyboardEvent): void {
  if (confirmingStop.value) return
  if (event.key === 'Escape') {
    event.stopPropagation()
    emit('close')
    return
  }
  if (panel.value) trapTabKey(panel.value, event)
}

onMounted(async () => {
  openModal()
  window.addEventListener('keydown', onKeydown, true)
  void loadUsage()
  await store.load().catch((error: unknown) => toast.notifyError(error))
  await nextTick()
  closeButton.value?.focus()
})

onUnmounted(() => {
  if (settingsTimer) void saveSettingsNow()
  closeModal()
  window.removeEventListener('keydown', onKeydown, true)
})
</script>

<template>
  <div
    class="fixed inset-0 z-(--z-modal) grid place-items-center bg-black/70 p-5 backdrop-blur-sm"
    @click.self="!confirmingStop && emit('close')"
  >
    <div
      ref="panel"
      class="dialog-panel flex max-h-[88vh] w-full max-w-[920px] flex-col overflow-hidden rounded-[var(--radius-card)] border border-border-strong bg-surface shadow-modal"
      role="dialog"
      aria-modal="true"
      aria-label="Run queue"
      data-testid="run-queue-panel"
    >
      <header class="flex items-center gap-3 border-b border-border px-6 py-4">
        <div class="min-w-0">
          <h2 class="text-[15px] font-semibold tracking-[-0.01em] text-text">Run queue</h2>
          <p class="text-[12px] text-text-subtle">
            Tasks run one after another, each in its own Claude session, until everything on it is
            claimed.
          </p>
        </div>
        <button
          ref="closeButton"
          type="button"
          class="focus-ring ml-auto grid size-8 shrink-0 place-items-center rounded-[var(--radius-control)] text-text-subtle transition-colors hover:bg-surface-raised hover:text-text"
          aria-label="Close"
          data-testid="run-queue-close"
          @click="emit('close')"
        >
          <CloseGlyph />
        </button>
      </header>

      <Transition name="fade-quick">
        <div
          v-if="status"
          class="flex items-baseline gap-3 border-b px-6 py-2.5"
          :class="{
            'border-accent/30 bg-accent-soft': status.tone === 'running',
            'border-warn/30 bg-warn-soft': status.tone === 'holding',
            'border-border bg-surface-raised': status.tone === 'scheduled'
          }"
          role="status"
          aria-live="polite"
          data-testid="run-queue-status"
        >
          <span
            class="shrink-0 text-[13px] font-semibold"
            :class="status.tone === 'holding' ? 'text-warn' : status.tone === 'running' ? 'text-accent' : 'text-text'"
          >
            {{ status.title }}
          </span>
          <span class="min-w-0 text-[12px] text-text-muted">{{ status.detail }}</span>
        </div>
      </Transition>

      <div class="grid min-h-0 flex-1 grid-cols-1 md:grid-cols-[minmax(0,1fr)_300px]">
        <!-- The order -->
        <section class="flex min-h-0 flex-col" aria-labelledby="queue-order-heading">
          <div class="flex items-center gap-2 px-6 pb-2 pt-4">
            <h3 id="queue-order-heading" class="section-label">Order</h3>
            <span class="font-mono text-[11px] tabular-nums text-text-subtle">
              {{ progress.waiting }} waiting<template v-if="progress.settled">, {{ progress.settled }} run</template>
            </span>
            <AppButton
              v-if="hasSettled"
              class="ml-auto"
              variant="ghost"
              size="sm"
              :disabled="busy"
              data-testid="run-queue-clear"
              @click="clearSettled"
            >
              Clear finished
            </AppButton>
          </div>

          <div class="min-h-[180px] flex-1 overflow-y-auto px-6 pb-3">
            <div
              v-if="items.length === 0"
              class="flex flex-col items-start gap-1.5 rounded-[var(--radius-card)] border border-dashed border-border px-5 py-8"
              data-testid="run-queue-empty"
            >
              <p class="text-[13.5px] font-medium text-text">Nothing queued</p>
              <p class="max-w-[46ch] text-[12.5px] leading-relaxed text-text-muted">
                Add the tasks you want run, in the order you want them. Each one gets a Claude
                session that works its open steps and claims them, then the next one starts.
              </p>
            </div>

            <TransitionGroup v-else tag="ol" name="step-list" class="relative flex flex-col" data-testid="run-queue-list">
              <li
                v-for="(item, index) in items"
                :key="item.id"
                class="group relative grid grid-cols-[22px_16px_minmax(0,1fr)_auto] items-start gap-x-2.5 rounded-[var(--radius-control)] py-2 pr-1"
                :class="item.state === 'RUNNING' ? 'bg-accent-soft' : ''"
                :data-state="item.state"
                data-testid="run-queue-item"
              >
                <span class="pt-0.5 text-right font-mono text-[11px] tabular-nums text-text-subtle">
                  {{ index + 1 }}
                </span>

                <span class="relative flex h-full justify-center pt-[5px]">
                  <QueueBead :state="item.state" size="md" :live="queue.state === 'RUNNING'" />
                  <span
                    v-if="index < items.length - 1"
                    class="absolute bottom-[-10px] top-[20px] w-px"
                    :class="railPassed(item) ? 'bg-border-strong' : 'bg-border'"
                    aria-hidden="true"
                  />
                </span>

                <span class="flex min-w-0 flex-col gap-0.5">
                  <span class="truncate text-[13px] font-medium text-text">{{ item.taskTitle }}</span>
                  <span class="flex min-w-0 items-baseline gap-2 text-[11px]">
                    <span class="truncate font-mono text-anchor">{{ item.anchor }}</span>
                    <span class="shrink-0" :class="STATE_TONE[item.state]">{{ STATE_WORD[item.state] }}</span>
                  </span>
                  <span v-if="item.detail" class="text-[11.5px] leading-snug text-text-muted">{{ item.detail }}</span>
                </span>

                <span class="flex items-center gap-0.5 self-center">
                  <template v-if="item.state === 'QUEUED'">
                    <button
                      type="button"
                      class="focus-ring grid size-7 place-items-center rounded-md text-text-subtle transition-colors hover:bg-surface-raised hover:text-text disabled:opacity-30"
                      :aria-label="`Move ${item.taskTitle} earlier`"
                      :disabled="busy || waitingIndex(item) === 0"
                      data-testid="run-queue-up"
                      @click="moveBy(item, -1)"
                    >
                      <svg class="size-3.5" viewBox="0 0 16 16" fill="none" aria-hidden="true">
                        <path d="M4 10l4-4 4 4" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" />
                      </svg>
                    </button>
                    <button
                      type="button"
                      class="focus-ring grid size-7 place-items-center rounded-md text-text-subtle transition-colors hover:bg-surface-raised hover:text-text disabled:opacity-30"
                      :aria-label="`Move ${item.taskTitle} later`"
                      :disabled="busy || waitingIndex(item) === waitingIds.length - 1"
                      data-testid="run-queue-down"
                      @click="moveBy(item, 1)"
                    >
                      <svg class="size-3.5" viewBox="0 0 16 16" fill="none" aria-hidden="true">
                        <path d="M4 6l4 4 4-4" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" />
                      </svg>
                    </button>
                    <AppButton
                      variant="danger-quiet"
                      size="sm"
                      :disabled="busy"
                      :aria-label="`Take ${item.taskTitle} off the queue`"
                      data-testid="run-queue-remove"
                      @click="remove(item)"
                    >
                      Remove
                    </AppButton>
                  </template>
                  <AppButton
                    v-else-if="item.state === 'RUNNING'"
                    variant="secondary"
                    size="sm"
                    data-testid="run-queue-watch"
                    @click="watchSession(item)"
                  >
                    Watch session
                  </AppButton>
                </span>
              </li>
            </TransitionGroup>
          </div>

          <div class="border-t border-border px-6 py-3">
            <QueueTaskPicker :excluded="excluded" :disabled="busy" @pick="add" />
          </div>
        </section>

        <!-- How and when -->
        <aside class="flex min-h-0 flex-col gap-6 overflow-y-auto border-t border-border px-5 py-4 md:border-l md:border-t-0">
          <section aria-labelledby="queue-when-heading">
            <h3 id="queue-when-heading" class="section-label mb-2">When</h3>
            <div class="relative flex gap-0.5 rounded-[var(--radius-control)] bg-surface p-0.5" role="radiogroup" aria-label="When the queue starts">
              <button
                v-for="choice in (['now', 'later'] as const)"
                :key="choice"
                type="button"
                role="radio"
                :aria-checked="when === choice"
                class="focus-ring h-7 flex-1 rounded-[7px] text-[12px] transition-colors"
                :class="when === choice ? 'bg-surface-raised text-text shadow-[0_1px_2px_rgb(0_0_0/0.4)]' : 'text-text-subtle hover:text-text'"
                :data-testid="`run-queue-when-${choice}`"
                @click="when = choice"
              >
                {{ choice === 'now' ? 'Now' : 'At a time' }}
              </button>
            </div>
            <div v-if="when === 'later'" class="mt-2.5">
              <label for="queue-start-at" class="sr-only">Start time</label>
              <input
                id="queue-start-at"
                v-model="startLocal"
                type="datetime-local"
                class="field h-(--spacing-control) w-full rounded-[var(--radius-control)] px-3 font-mono text-[12.5px] text-text [color-scheme:dark]"
                :aria-invalid="startInPast"
                data-testid="run-queue-start-at"
              />
              <p class="mt-1.5 text-[11.5px]" :class="startInPast ? 'text-danger' : 'text-text-subtle'">
                <template v-if="startInPast">That time has passed. Pick a later one.</template>
                <template v-else>In {{ formatResetIn(startIso, now) }}. Rekall has to be open then.</template>
              </p>
            </div>
          </section>

          <section aria-labelledby="queue-ceiling-heading">
            <div class="mb-1 flex items-center justify-between gap-2">
              <h3 id="queue-ceiling-heading" class="section-label">Usage ceiling</h3>
              <AppCheckbox v-model="ceilingOn" label="On" data-testid="run-queue-ceiling-on" />
            </div>
            <p class="mb-2 text-[11.5px] leading-relaxed text-text-muted">
              <template v-if="ceilingOn">
                No new task or step starts once a window reaches the line. The queue waits for that
                window to reset, then carries on.
              </template>
              <template v-else>Off: the queue runs until it is empty, whatever the usage.</template>
            </p>
            <CeilingGauge v-model="ceiling" :windows="watchedWindows" :disabled="!ceilingOn" :now="now" />
          </section>

          <section aria-labelledby="queue-session-heading" class="flex flex-col gap-3">
            <h3 id="queue-session-heading" class="section-label">Sessions</h3>
            <div class="grid grid-cols-2 gap-2">
              <div>
                <label for="queue-model" class="mb-1 block text-[11px] text-text-subtle">Model</label>
                <AppSelect id="queue-model" v-model="model" :options="MODEL_OPTIONS" />
              </div>
              <div>
                <label for="queue-effort" class="mb-1 block text-[11px] text-text-subtle">Effort</label>
                <AppSelect id="queue-effort" v-model="effort" :options="EFFORT_OPTIONS" />
              </div>
            </div>
            <div>
              <AppCheckbox v-model="skipPermissions" label="Skip permission prompts" data-testid="run-queue-skip" />
              <p class="mt-1.5 text-[11.5px] leading-relaxed" :class="skipPermissions ? 'text-text-subtle' : 'text-warn'">
                <template v-if="skipPermissions">Sessions run with <span class="font-mono">--dangerously-skip-permissions</span>.</template>
                <template v-else>Off: an unattended session stops at its first permission prompt and waits for you.</template>
              </p>
            </div>
          </section>
        </aside>
      </div>

      <footer class="flex flex-wrap items-center justify-end gap-2 border-t border-border bg-canvas px-6 py-3">
        <p v-if="progress.waiting === 0 && !armed" class="mr-auto text-[12px] text-text-subtle">
          Add a task to start the queue.
        </p>
        <AppButton
          v-if="armed"
          variant="danger"
          size="sm"
          :disabled="busy"
          data-testid="run-queue-stop"
          @click="requestStop"
        >
          {{ queue.state === 'SCHEDULED' ? 'Cancel schedule' : 'Stop the queue' }}
        </AppButton>
        <AppButton
          v-if="!running"
          variant="primary"
          size="sm"
          :disabled="startBlocked || busy"
          :loading="busy"
          data-testid="run-queue-start"
          @click="start"
        >
          {{ startLabel }}
        </AppButton>
      </footer>
    </div>

    <Transition name="dialog">
      <AppConfirm
        v-if="confirmingStop"
        title="Stop the queue?"
        body="The queue stops starting tasks, and the session it has open is closed now rather than at the end of its step."
        :blast="stopBlast"
        confirm-label="Stop the queue"
        @cancel="confirmingStop = false"
        @confirm="confirmStop"
      />
    </Transition>
  </div>
</template>
