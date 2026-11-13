<script setup lang="ts">
import { computed, defineAsyncComponent, onMounted, onUnmounted, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import { useRoute } from 'vue-router'
import AnchorBar from '@/components/console/AnchorBar.vue'
import DescriptionPane from '@/features/catalog/components/DescriptionPane.vue'
import DeviationPane from '@/features/deviations/components/DeviationPane.vue'
import EpicPane from '@/features/catalog/components/epic/EpicPane.vue'
import RecordDialog from '@/features/catalog/components/RecordDialog.vue'
import NoteListPane from '@/features/notes/components/NoteListPane.vue'
import NotePane from '@/features/notes/components/NotePane.vue'
import StepsPane from '@/features/steps/components/StepsPane.vue'
import TaskPage from '@/features/task/components/TaskPage.vue'
import WorkPage from '@/features/work/components/WorkPage.vue'
import WrapupPane from '@/features/wrapup/components/WrapupPane.vue'
import SettingsPanel from '@/features/settings/components/SettingsPanel.vue'
import TagsPanel from '@/features/tags/components/TagsPanel.vue'
import RunQueuePanel from '@/features/queue/components/RunQueuePanel.vue'
import AppToaster from '@/components/ui/AppToaster.vue'
import ChatReopenTag from '@/features/chat/components/ChatReopenTag.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useDeviationsStore } from '@/features/deviations/store/deviations.store'
import { useClaudeConnectionStore } from '@/features/claude/store/claudeConnection.store'
import { useChatStore } from '@/features/chat/store/chat.store'
import { useChatDockStore } from '@/features/chat/store/chatDock.store'
import { useChatDockArea } from '@/features/chat/composables/useChatDockArea'
import { useChatWindowShown } from '@/features/chat/composables/useChatWindowShown'
import { preferredSessionKind } from '@/common/config/claude-launch'
import { useRunQueueStore } from '@/features/queue/store/runQueue.store'
import { useQueueSessionSync } from '@/features/queue/composables/useQueueSessionSync'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useModalGate } from '@/composables/useModalGate'
import { useStepStream } from '@/features/steps/composables/useStepStream'
import { useClaimNotifications } from '@/composables/useClaimNotifications'
import { useConsoleLocation } from '@/composables/useConsoleLocation'
import { useTaskEditor } from '@/features/catalog/composables/useTaskEditor'
import { isEpic } from '@/features/catalog/model/catalog'
import { type TaskStatus } from '@/features/catalog/model/task-status'

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
const { selectedTaskId, selectedTask, paneFocus, reviewQueue, isLoading } = storeToRefs(store)
const runQueue = useRunQueueStore()
const deviations = useDeviationsStore()
const { panelOpen: runQueueOpen } = storeToRefs(runQueue)
const { run } = useAsyncAction()
const { isModalOpen } = useModalGate()
const chat = useChatStore()
const chatDock = useChatDockStore()
const noteArea = ref<HTMLElement | null>(null)
useChatDockArea(noteArea, true)

const chatWindowShown = useChatWindowShown()

function chatOwnsTerminalKey(): boolean {
  return chat.sessionFor(selectedTaskId.value) !== null || preferredSessionKind() === 'chat'
}

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

const STATUS_BY_KEY: Record<string, TaskStatus> = {
  '1': 'IN_PROGRESS',
  '2': 'TODO',
  '3': 'BLOCKED',
  '4': 'DONE',
  '5': 'BACKLOG'
}

const anchorBar = ref<InstanceType<typeof AnchorBar> | null>(null)
const descriptionPane = ref<InstanceType<typeof DescriptionPane> | null>(null)
const epicPane = ref<InstanceType<typeof EpicPane> | null>(null)
const notePane = ref<InstanceType<typeof NotePane> | null>(null)
const noteList = ref<InstanceType<typeof NoteListPane> | null>(null)
const deviationPane = ref<InstanceType<typeof DeviationPane> | null>(null)
const settingsOpen = ref(false)
const claudeConnection = useClaudeConnectionStore()
const tagsOpen = ref(false)

function onWorkKey(key: string, event: KeyboardEvent): void {
  if (key === 'q') {
    event.preventDefault()
    runQueue.openPanel()
  } else if (key === 't') {
    event.preventDefault()
    taskEditor.beginCreate()
  }
}

