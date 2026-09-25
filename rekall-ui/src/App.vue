<script setup lang="ts">
import { defineAsyncComponent, onMounted, onUnmounted, ref } from 'vue'
import { storeToRefs } from 'pinia'
import AnchorBar from '@/components/console/AnchorBar.vue'
import DescriptionPane from '@/components/console/DescriptionPane.vue'
import NavigatorPane from '@/components/console/NavigatorPane.vue'
import NoteListPane from '@/components/console/NoteListPane.vue'
import NotePane from '@/components/console/NotePane.vue'
import NoteComposerPane from '@/components/console/NoteComposerPane.vue'
import NotePlacementsPane from '@/components/console/NotePlacementsPane.vue'
import StepsPane from '@/components/console/StepsPane.vue'
import WrapupPane from '@/components/console/WrapupPane.vue'
import SettingsPanel from '@/components/settings/SettingsPanel.vue'
import TagsPanel from '@/components/settings/TagsPanel.vue'
import RunQueuePanel from '@/components/queue/RunQueuePanel.vue'
import AppToaster from '@/components/ui/AppToaster.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useRunQueueStore } from '@/stores/runQueue.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useModalGate } from '@/composables/useModalGate'
import { useStepStream } from '@/composables/useStepStream'
import { useClaimNotifications } from '@/composables/useClaimNotifications'
import type { TaskStatus } from '@/model/catalog'

// xterm and its stylesheet are only parsed once a terminal is opened: most sessions never do,
// and the console's first paint should not pay for them.
const TerminalPane = defineAsyncComponent(() => import('@/components/console/TerminalPane.vue'))

const store = useConsoleStore()
const { selectedTaskId, navMode, paneFocus, noteComposerOpen, reviewQueue, isLoading } = storeToRefs(store)
const runQueue = useRunQueueStore()
const { panelOpen: runQueueOpen } = storeToRefs(runQueue)
const { run } = useAsyncAction()
const { isModalOpen } = useModalGate()

useStepStream(
  (taskId, steps) => store.applyStepEvent(taskId, steps),
  (review) => store.applyTaskReview(review),
  (event) => store.applyWrapupEvent(event),
  (reference) => store.applyCommitReference(reference),
  (queue) => runQueue.apply(queue),
  () => void store.applyNoteWritten()
)

useClaimNotifications(reviewQueue, isLoading)

const STATUS_BY_KEY: Record<string, TaskStatus> = {
  '1': 'IN_PROGRESS',
  '2': 'TODO',
  '3': 'BLOCKED',
  '4': 'DONE'
}

const anchorBar = ref<InstanceType<typeof AnchorBar> | null>(null)
const navigator = ref<InstanceType<typeof NavigatorPane> | null>(null)
const descriptionPane = ref<InstanceType<typeof DescriptionPane> | null>(null)
const notePane = ref<InstanceType<typeof NotePane> | null>(null)
const settingsOpen = ref(false)
const tagsOpen = ref(false)

/**
 * Browsing tasks, a note lands on the task in view with no questions. Browsing notes, the task in
 * view is not on screen, so the composer asks where the note goes.
 */
async function newNote(): Promise<void> {
  if (navMode.value === 'notes') {
    store.openNoteComposer()
    return
  }
  const taskId = selectedTaskId.value
  if (!taskId) return
  await run(() => store.createNote([taskId]), 'Note created')
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

  if (key === 'n') {
    event.preventDefault()
    void newNote()
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
    if (paneFocus.value === 'description') descriptionPane.value?.toggleMode()
    else if (paneFocus.value === 'note') notePane.value?.toggleMode()
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

  if (key === 'b') {
    event.preventDefault()
    store.setNavMode(navMode.value === 'tasks' ? 'notes' : 'tasks')
    return
  }

  if (key === 't') {
    event.preventDefault()
    navigator.value?.beginCreate()
    return
  }

  if (key === 'e') {
    event.preventDefault()
    navigator.value?.editSelected()
    return
  }

  if (key === 'j' || key === 'k') {
    event.preventDefault()
    const list = navMode.value === 'tasks' ? store.visibleTasks : store.visibleDocuments
    const currentId = navMode.value === 'tasks' ? store.selectedTaskId : store.selectedDocId
    const at = list.findIndex((item) => item.id === currentId)
    const next = list[at + (key === 'j' ? 1 : -1)]
    if (!next) return
    if (navMode.value === 'tasks') store.selectTask(next.id as never)
    else store.selectDocument(next.id as never)
    return
  }

  const status = STATUS_BY_KEY[event.key]
  if (status && navMode.value === 'tasks' && selectedTaskId.value) {
    event.preventDefault()
    void run(() => store.setTaskStatus(selectedTaskId.value!, status))
  }
}

onMounted(() => {
  window.addEventListener('keydown', onKeydown)
  // The beacon needs the queue from the first paint; the SSE feed only carries changes.
  void runQueue.load().catch(() => undefined)
})
onUnmounted(() => window.removeEventListener('keydown', onKeydown))
</script>

<template>
  <div class="flex h-full flex-col">
    <a
      href="#note"
      class="focus-ring sr-only focus:not-sr-only focus:fixed focus:left-4 focus:top-4 focus:z-(--z-toast) focus:rounded-[var(--radius-control)] focus:bg-accent focus:px-4 focus:py-2 focus:text-[13px] focus:font-semibold focus:text-accent-ink"
    >
      Skip to the editor
    </a>

    <AnchorBar
      ref="anchorBar"
      @new-note="newNote"
      @open-settings="settingsOpen = true"
      @open-tags="tagsOpen = true"
    />

    <div class="flex min-h-0 flex-1">
      <NavigatorPane ref="navigator" />
      <Transition name="pane" mode="out-in">
        <NoteListPane v-if="navMode === 'tasks'" />
        <NoteComposerPane v-else-if="noteComposerOpen" />
        <NotePlacementsPane v-else />
      </Transition>
      <!-- A place focus is sent to (the skip link, a pane shortcut), never a control, so it
           draws no ring: the pane that appears in it is what says where you are. -->
      <div id="note" class="flex min-h-0 min-w-0 flex-1 outline-none" tabindex="-1">
        <Transition name="pane" mode="out-in">
          <WrapupPane v-if="paneFocus === 'wrapup'" />
          <DescriptionPane v-else-if="paneFocus === 'description'" ref="descriptionPane" />
          <StepsPane v-else-if="paneFocus === 'steps'" />
          <TerminalPane v-else-if="paneFocus === 'terminal'" />
          <NotePane v-else ref="notePane" />
        </Transition>
      </div>
    </div>

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

