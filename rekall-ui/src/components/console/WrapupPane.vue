<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import AppButton from '@/components/ui/AppButton.vue'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import AppMarkdownEditor from '@/components/ui/AppMarkdownEditor.vue'
import AppModeToggle from '@/components/ui/AppModeToggle.vue'
import LaunchClaudeCodeButton from '@/components/claude/LaunchClaudeCodeButton.vue'
import WrapupHereDialog from '@/components/console/WrapupHereDialog.vue'
import RevisionHistoryButton from '@/components/console/RevisionHistoryButton.vue'
import TaskTitleRow from '@/components/console/TaskTitleRow.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useTerminalStore } from '@/stores/terminal.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { relativeTime } from '@/common/format/relative-time'
import { rkWrapupCommand } from '@/common/format/rk-command'
import { WRAPUP_AUTHOR_LABEL } from '@/model/catalog'

const store = useConsoleStore()
const terminals = useTerminalStore()
const { selectedTask, selectedWrapup, wrapupIsBehind } = storeToRefs(store)
const { run } = useAsyncAction()
const { run: runSend, isRunning: sendingWrapupHere } = useAsyncAction()

const liveTerminal = computed(() => terminals.terminalForTask(selectedTask.value?.id ?? null))
const wrapupHereOpen = ref(false)

const mode = ref<'write' | 'read'>('read')
const isConfirmingDelete = ref(false)

const isDrafting = ref(false)
const draft = ref('')
let saveTimer: ReturnType<typeof setTimeout> | null = null

watch(
  () => selectedTask.value?.id ?? null,
  () => {
    if (saveTimer) clearTimeout(saveTimer)
    draft.value = selectedWrapup.value?.bodyMarkdown ?? ''
    isDrafting.value = false
    mode.value = selectedWrapup.value ? 'read' : 'write'
  },
  { immediate: true }
)

// In read mode, adopt a body that lands over the SSE feed; in write mode the buffer belongs to
// the person typing, so leave it alone.
watch(
  () => selectedWrapup.value?.bodyMarkdown ?? '',
  (value) => {
    if (mode.value === 'write' || value === draft.value) return
    draft.value = value
  }
)

const writtenBy = computed(() =>
  selectedWrapup.value ? WRAPUP_AUTHOR_LABEL[selectedWrapup.value.writtenBy] : ''
)

function scheduleSave(): void {
  const task = selectedTask.value
  if (!task) return
  if (!draft.value.trim()) {
    store.saveState = 'saved'
    return
  }
  store.saveState = 'unsaved'
  if (saveTimer) clearTimeout(saveTimer)
  saveTimer = setTimeout(() => {
    void run(() => store.saveWrapupBody(task.id, draft.value))
  }, 700)
}

// A restore rewrote the wrapup on the server: drop any pending save of the old buffer and show
// what was restored.
function adoptRestored(): void {
  if (saveTimer) clearTimeout(saveTimer)
  saveTimer = null
  draft.value = selectedWrapup.value?.bodyMarkdown ?? ''
  isDrafting.value = false
  mode.value = 'read'
  store.saveState = 'saved'
}

function beginWriting(): void {
  isDrafting.value = true
  mode.value = 'write'
}

async function confirmDelete(): Promise<void> {
  const task = selectedTask.value
  if (!task) return
  await run(() => store.removeWrapup(task.id), 'Wrapup deleted')
  isConfirmingDelete.value = false
}

async function sendWrapupHere(message: string): Promise<void> {
  const task = selectedTask.value
  const terminal = liveTerminal.value
  if (!task || !terminal) return
  const sent = await runSend(
    () => terminals.sendCommand(terminal.id, rkWrapupCommand(task.anchor, message)),
    'Sent to the terminal.'
  )
  if (sent === null) return
  wrapupHereOpen.value = false
  store.openTerminal()
  terminals.select(terminal.id)
}
</script>

