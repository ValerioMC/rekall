<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
import { storeToRefs } from 'pinia'
import AnchorBar from '@/components/console/AnchorBar.vue'
import DescriptionPane from '@/components/console/DescriptionPane.vue'
import NavigatorPane from '@/components/console/NavigatorPane.vue'
import NoteListPane from '@/components/console/NoteListPane.vue'
import NotePane from '@/components/console/NotePane.vue'
import StepsPane from '@/components/console/StepsPane.vue'
import WrapupPane from '@/components/console/WrapupPane.vue'
import ClaudeSessionPane from '@/components/console/ClaudeSessionPane.vue'
import SettingsPanel from '@/components/settings/SettingsPanel.vue'
import AppToaster from '@/components/ui/AppToaster.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useModalGate } from '@/composables/useModalGate'
import { useStepStream } from '@/composables/useStepStream'
import type { TaskStatus } from '@/model/catalog'

const store = useConsoleStore()
const { selectedTaskId, navMode, paneFocus } = storeToRefs(store)
const { run } = useAsyncAction()
const { isModalOpen } = useModalGate()

useStepStream(
  (taskId, steps) => store.applyStepEvent(taskId, steps),
  (review) => store.applyTaskReview(review),
  (event) => store.applyWrapupEvent(event)
)

const STATUS_BY_KEY: Record<string, TaskStatus> = {
  '1': 'IN_PROGRESS',
  '2': 'TODO',
  '3': 'BLOCKED',
  '4': 'DONE'
}

const anchorBar = ref<InstanceType<typeof AnchorBar> | null>(null)
const navigator = ref<InstanceType<typeof NavigatorPane> | null>(null)
const settingsOpen = ref(false)

async function newNote(): Promise<void> {
  if (!selectedTaskId.value) return
  await run(() => store.createNote(selectedTaskId.value!), 'Note created')
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

  // Every other shortcut is a bare key, so a Cmd/Ctrl/Alt chord belongs to the
  // browser. Copy, paste, cut and select-all must reach it untouched.
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

  if (key === 'c') {
    event.preventDefault()
    store.toggleClaude()
    document.getElementById('note')?.focus()
    return
  }

  if (key === 'b') {
    event.preventDefault()
    navMode.value = navMode.value === 'tasks' ? 'notes' : 'tasks'
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

onMounted(() => window.addEventListener('keydown', onKeydown))
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

    <AnchorBar ref="anchorBar" @new-note="newNote" @open-settings="settingsOpen = true" />

    <div class="flex min-h-0 flex-1">
      <NavigatorPane ref="navigator" />
      <NoteListPane />
      <div id="note" class="flex min-h-0 min-w-0 flex-1" tabindex="-1">
        <WrapupPane v-if="paneFocus === 'wrapup'" />
        <DescriptionPane v-else-if="paneFocus === 'description'" />
        <StepsPane v-else-if="paneFocus === 'steps'" />
        <ClaudeSessionPane v-else-if="paneFocus === 'claude'" />
        <NotePane v-else />
      </div>
    </div>

    <SettingsPanel v-if="settingsOpen" @close="settingsOpen = false" />
    <AppToaster />
  </div>
</template>
