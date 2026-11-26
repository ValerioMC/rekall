<script setup lang="ts">
import { computed, defineAsyncComponent, onMounted, onUnmounted, watch } from 'vue'
import { storeToRefs } from 'pinia'
import { useRoute, useRouter } from 'vue-router'
import ConsoleHeader from '@/components/console/ConsoleHeader.vue'
import RecordDialog from '@/features/catalog/components/RecordDialog.vue'
import ReviewPage from '@/features/review/components/ReviewPage.vue'
import TaskPage from '@/features/task/components/TaskPage.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import { useDeviationsStore } from '@/features/deviations/store/deviations.store'
import { useRunQueueStore } from '@/features/queue/store/runQueue.store'
import { useQueueSessionSync } from '@/features/queue/composables/useQueueSessionSync'
import { useModalGate } from '@/composables/useModalGate'
import { useContextFillStream } from '@/features/context-fill/composables/useContextFillStream'
import { useStepStream } from '@/features/steps/composables/useStepStream'
import { useClaimNotifications } from '@/composables/useClaimNotifications'
import { useConsoleLocation } from '@/composables/useConsoleLocation'
import { useTaskEditor } from '@/features/catalog/composables/useTaskEditor'

// xterm and its stylesheet are only parsed once a terminal is opened: most sessions never do,
// and the console's first paint should not pay for them.
const TerminalPane = defineAsyncComponent(() => import('@/features/claude/components/TerminalPane.vue'))
const ChatPane = defineAsyncComponent(() => import('@/features/chat/components/ChatPane.vue'))

const store = useConsoleStore()
const route = useRoute()
const taskEditor = useTaskEditor()
const onTask = computed(() => route.name === 'task')
const onReview = computed(() => route.name === 'review')
const { selectedTaskId, selectedTask, paneFocus, chatInPane, projectChatId, reviewQueue, isLoading } =
  storeToRefs(store)
const terminals = useTerminalStore()
const runQueue = useRunQueueStore()
const deviations = useDeviationsStore()
const { isModalOpen } = useModalGate()
useStepStream(
  (taskId, steps) => store.applyStepEvent(taskId, steps),
  (review) => store.applyTaskReview(review),
  (event) => store.applyWrapupEvent(event),
  (reference) => store.applyCommitReference(reference),
  (queue) => runQueue.apply(queue),
  () => void store.applyNoteWritten(),
  () => void store.applyTaskCreated(),
  (taskId) => void deviations.load(taskId).catch(() => undefined),
  (taskId) => void store.loadFeedback(taskId).catch(() => undefined),
  (taskId) => void store.loadDecisions(taskId).catch(() => undefined)
)

useContextFillStream((event) => store.applyContextFillEvent(event))

useQueueSessionSync()
useClaimNotifications(reviewQueue, isLoading)
useConsoleLocation()

const router = useRouter()

/** `/console` hosts a full-size session and nothing else: a task shown on any other pane is on its page. */
const showsTerminal = computed(() => paneFocus.value === 'terminal' && selectedTask.value !== null)

/** A chat on no task selects nothing, so the header is handed that chat's anchors. */
const chatOnNoTaskAnchor = computed(() => {
  if (!chatInPane.value || selectedTaskId.value !== null) return undefined
  return terminals.terminals.find((session) => session.id === projectChatId.value)?.anchors
})

function leaveConsole(): void {
  void router.replace(selectedTaskId.value ? `/task/${selectedTaskId.value}` : '/projects')
}

watch(
  [() => route.name, () => showsTerminal.value || chatInPane.value, () => store.isLoading],
  ([name, session, loading]) => {
    if (name === 'console' && !session && !loading) leaveConsole()
  },
  { immediate: true }
)

function onKeydown(event: KeyboardEvent): void {
  if (isModalOpen.value > 0) return

  const target = event.target as HTMLElement | null
  const typing =
    !!target &&
    (/^(INPUT|TEXTAREA|SELECT)$/.test(target.tagName) || target.isContentEditable)

  // Shortcuts are bare keys; a Cmd/Ctrl/Alt chord belongs to the browser.
  if (event.metaKey || event.ctrlKey || event.altKey) return

  if (typing) return

  const key = event.key.toLowerCase()
  if (key === 'q') {
    event.preventDefault()
    runQueue.openPanel()
  } else if (key === 't') {
    event.preventDefault()
    taskEditor.beginCreate()
  } else if (key === 'e' && onTask.value) {
    event.preventDefault()
    taskEditor.editSelected()
  }
}

onMounted(() => {
  window.addEventListener('keydown', onKeydown)
})
onUnmounted(() => {
  window.removeEventListener('keydown', onKeydown)
})
</script>

<template>
  <div class="flex min-h-0 flex-1 flex-col">
    <a
      href="#note"
      class="focus-ring sr-only focus:not-sr-only focus:fixed focus:left-4 focus:top-4 focus:z-(--z-toast) focus:rounded-control focus:bg-accent focus:px-4 focus:py-2 focus:text-body focus:font-semibold focus:text-accent-ink"
    >
      Skip to the editor
    </a>

    <ConsoleHeader :anchor="chatOnNoTaskAnchor" />
    <div class="flex min-h-0 flex-1">
      <TaskPage v-if="onTask" />
      <ReviewPage v-else-if="onReview" />
      <div v-else class="flex min-h-0 min-w-0 flex-1" data-testid="console-terminal">
        <!-- A place focus is sent to (the skip link), never a control, so it draws no ring: the pane
             that appears in it is what says where you are. -->
        <div id="note" class="flex min-h-0 min-w-0 flex-1 outline-none" tabindex="-1">
          <TerminalPane v-if="showsTerminal" />
          <ChatPane v-else-if="chatInPane" />
        </div>
      </div>
    </div>

    <Transition name="dialog">
      <RecordDialog v-if="taskEditor.editing.value" :draft="taskEditor.editing.value" @close="taskEditor.close()" />
    </Transition>
    <Transition name="dialog">
    </Transition>
  </div>
</template>

