<script setup lang="ts">
import { computed, defineAsyncComponent, onMounted, onUnmounted, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import { useRoute, useRouter } from 'vue-router'
import AnchorBar from '@/components/console/AnchorBar.vue'
import RecordDialog from '@/features/catalog/components/RecordDialog.vue'
import ReviewPage from '@/features/review/components/ReviewPage.vue'
import TaskPage from '@/features/task/components/TaskPage.vue'
import WorkPage from '@/features/work/components/WorkPage.vue'
import TagsPanel from '@/features/tags/components/TagsPanel.vue'
import RunQueuePanel from '@/features/queue/components/RunQueuePanel.vue'
import AppToaster from '@/components/ui/AppToaster.vue'
import ChatReopenTag from '@/features/chat/components/ChatReopenTag.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useDeviationsStore } from '@/features/deviations/store/deviations.store'
import { useClaudeConnectionStore } from '@/features/claude/store/claudeConnection.store'
import { useChatDockArea } from '@/features/chat/composables/useChatDockArea'
import { useChatWindowShown } from '@/features/chat/composables/useChatWindowShown'
import { useRunQueueStore } from '@/features/queue/store/runQueue.store'
import { useQueueSessionSync } from '@/features/queue/composables/useQueueSessionSync'
import { useModalGate } from '@/composables/useModalGate'
import { useStepStream } from '@/features/steps/composables/useStepStream'
import { useClaimNotifications } from '@/composables/useClaimNotifications'
import { useConsoleLocation } from '@/composables/useConsoleLocation'
import { useTaskEditor } from '@/features/catalog/composables/useTaskEditor'

// xterm and its stylesheet are only parsed once a terminal is opened: most sessions never do,
// and the console's first paint should not pay for them.
const TerminalPane = defineAsyncComponent(() => import('@/features/claude/components/TerminalPane.vue'))
// The chat window loads the same way: nothing of it is fetched before a chat session exists.
const ChatWindow = defineAsyncComponent(() => import('@/features/chat/components/ChatWindow.vue'))

const store = useConsoleStore()
const route = useRoute()
const taskEditor = useTaskEditor()
const onWork = computed(() => route.name === 'work' || route.name === 'work-board')
const onTask = computed(() => route.name === 'task')
const onReview = computed(() => route.name === 'review')
const { selectedTaskId, selectedTask, paneFocus, reviewQueue, isLoading } = storeToRefs(store)
const runQueue = useRunQueueStore()
const deviations = useDeviationsStore()
const { panelOpen: runQueueOpen } = storeToRefs(runQueue)
const { isModalOpen } = useModalGate()
const noteArea = ref<HTMLElement | null>(null)
useChatDockArea(noteArea, true)

const chatWindowShown = useChatWindowShown()

useStepStream(
  (taskId, steps) => store.applyStepEvent(taskId, steps),
  (review) => store.applyTaskReview(review),
  (event) => store.applyWrapupEvent(event),
  (reference) => store.applyCommitReference(reference),
  (queue) => runQueue.apply(queue),
  () => void store.applyNoteWritten(),
  () => void store.applyTaskCreated(),
  (taskId) => void deviations.load(taskId).catch(() => undefined),
  (taskId) => void store.loadFeedback(taskId).catch(() => undefined)
)

useQueueSessionSync()
useClaimNotifications(reviewQueue, isLoading)
useConsoleLocation()

const claudeConnection = useClaudeConnectionStore()
const router = useRouter()

/** `/console` hosts the terminal and nothing else: a task shown on any other pane is on its page. */
const showsTerminal = computed(() => paneFocus.value === 'terminal' && selectedTask.value !== null)

watch(
  [() => route.name, showsTerminal, () => store.isLoading, chatWindowShown],
  ([name, terminal, loading, chatShown]) => {
    if (name !== 'console' || terminal || loading || chatShown) return
    void router.replace(selectedTaskId.value ? `/task/${selectedTaskId.value}` : '/')
  },
  { immediate: true }
)

function openSettings(): void {
  void router.push('/settings')
}
const tagsOpen = ref(false)

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

/** The warning on the Settings key is a hint: a registration that cannot be read stays silent. */
async function refreshClaudeConnection(): Promise<void> {
  await claudeConnection.load().catch(() => undefined)
}

onMounted(() => {
  window.addEventListener('keydown', onKeydown)
  // The beacon needs the queue from the first paint; the SSE feed only carries changes.
  void runQueue.load().catch(() => undefined)
  void refreshClaudeConnection()
})
onUnmounted(() => window.removeEventListener('keydown', onKeydown))
</script>

<template>
  <div class="flex min-h-0 flex-1 flex-col">
    <a
      href="#note"
      class="focus-ring sr-only focus:not-sr-only focus:fixed focus:left-4 focus:top-4 focus:z-(--z-toast) focus:rounded-control focus:bg-accent focus:px-4 focus:py-2 focus:text-body focus:font-semibold focus:text-accent-ink"
    >
      Skip to the editor
    </a>

    <AnchorBar
      @open-settings="openSettings"
      @open-tags="tagsOpen = true"
    />

    <div class="flex min-h-0 flex-1">
      <WorkPage v-if="onWork" @new-task="taskEditor.beginCreate()" />
      <TaskPage v-else-if="onTask" />
      <ReviewPage v-else-if="onReview" />
      <div v-else ref="noteArea" class="relative flex min-h-0 min-w-0 flex-1" data-testid="console-terminal">
        <!-- A place focus is sent to (the skip link), never a control, so it draws no ring: the pane
             that appears in it is what says where you are. -->
        <div id="note" class="flex min-h-0 min-w-0 flex-1 outline-none" tabindex="-1">
          <TerminalPane v-if="showsTerminal" />
        </div>
        <Transition name="pane">
          <ChatWindow v-if="chatWindowShown" />
        </Transition>
        <ChatReopenTag />
      </div>
    </div>

    <Transition name="dialog">
      <RecordDialog v-if="taskEditor.editing.value" :draft="taskEditor.editing.value" @close="taskEditor.close()" />
    </Transition>
    <Transition name="dialog">
    </Transition>
    <Transition name="dialog">
      <TagsPanel v-if="tagsOpen" @close="tagsOpen = false" />
    </Transition>
    <Transition name="dialog">
      <RunQueuePanel v-if="runQueueOpen" @close="runQueue.closePanel()" />
    </Transition>
    <AppToaster />
  </div>
</template>