<template>
  <section class="flex min-h-0 flex-1 flex-col bg-canvas" aria-label="Wrapup">
    <p v-if="!selectedTask" class="px-9 py-12 text-body text-text-muted">
      Pick a task to see what it currently is.
    </p>

    <template v-else>
      <header class="flex shrink-0 items-start gap-3.5 border-b border-border px-5 py-3.5">
        <div class="min-w-0 flex-1">
          <p class="eyebrow flex items-center gap-1.5">
            <span class="h-2.5 w-[3px] shrink-0 rounded-full bg-accent" aria-hidden="true" />
            Wrapup
          </p>
          <TaskTitleRow :task="selectedTask" copy-testid="copy-wrapup-anchor" />
        </div>

        <LaunchClaudeCodeButton
          class="shrink-0"
          :anchors="selectedTask.anchor"
          :folder="selectedTask.projectRepoFolder"
          missing-hint="Set this project's folder on its page to open a session from it"
        />

        <AppButton
          v-if="liveTerminal"
          variant="quiet"
          size="sm"
          data-testid="wrapup-here"
          title="Type the wrapup command into the session already running here"
          @click="wrapupHereOpen = true"
        >
          <span
            class="session-caret shrink-0"
            :class="sendingWrapupHere && 'session-caret-busy'"
            aria-hidden="true"
          />
          Wrapup here
        </AppButton>

        <RevisionHistoryButton
          :task-id="selectedTask.id"
          :task-title="selectedTask.title"
          kind="WRAPUP"
          @restored="adoptRestored"
        />

        <div v-if="selectedWrapup" class="flex shrink-0 items-center gap-2">
          <AppModeToggle v-model="mode" />
          <AppButton variant="danger-quiet" size="sm" @click="isConfirmingDelete = true">Delete</AppButton>
        </div>
      </header>

      <div
        v-if="selectedWrapup"
        class="flex shrink-0 flex-wrap items-center gap-x-3 gap-y-1.5 border-b border-border bg-surface px-5 py-2.5"
      >
        <span class="inline-flex items-center gap-1.5 text-caption text-text-muted">
          <svg
            v-if="selectedWrapup.writtenBy === 'CLAUDE'"
            class="size-3 shrink-0 text-text-subtle"
            viewBox="0 0 12 12"
            aria-hidden="true"
          >
            <path
              d="M6 0.5v3M6 8.5v3M0.5 6h3M8.5 6h3M2.3 2.3l2.1 2.1M7.6 7.6l2.1 2.1M9.7 2.3 7.6 4.4M4.4 7.6l-2.1 2.1"
              stroke="currentColor"
              stroke-width="1"
              stroke-linecap="round"
            />
          </svg>
          <svg v-else class="size-3 shrink-0 text-text-subtle" viewBox="0 0 12 12" aria-hidden="true">
            <path
              d="M2 10 3 6.6l4.4-4.4a1.2 1.2 0 0 1 1.7 1.7L4.7 8.3 2 10Z"
              fill="none"
              stroke="currentColor"
              stroke-width="1"
              stroke-linejoin="round"
            />
          </svg>
          Written by
          <span class="font-medium text-text">{{ writtenBy }}</span>
          <span class="text-text-muted">
            &middot; {{ relativeTime(selectedWrapup.updatedAt) }}
          </span>
        </span>

        <span
          v-if="wrapupIsBehind > 0"
          class="inline-flex items-center gap-1.5 rounded-full bg-warn-soft px-2 py-0.5 text-caption text-warn"
        >
          <span class="size-[5px] shrink-0 rounded-full bg-warn" aria-hidden="true" />
          {{ wrapupIsBehind }} note{{ wrapupIsBehind === 1 ? ' is' : 's are' }} newer than this
        </span>
      </div>

      <div v-if="!selectedWrapup && !isDrafting" class="min-h-0 flex-1 overflow-y-auto">
        <div class="max-w-[560px] px-9 py-12">
          <p class="eyebrow">
            Wrapup
          </p>
          <h2 class="mb-1.5 mt-1.5 text-display font-semibold leading-tight tracking-[-0.025em] text-text">
            Nobody has said what this is yet
          </h2>
          <p class="mb-6 text-body leading-relaxed text-text-muted">
            One per task, replaced each time it is written. It describes the implementation as it
            stands — what exists, how it fits, what is still open — and never what changed along
            the way. That is what the notes are for.
          </p>

          <p class="text-small text-text-muted">
            Copy the anchor above, append <span class="font-mono text-anchor">wrapup</span>, and
            run it at the end of a session to have Claude write it, or
            <button
              class="focus-ring rounded-key text-accent underline decoration-accent/40 underline-offset-2 hover:decoration-accent"
              data-testid="write-wrapup-by-hand"
              @click="beginWriting"
            >
              write it yourself
            </button>
            . Claude replaces what you write the next time it runs that command.
          </p>
        </div>
      </div>

      <div v-else class="flex min-h-0 flex-1 flex-col">
        <p
          v-if="isDrafting && !selectedWrapup"
          class="shrink-0 border-b border-border bg-surface px-5 py-2 text-caption text-text-muted"
        >
          This saves itself. Describe the state, not the session.
        </p>
        <div class="min-h-0 flex-1 overflow-y-auto p-4">
          <Transition name="fade-quick" mode="out-in">
            <AppMarkdownEditor
              v-if="mode === 'write'"
              key="write"
              v-model="draft"
              height="100%"
              :path-root="selectedTask?.projectRepoFolder"
              @update:model-value="scheduleSave"
            />
            <AppMarkdownEditor v-else key="read" :model-value="draft" readonly />
          </Transition>
        </div>
      </div>
    </template>

    <Transition name="dialog">
      <AppConfirm
        v-if="isConfirmingDelete && selectedTask"
        :title="`Delete the wrapup on ${selectedTask.title}?`"
        body="The task and its notes stay. What goes is the description of where the implementation currently stands."
        blast="kept in History · the next `/rk … wrapup` starts from nothing"
        confirm-label="Delete wrapup"
        @cancel="isConfirmingDelete = false"
        @confirm="confirmDelete"
      />
    </Transition>

    <Transition name="dialog">
      <WrapupHereDialog
        v-if="wrapupHereOpen && selectedTask"
        :task-title="selectedTask.title"
        :anchor="selectedTask.anchor"
        :sending="sendingWrapupHere"
        @cancel="wrapupHereOpen = false"
        @send="sendWrapupHere"
      />
    </Transition>
  </section>
</template>
