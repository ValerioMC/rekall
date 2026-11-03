<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import AppButton from '@/components/ui/AppButton.vue'
import AppEmptyState from '@/components/ui/AppEmptyState.vue'
import ChatApprovalCard from './ChatApprovalCard.vue'
import ChatComposer from './ChatComposer.vue'
import ChatHeader from './ChatHeader.vue'
import ChatTimeline from './ChatTimeline.vue'
import { preferredEffort, preferredModel, skipsPermissions } from '@/common/config/claude-launch'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useAttentionTitle } from '@/features/chat/composables/useAttentionTitle'
import { useChatEvents } from '@/features/chat/composables/useChatEvents'
import { chatStatus, composerState } from '@/features/chat/lib/chat-status'
import { failureMessage } from '@/features/chat/lib/failure-message'
import { MIN_WIDTH, useChatDockStore } from '@/features/chat/store/chatDock.store'
import { useChatStore } from '@/features/chat/store/chat.store'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import { useConsoleStore } from '@/stores/console.store'
import type { Terminal } from '@/features/claude/model/terminal'
import type { TaskId, TerminalId } from '@/model/branded'

/**
 * The chat session as a panel docked beside the note pane, on the task the session runs on. It
 * takes the answers the session waits for: messages, stop, and the allow or deny of a tool call.
 */
const KEY_STEP_PX = 16
const KEY_STEP_SHIFT_PX = 64

const store = useConsoleStore()
const chat = useChatStore()
const terminals = useTerminalStore()
const dock = useChatDockStore()
const { selectedTask, selectedTaskSteps } = storeToRefs(store)
const { activeChatId, activeTimeline } = storeToRefs(chat)
const { run } = useAsyncAction()

const composer = ref<InstanceType<typeof ChatComposer> | null>(null)
const sendError = ref<string | null>(null)
const answerError = ref<string | null>(null)
const answeredRequestId = ref<string | null>(null)
const stopRequested = ref(false)

const session = computed<Terminal | null>(
  () => terminals.terminals.find((terminal) => terminal.id === activeChatId.value) ?? null
)
const others = computed(() => chat.liveChats.filter((terminal) => terminal.id !== activeChatId.value))
const { connected, reconnecting } = useChatEvents(activeChatId)

const status = computed(() =>
  chatStatus({
    timeline: activeTimeline.value,
    session: session.value,
    connected: connected.value,
    reconnecting: reconnecting.value
  })
)
const pending = computed(() => activeTimeline.value.pending)
const followingNext = computed(() => (activeChatId.value ? chat.isFollowing(activeChatId.value) : false))
const folder = computed(() => selectedTask.value?.projectRepoFolder ?? null)
const sessionSteps = computed(() =>
  session.value?.taskId && session.value.taskId === selectedTask.value?.id
    ? selectedTaskSteps.value.filter((step) => step.state !== 'DRAFT')
    : []
)
const answering = computed(
  () => chat.answering || (pending.value !== null && answeredRequestId.value === pending.value.requestId)
)
const full = computed(() => dock.mode === 'full')
const sized = computed(() => dock.mode === 'inline' || dock.mode === 'overlay')

const rootClass = computed(() => {
  if (dock.mode === 'full') return 'absolute inset-0 z-(--z-overlay)'
  if (dock.mode === 'overlay') return 'absolute inset-y-0 right-0 z-(--z-overlay) shadow-modal'
  return 'relative shrink-0'
})
const rootStyle = computed(() => (sized.value ? { width: `${dock.shownWidth}px` } : undefined))

useAttentionTitle(() => status.value === 'waiting')

function returnFocusToConsole(event: KeyboardEvent): void {
  if (event.key !== 'Escape') return
  document.getElementById('note')?.focus()
}

function selectTask(taskId: TaskId): void {
  store.selectTask(taskId)
}

function switchTo(id: TerminalId): void {
  const target = terminals.terminals.find((terminal) => terminal.id === id)
  chat.select(id)
  if (target?.taskId) store.selectTask(target.taskId)
}

async function send(text: string): Promise<void> {
  const id = activeChatId.value
  if (!id) return
  sendError.value = null
  try {
    await chat.send(id, text)
    composer.value?.clear()
  } catch (error) {
    sendError.value = failureMessage(error)
  }
}

async function stopTurn(): Promise<void> {
  const id = activeChatId.value
  if (!id) return
  sendError.value = null
  stopRequested.value = true
  try {
    await chat.stop(id)
  } catch (error) {
    stopRequested.value = false
    sendError.value = failureMessage(error)
  }
}

async function answer(allowed: boolean): Promise<void> {
  const id = activeChatId.value
  const request = pending.value
  if (!id || !request) return
  answerError.value = null
  try {
    if (allowed) await chat.allow(id)
    else await chat.deny(id)
    answeredRequestId.value = request.requestId
  } catch (error) {
    answerError.value = failureMessage(error)
  }
}

async function stopSession(): Promise<void> {
  const id = session.value?.id
  if (!id) return
  await run(() => terminals.close(id))
  chat.select(null)
}

