<script setup lang="ts">
/**
 * Where the run queue is set up and watched: the order on the left, how and when it runs on the
 * right, and one action at the foot that says exactly what will happen.
 *
 * Settings save as they change and apply from the next session opened and the next ceiling
 * check, so they can be adjusted while a queue runs. The start, the schedule and the stop are
 * the only things that wait for a button.
 */
import { computed, nextTick, onMounted, ref } from 'vue'
import { storeToRefs } from 'pinia'
import AppButton from '@/components/ui/AppButton.vue'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import AppDialog from '@/components/ui/AppDialog.vue'
import CloseGlyph from '@/components/ui/CloseGlyph.vue'
import QueueOrderSection from '@/features/queue/components/QueueOrderSection.vue'
import QueueSettingsAside from '@/features/queue/components/QueueSettingsAside.vue'
import QueueStatusLine from '@/features/queue/components/QueueStatusLine.vue'
import { useRunQueueStore } from '@/features/queue/store/runQueue.store'
import { useConsoleStore } from '@/stores/console.store'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import { useToastStore } from '@/stores/toast.store'
import { useNow } from '@/composables/useNow'
import { useQueueSettingsDraft } from '@/features/queue/composables/useQueueSettingsDraft'
import {
  clockLabel,
  fromLocalInputValue,
  suggestedStart,
  toLocalInputValue,
  type RunQueueItem
} from '@/features/queue/model/runQueue'
import type { TaskId } from '@/model/branded'

const emit = defineEmits<{ close: [] }>()

const store = useRunQueueStore()
const consoleStore = useConsoleStore()
const terminals = useTerminalStore()
const toast = useToastStore()
const { queue, progress } = storeToRefs(store)
const now = useNow(15_000)

const closeButton = ref<HTMLButtonElement | null>(null)
const busy = ref(false)
const confirmingStop = ref(false)

// ---------------------------------------------------------------- settings

const { ceilingOn, ceiling, skipPermissions, model, effort, settle } = useQueueSettingsDraft(
  queue.value,
  (settings) => store.saveSettings(settings),
  (error) => toast.notifyError(error)
)

// ---------------------------------------------------------------- when

const when = ref<'now' | 'later'>(queue.value.startAt ? 'later' : 'now')
const startLocal = ref(toLocalInputValue(queue.value.startAt ?? suggestedStart()))
const startIso = computed(() => fromLocalInputValue(startLocal.value))
const startInPast = computed(
  () => when.value === 'later' && (!startIso.value || Date.parse(startIso.value) <= now.value)
)

// ---------------------------------------------------------------- the order

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

function moveTo(item: RunQueueItem, index: number): void {
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
    await settle()
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

// ---------------------------------------------------------------- dialog

onMounted(async () => {
  await store.load().catch((error: unknown) => toast.notifyError(error))
  await nextTick()
  closeButton.value?.focus()
})
</script>

<template>
  <AppDialog
    label="Run queue"
    panel-class="flex flex-col max-h-[88vh] max-w-[920px]"
    panel-testid="run-queue-panel"
    @dismiss="emit('close')"
  >
    <header class="flex items-center gap-3 border-b border-border px-6 py-4">
      <div class="min-w-0">
        <h2 class="text-title font-semibold tracking-[-0.01em] text-text">Run queue</h2>
        <p class="text-small text-text-subtle">
          Tasks run one after another, each in its own Claude session, until everything on it is
          claimed.
        </p>
      </div>
      <button
        ref="closeButton"
        type="button"
        class="focus-ring ml-auto grid size-8 shrink-0 place-items-center rounded-control text-text-subtle transition-colors hover:bg-surface-raised hover:text-text"
        aria-label="Close"
        data-testid="run-queue-close"
        @click="emit('close')"
      >
        <CloseGlyph />
      </button>
    </header>

    <QueueStatusLine :queue="queue" :progress="progress" :now="now" />

    <div class="grid min-h-0 flex-1 grid-cols-1 md:grid-cols-[minmax(0,1fr)_300px]">
      <QueueOrderSection
        :queue="queue"
        :progress="progress"
        :busy="busy"
        @add="add"
        @move="moveTo"
        @remove="remove"
        @clear="clearSettled"
        @watch="watchSession"
      />

      <QueueSettingsAside
        v-model:when="when"
        v-model:start-local="startLocal"
        v-model:ceiling-on="ceilingOn"
        v-model:ceiling="ceiling"
        v-model:model="model"
        v-model:effort="effort"
        v-model:skip-permissions="skipPermissions"
        :now="now"
        :start-iso="startIso"
        :start-in-past="startInPast"
      />
    </div>

    <footer class="flex flex-wrap items-center justify-end gap-2 border-t border-border bg-canvas px-6 py-3">
      <p v-if="progress.waiting === 0 && !armed" class="mr-auto text-small text-text-subtle">
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
    <template #overlay>
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
    </template>
  </AppDialog>
</template>
