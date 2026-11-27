<script setup lang="ts">
import TerminalConnectionChips from '@/features/connection/components/TerminalConnectionChips.vue'
import { computed, onMounted, ref, watch } from 'vue'
import TaskTitleRow from '@/features/catalog/components/TaskTitleRow.vue'
import SessionProviderMark from '@/features/claude/components/SessionProviderMark.vue'
import SessionSwitcher from '@/features/claude/components/SessionSwitcher.vue'
import TerminalScreen from '@/features/claude/components/TerminalScreen.vue'
import { storeToRefs } from 'pinia'
import { useConsoleStore } from '@/stores/console.store'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { identityHue } from '@/common/identity'
import { launchChoices } from '@/common/config/session-launch'
import { useRunProvider } from '@/features/claude/composables/useRunProvider'
import { SESSION_PROVIDER_LABEL, SESSION_WORKFLOW_LABEL } from '@/features/claude/model/session-provider'
import type { Terminal, TerminalEndedFrame } from '@/features/claude/model/terminal'

const store = useConsoleStore()
const terminals = useTerminalStore()
const { selectedTask } = storeToRefs(store)
const { terminals: allTerminals, activeTerminal, activeTerminalId } = storeToRefs(terminals)
const { run, isRunning } = useAsyncAction()

const launchProvider = useRunProvider(() => selectedTask.value?.id ?? null)
const connected = ref(false)
const ended = ref<{ exitCode: number; detail: string } | null>(null)

const hue = computed(() => identityHue(selectedTask.value?.projectId ?? ''))
const folder = computed(() => selectedTask.value?.projectRepoFolder ?? null)

/** Only sessions on a task can be shown here; one on a project alone lives on the Diagrams screen. */
const otherSessions = computed(() =>
  allTerminals.value
    .filter(
      (terminal) =>
        terminal.kind === 'TERMINAL' && terminal.live && terminal.taskId !== null && terminal.id !== activeTerminalId.value
    )
    .sort((a, b) => b.startedAt.localeCompare(a.startedAt))
)

/** Switch the pane to another live session without leaving the terminal view. */
function switchTo(terminal: Terminal): void {
  if (!terminal.taskId) return
  store.selectTask(terminal.taskId)
  store.openTerminal()
  terminals.select(terminal.id)
}

function onEnded(frame: TerminalEndedFrame): void {
  ended.value = { exitCode: frame.exitCode, detail: frame.detail }
  const endedTerminal = activeTerminal.value
  if (activeTerminalId.value) terminals.markEnded(activeTerminalId.value)
  if (endedTerminal?.taskId)
    void run(() => terminals.followNextSession(endedTerminal.taskId!, endedTerminal.id))
}

watch(activeTerminalId, () => (ended.value = null))

async function openHere(): Promise<void> {
  const task = selectedTask.value
  if (!task || !folder.value) return
  await run(
    () =>
      terminals.openForTask(task.id, {
        ...launchChoices(launchProvider.value),
        kind: 'TERMINAL'
      }),
    'Terminal opened'
  )
}

async function restart(): Promise<void> {
  const current = activeTerminal.value
  if (!current?.taskId) return
  const taskId = current.taskId
  await run(async () => {
    await terminals.close(current.id)
    await terminals.openForTask(taskId, {
      provider: current.provider,
      skipPermissions: current.skipPermissions,
      model: current.model ?? undefined,
      effort: current.effort ?? undefined,
      stepId: current.stepId,
      kind: current.kind
    })
  }, 'Terminal restarted')
}

async function closeActive(): Promise<void> {
  const current = activeTerminalId.value
  if (!current) return
  await run(() => terminals.close(current))
}

function pickForTask(): void {
  const live = terminals.liveTerminalForTask(selectedTask.value?.id ?? null)
  terminals.select(live ? live.id : null)
}

onMounted(async () => {
  await run(() => terminals.load())
  pickForTask()
})

watch(
  () => selectedTask.value?.id ?? null,
  async (taskId) => {
    if (taskId) await run(() => terminals.load())
    pickForTask()
  }
)
</script>