async function openHere(): Promise<void> {
  const task = selectedTask.value
  if (!task || !folder.value) return
  const opened = await run(
    () =>
      terminals.openForTask(task.id, {
        skipPermissions: skipsPermissions(),
        model: preferredModel(),
        effort: preferredEffort(),
        kind: 'CHAT'
      }),
    'Chat session opened'
  )
  if (opened) chat.select(opened.id)
}

function startResize(event: PointerEvent): void {
  const handle = event.currentTarget as HTMLElement
  const startX = event.clientX
  const startWidth = dock.shownWidth
  handle.setPointerCapture?.(event.pointerId)

  function move(moved: PointerEvent): void {
    dock.setWidth(Math.min(dock.maxWidth, startWidth + startX - moved.clientX))
  }

  function end(): void {
    handle.removeEventListener('pointermove', move)
    handle.removeEventListener('pointerup', end)
    handle.removeEventListener('pointercancel', end)
  }

  handle.addEventListener('pointermove', move)
  handle.addEventListener('pointerup', end)
  handle.addEventListener('pointercancel', end)
}

function resizeWithKeys(event: KeyboardEvent): void {
  if (event.key !== 'ArrowLeft' && event.key !== 'ArrowRight') return
  event.preventDefault()
  const step = event.shiftKey ? KEY_STEP_SHIFT_PX : KEY_STEP_PX
  dock.nudge(event.key === 'ArrowLeft' ? step : -step)
}

watch(
  () => selectedTask.value?.id ?? null,
  (taskId) => chat.showTaskSession(taskId),
  { immediate: true }
)

watch(activeChatId, () => {
  sendError.value = null
  answerError.value = null
  stopRequested.value = false
})

watch(
  () => activeTimeline.value.turn,
  (turn) => {
    if (turn !== 'working') stopRequested.value = false
  }
)

watch(pending, async (request, previous) => {
  answerError.value = null
  if (request || !previous) return
  await nextTick()
  composer.value?.focus()
})

onMounted(async () => {
  await nextTick()
  if (!pending.value) composer.value?.focus()
})
</script>

<template>
  <section
    class="flex min-h-0 flex-col border-l border-border bg-canvas"
    :class="rootClass"
    :style="rootStyle"
    aria-label="Chat"
    data-testid="chat-window"
    :data-mode="dock.mode"
    @keydown="returnFocusToConsole"
  >
    <div
      v-if="sized"
      class="focus-ring absolute inset-y-0 left-0 z-10 w-1.5 -translate-x-1/2 cursor-col-resize touch-none hover:bg-accent/40"
      role="separator"
      aria-orientation="vertical"
      aria-label="Resize the chat window"
      tabindex="0"
      :aria-valuenow="dock.shownWidth"
      :aria-valuemin="MIN_WIDTH"
      :aria-valuemax="dock.maxWidth"
      data-testid="chat-resize"
      @pointerdown.prevent="startResize"
      @keydown="resizeWithKeys"
      @dblclick="dock.resetWidth()"
    />

    <template v-if="session">
      <ChatHeader
        :session="session"
        :status="status"
        :others="others"
        :steps="sessionSteps"
        :expanded="full"
        @select-task="selectTask"
        @switch="switchTo"
        @toggle-expand="dock.toggleExpanded()"
        @hide="dock.hide()"
        @stop="stopSession"
      />

      <p
        v-if="status === 'reconnecting'"
        class="shrink-0 border-b border-warn/40 bg-warn-soft px-4 py-1.5 text-caption text-warn"
        role="status"
        data-testid="chat-reconnecting"
      >
        Reconnecting…
      </p>

      <ChatTimeline :items="activeTimeline.items" :following-next="followingNext" />

      <ChatApprovalCard
        v-if="pending"
        :key="pending.requestId"
        :request="pending"
        :busy="answering"
        :error="answerError"
        @allow="answer(true)"
        @deny="answer(false)"
      />

      <ChatComposer
        ref="composer"
        :state="composerState(status)"
        :sending="chat.sending"
        :stopping="stopRequested"
        :error="sendError"
        :following-next="followingNext"
        @send="send"
        @stop="stopTurn"
      />
    </template>

    <div v-else class="grid min-h-0 flex-1 place-items-center px-6">
      <AppEmptyState
        title="No chat session on this task"
        description="Claude runs here without a terminal: the conversation, its tool calls and the approvals it asks for appear in this window."
      >
        <AppButton v-if="selectedTask && folder" variant="primary" data-testid="chat-open" @click="openHere">
          Run here
        </AppButton>
        <RouterLink
          v-else-if="selectedTask"
          :to="`/projects/${selectedTask.projectId}`"
          class="focus-ring inline-flex items-center gap-1.5 rounded-control border border-warn/40 bg-warn-soft px-3 py-1.5 text-caption font-medium text-warn"
          data-testid="chat-set-folder"
        >
          Set this project's folder on its page first
          <span aria-hidden="true">&rarr;</span>
        </RouterLink>
      </AppEmptyState>
    </div>
  </section>
</template>
