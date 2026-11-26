<script setup lang="ts">
import { computed, watch } from 'vue'
import { storeToRefs } from 'pinia'
import ChatConversation from './ChatConversation.vue'
import ChatHeader from './ChatHeader.vue'
import ChatRunHere from './ChatRunHere.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useShownChat } from '@/features/chat/composables/useShownChat'
import { MIN_WIDTH, useChatDockStore } from '@/features/chat/store/chatDock.store'
import { useChatStore } from '@/features/chat/store/chat.store'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import { useConsoleStore } from '@/stores/console.store'
import type { Terminal } from '@/features/claude/model/terminal'
import type { TaskId, TerminalId } from '@/model/branded'

/** The chat session as a panel docked beside a page, on the task the session runs on. */
const KEY_STEP_PX = 16
const KEY_STEP_SHIFT_PX = 64

const store = useConsoleStore()
const chat = useChatStore()
const terminals = useTerminalStore()
const dock = useChatDockStore()
const { selectedTask } = storeToRefs(store)
const { activeTimeline } = storeToRefs(chat)
const { run } = useAsyncAction()
const { session, others, status, sessionSteps } = useShownChat()

const full = computed(() => dock.placement === 'full')
const sized = computed(() => dock.placement !== 'full')

const rootClass = computed(() => {
  if (dock.placement === 'full') return 'absolute inset-0 z-(--z-overlay)'
  if (dock.placement === 'overlay') return 'absolute inset-y-0 right-0 z-(--z-overlay) shadow-modal'
  return 'relative shrink-0'
})
const rootStyle = computed(() => (sized.value ? { width: `${dock.shownWidth}px` } : undefined))

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

async function stopSession(): Promise<void> {
  const id = session.value?.id
  if (!id) return
  await run(() => terminals.close(id))
  chat.select(null)
}

function showOpened(opened: Terminal): void {
  if (opened.kind === 'CHAT') {
    chat.select(opened.id)
    return
  }
  dock.hide()
  terminals.float(opened.id)
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
</script>

<template>
  <section
    class="flex min-h-0 flex-col border-l border-border bg-canvas"
    :class="rootClass"
    :style="rootStyle"
    aria-label="Chat"
    data-testid="chat-window"
    :data-mode="dock.placement"
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
        :context="activeTimeline.context"
        :cost="activeTimeline.cost"
        @select-task="selectTask"
        @switch="switchTo"
        @toggle-expand="dock.toggleExpanded()"
        @hide="dock.hide()"
        @stop="stopSession"
      />

      <ChatConversation :session="session" :status="status" />
    </template>

    <ChatRunHere v-else @opened="showOpened" />
  </section>
</template>