function onKeydown(event: KeyboardEvent): void {
  if (isModalOpen.value > 0) return

  const target = event.target as HTMLElement | null
  const typing =
    !!target &&
    (/^(INPUT|TEXTAREA|SELECT)$/.test(target.tagName) || target.isContentEditable)

  if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') {
    event.preventDefault()
    anchorBar.value?.focus()
    return
  }

  // Shortcuts are bare keys; a Cmd/Ctrl/Alt chord belongs to the browser.
  if (event.metaKey || event.ctrlKey || event.altKey) return

  if (typing) return

  const key = event.key.toLowerCase()

  if (onWork.value || onTask.value) {
    onWorkKey(key, event)
    return
  }

  if (paneFocus.value === 'deviation') {
    if (event.key === 'Escape') {
      event.preventDefault()
      store.closeDeviation()
      return
    }
    if (event.key === '[' || event.key === ']') {
      event.preventDefault()
      deviationPane.value?.step(event.key === ']' ? 1 : -1)
      return
    }
  }

  if (key === 'n') {
    event.preventDefault()
    void noteList.value?.addInformation()
    return
  }

  if (key === 'w' || key === 'd' || key === 's') {
    event.preventDefault()
    if (key === 'w') store.toggleWrapup()
    else if (key === 's') store.toggleSteps()
    else store.toggleDescription()
    document.getElementById('note')?.focus()
    return
  }

  if (key === 'r') {
    event.preventDefault()
    if (paneFocus.value === 'description') (epicPane.value ?? descriptionPane.value)?.toggleMode()
    else if (paneFocus.value === 'note') notePane.value?.toggleMode()
    return
  }

  if (key === 'c' && chatOwnsTerminalKey()) {
    event.preventDefault()
    if (event.shiftKey) {
      chatDock.open(selectedTaskId.value)
      chatDock.toggleExpanded()
    } else if (chatWindowShown.value) chatDock.hide()
    else chatDock.open(selectedTaskId.value)
    return
  }

  if (key === 'c') {
    event.preventDefault()
    store.toggleTerminal()
    document.getElementById('note')?.focus()
    return
  }

  if (key === 'q') {
    event.preventDefault()
    runQueue.openPanel()
    return
  }

  if (key === 't') {
    event.preventDefault()
    taskEditor.beginCreate()
    return
  }

  if (key === 'e') {
    event.preventDefault()
    taskEditor.editSelected()
    return
  }

  if (key === 'j' || key === 'k') {
    event.preventDefault()
    const list = store.visibleTasks
    const at = list.findIndex((item) => item.id === selectedTaskId.value)
    const next = list[at + (key === 'j' ? 1 : -1)]
    if (next) store.selectTask(next.id)
    return
  }

  const status = STATUS_BY_KEY[event.key]
  if (status && selectedTaskId.value) {
    event.preventDefault()
    void run(() => store.setTaskStatus(selectedTaskId.value!, status))
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
// Settings is where the registration gets repaired or changed by hand, so read it again on leaving.
watch(settingsOpen, (open) => {
  if (!open) void refreshClaudeConnection()
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
      ref="anchorBar"
      @open-settings="settingsOpen = true"
      @open-tags="tagsOpen = true"
    />

    <div class="flex min-h-0 flex-1">
      <WorkPage v-if="onWork" @new-task="taskEditor.beginCreate()" />
      <TaskPage v-else-if="onTask" />
      <template v-else>
      <NoteListPane ref="noteList" />
      <!-- A place focus is sent to (the skip link, a pane shortcut), never a control, so it
           draws no ring: the pane that appears in it is what says where you are. -->
      <div ref="noteArea" class="relative flex min-h-0 min-w-0 flex-1">
        <div id="note" class="flex min-h-0 min-w-0 flex-1 outline-none" tabindex="-1">
          <Transition name="pane" mode="out-in">
            <WrapupPane v-if="paneFocus === 'wrapup'" />
            <EpicPane v-else-if="paneFocus === 'description' && selectedTask && isEpic(selectedTask)" ref="epicPane" />
            <DescriptionPane v-else-if="paneFocus === 'description'" ref="descriptionPane" />
            <StepsPane v-else-if="paneFocus === 'steps' && !(selectedTask && isEpic(selectedTask))" />
            <DeviationPane v-else-if="paneFocus === 'deviation' && !(selectedTask && isEpic(selectedTask))" ref="deviationPane" />
            <TerminalPane v-else-if="paneFocus === 'terminal' && !(selectedTask && isEpic(selectedTask))" />
            <NotePane v-else ref="notePane" />
          </Transition>
        </div>
        <Transition name="pane">
          <ChatWindow v-if="chatWindowShown" />
        </Transition>
        <ChatReopenTag />
      </div>
      </template>
    </div>

    <Transition name="dialog">
      <RecordDialog v-if="taskEditor.editing.value" :draft="taskEditor.editing.value" @close="taskEditor.close()" />
    </Transition>
    <Transition name="dialog">
      <SettingsPanel v-if="settingsOpen" @close="settingsOpen = false" />
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