<template>
  <section class="flex min-h-0 w-full min-w-0 flex-1 flex-col bg-canvas" aria-label="Terminal">
    <p v-if="!selectedTask" class="px-9 py-12 text-[13px] text-text-muted">
      Pick a task to open a terminal on it.
    </p>

    <template v-else>
      <div class="relative shrink-0 border-b border-border">
        <div class="pointer-events-none absolute inset-0 overflow-hidden" aria-hidden="true">
          <div
            class="texture-grid absolute inset-0 opacity-60"
            :style="{ '--texture-tint': hue.base }"
          />
          <div
            class="absolute inset-x-0 top-0 h-px"
            :style="{ background: hue.line }"
          />
        </div>
        <header class="relative flex items-start gap-4 px-5 py-3.5">
          <div class="min-w-0 flex-1">
            <p class="eyebrow flex items-center gap-1.5">
              <span class="h-2.5 w-[3px] shrink-0 rounded-full bg-accent" aria-hidden="true" />
              Terminal
            </p>
            <TaskTitleRow :task="selectedTask" copy-testid="copy-terminal-anchor">
              <template #flags>
              <span
                v-if="activeTerminal"
                class="inline-flex shrink-0 items-center gap-1.5 rounded-key bg-surface-raised px-2 py-0.5 text-micro font-semibold tracking-[0.02em]"
                :class="connected ? 'text-safe' : ended ? 'text-text-subtle' : 'text-accent'"
                data-testid="terminal-status"
              >
                <span
                  v-if="connected"
                  class="relative grid size-2 place-items-center"
                  aria-hidden="true"
                >
                  <span class="absolute inline-flex size-2 animate-ping rounded-full bg-current/50" />
                  <span class="relative inline-flex size-1.5 rounded-full bg-current" />
                </span>
                {{ connected ? 'connected' : ended ? 'ended' : 'connecting' }}
              </span>
              <TerminalConnectionChips
                v-if="activeTerminal"
                :terminal-id="activeTerminal.id"
                :project-id="activeTerminal.projectId"
                :connections="activeTerminal.connections"
              />
              </template>
            </TaskTitleRow>
          </div>

          <div class="flex shrink-0 items-center gap-1.5">
            <SessionSwitcher v-if="otherSessions.length > 0" :sessions="otherSessions" @switch="switchTo" />
            <button
              v-if="activeTerminal"
              class="key-quiet rounded-key focus-ring inline-flex h-7 items-center gap-1.5 px-2.5 text-[11.5px]"
              data-testid="terminal-restart"
              :disabled="isRunning"
              @click="restart"
            >
              Restart
            </button>
            <button
              v-if="activeTerminal"
              class="key-danger rounded-key focus-ring inline-flex h-7 items-center gap-1.5 px-2.5 text-[11.5px]"
              data-testid="terminal-close"
              :disabled="isRunning"
              @click="closeActive"
            >
              Close
            </button>
            <button
              v-else
              class="key-gold rounded-key focus-ring inline-flex h-7 items-center gap-1.5 px-2.5 text-[11.5px] font-medium disabled:cursor-not-allowed disabled:opacity-60"
              data-testid="terminal-open"
              :disabled="isRunning || !folder"
              @click="openHere"
            >
              Run here
            </button>
          </div>
        </header>
        <div class="relative flex flex-wrap items-center gap-x-3 gap-y-1 border-t border-border bg-surface px-5 py-2">
          <span v-if="activeTerminal" class="inline-flex shrink-0 items-center gap-1.5 text-caption text-text-muted">
            <SessionProviderMark :provider="activeTerminal.provider" :size="12" />
            {{ SESSION_PROVIDER_LABEL[activeTerminal.provider] }}
          </span>
          <span v-if="activeTerminal" class="truncate font-mono text-micro text-text-subtle">
            {{ activeTerminal.workingDir }}
          </span>
          <span
            v-if="activeTerminal"
            class="rounded-key px-1.5 py-px text-[10px] font-semibold tracking-[0.02em]"
            :class="activeTerminal.skipPermissions ? 'bg-warn-soft text-warn' : 'bg-safe-soft text-safe'"
          >
            {{ activeTerminal.skipPermissions ? 'permissions skipped' : 'permissions on' }}
          </span>
          <span v-else class="text-[11.5px] text-text-muted">
            Opens a terminal in this project’s folder and loads <code class="text-anchor/80">{{ selectedTask.anchor }}</code>.
          </span>
        </div>
      </div>

      <div class="relative min-h-0 flex-1 bg-[#0b0b0d]">
        <div
          v-if="!activeTerminal"
          class="absolute inset-0 grid place-items-center px-9 text-center"
        >
          <div class="flex max-w-[460px] flex-col items-center">
            <!-- A window with a prompt and a caret at rest: what will open here, drawn still,
                 because nothing is running yet and a mark that moved would say otherwise. -->
            <span
              class="mb-5 flex h-[54px] w-[78px] flex-col overflow-hidden rounded-[10px] border border-border-strong bg-surface shadow-lift"
              aria-hidden="true"
            >
              <span class="flex h-3.5 shrink-0 items-center gap-[3px] border-b border-border px-1.5">
                <span class="size-[4px] rounded-full bg-border-strong" />
                <span class="size-[4px] rounded-full bg-border-strong" />
                <span class="size-[4px] rounded-full bg-border-strong" />
              </span>
              <span class="flex flex-1 items-center gap-1.5 px-2.5 font-mono text-[11px] text-anchor/70">
                &rsaquo;
                <span class="h-[11px] w-[3px] rounded-[1px] bg-anchor/50" />
              </span>
            </span>
            <h3 class="text-[17px] font-semibold tracking-[-0.015em] text-text">
              No terminal open on this task
            </h3>
            <p class="mt-2 text-[12.5px] leading-relaxed text-text-muted">
              Opening one starts <code class="text-anchor/80">{{ launchProvider.toLowerCase() }}</code> in a pseudo-terminal in
              <span v-if="folder" class="font-mono text-[11.5px] text-text">{{ folder }}</span>
              <template v-else>this project's folder</template>, loads the task context with
              <code class="text-anchor/80">{{ SESSION_WORKFLOW_LABEL[launchProvider] }}</code>, and hands it to you.
            </p>
            <RouterLink
              v-if="!folder"
              :to="`/projects/${selectedTask.projectId}`"
              class="focus-ring mt-4 inline-flex items-center gap-1.5 rounded-[var(--radius-control)] border border-warn/40 bg-warn-soft px-3 py-1.5 text-[11.5px] font-medium text-warn transition-colors hover:border-warn"
              data-testid="terminal-set-folder"
            >
              Set this project's folder on its page first
              <span aria-hidden="true">&rarr;</span>
            </RouterLink>
          </div>
        </div>
        <div class="absolute inset-0 px-3 py-2">
          <TerminalScreen :terminal-id="activeTerminalId" @ended="onEnded" @connection="connected = $event" />
        </div>
      </div>
    </template>
  </section>
</template>
