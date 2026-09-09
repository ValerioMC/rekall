<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import ClaudeTranscript from '@/components/claude/ClaudeTranscript.vue'
import ClaudeComposer from '@/components/claude/ClaudeComposer.vue'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useClaudeStore } from '@/stores/claude.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useClaudeSessionStream } from '@/composables/useClaudeSessionStream'
import { identityHue } from '@/common/identity'
import { relativeTime } from '@/common/format/relative-time'
import { preferredEffort, preferredModel, skipsPermissions } from '@/common/config/claude-launch'
import {
  claudeEffortChoiceLabel,
  claudeEffortLabel,
  claudeModelChoiceLabel,
  claudeModelLabel,
  claudeSessionAcceptsPrompt,
  claudeStatusLabel,
  type ClaudeSession
} from '@/model/claude'
import type { ClaudeSessionId } from '@/model/branded'

const store = useConsoleStore()
const claude = useClaudeStore()
const { selectedTask } = storeToRefs(store)
const { activeSession, activeSessionId, activeMessages } = storeToRefs(claude)
const { run } = useAsyncAction()

const hue = computed(() => identityHue(selectedTask.value?.projectId ?? ''))

const taskSessions = computed(() => claude.sessionsForTask(selectedTask.value?.id ?? null))

const elsewhere = computed(() =>
  claude.liveSessions.filter((session) => session.taskId !== selectedTask.value?.id)
)

const working = computed(
  () => activeSession.value?.status === 'WORKING' || activeSession.value?.status === 'STARTING'
)

const composerDisabled = computed(
  () => !activeSession.value || !claudeSessionAcceptsPrompt(activeSession.value.status)
)

const modelLabel = computed(() => claudeModelLabel(activeSession.value?.model))

/** opus / sonnet / haiku each get a faint tone so "which model" reads at a glance. */
const modelTone = computed(() => {
  const label = modelLabel.value?.toLowerCase() ?? ''
  if (label.startsWith('opus')) return 'bg-accent-soft text-accent'
  if (label.startsWith('sonnet')) return 'bg-anchor/10 text-anchor'
  if (label.startsWith('haiku')) return 'bg-safe-soft text-safe'
  return 'bg-surface-raised text-text-muted'
})

const nextModelLabel = computed(() => claudeModelChoiceLabel(preferredModel()))

const effortLabel = computed(() => claudeEffortLabel(activeSession.value?.effort))
const nextEffortLabel = computed(() => claudeEffortChoiceLabel(preferredEffort()))

const composerHint = computed(() => {
  const session = activeSession.value
  if (!session) return undefined
  if (session.status === 'WORKING' || session.status === 'STARTING') return 'Claude is working. One turn at a time.'
  if (!session.live) return 'This session has ended. Start a new one to keep going.'
  return undefined
})

useClaudeSessionStream(activeSessionId, {
  onMessage: (message) => claude.applyMessage(message),
  onStatus: (session) => claude.applySession(session),
  onEnded: (session) => claude.applySession(session)
})

function pickDefault(): void {
  const own = taskSessions.value
  const live = own.find((session) => session.live)
  void claude.selectSession((live ?? own[0])?.id ?? null)
}

onMounted(() => {
  void run(() => claude.loadSessions()).then(pickDefault)
})

watch(
  () => selectedTask.value?.id ?? null,
  (taskId) => {
    if (!taskId) {
      void claude.selectSession(null)
      return
    }
    void run(() => claude.loadTaskSessions(taskId)).then(pickDefault)
  }
)

function select(id: ClaudeSessionId): void {
  void claude.selectSession(id)
}

async function startNew(): Promise<void> {
  const task = selectedTask.value
  if (!task) return
  await run(
    () =>
      claude.startForTask(task.id, {
        skipPermissions: skipsPermissions(),
        model: preferredModel(),
        effort: preferredEffort()
      }),
    'Session started'
  )
}

async function send(text: string): Promise<void> {
  const session = activeSession.value
  if (!session) return
  await run(() => claude.sendPrompt(session.id, text))
}

async function stop(): Promise<void> {
  const session = activeSession.value
  if (!session) return
  await run(() => claude.stop(session.id))
}

async function clear(): Promise<void> {
  const session = activeSession.value
  if (!session) return
  await run(() => claude.clearSession(session.id), 'Context cleared, /rk reloaded')
}

const removing = ref<ClaudeSession | null>(null)

async function confirmRemove(): Promise<void> {
  const session = removing.value
  removing.value = null
  if (!session) return
  await run(() => claude.remove(session.id))
  pickDefault()
}

function jumpTo(session: ClaudeSession): void {
  store.selectTask(session.taskId)
  store.openClaude()
  void claude.selectSession(session.id)
}

const menuOpen = ref(false)

function statusTone(status: ClaudeSession['status']): string {
  if (status === 'READY') return 'text-safe'
  if (status === 'WORKING' || status === 'STARTING') return 'text-accent'
  if (status === 'FAILED') return 'text-danger'
  return 'text-text-subtle'
}
</script>

<template>
  <section class="flex min-h-0 w-full min-w-0 flex-1 flex-col bg-canvas" aria-label="Session">
    <p v-if="!selectedTask" class="px-9 py-12 text-[13px] text-text-muted">
      Pick a task to run a session on it.
    </p>

    <template v-else>
      <div class="relative shrink-0 overflow-hidden border-b border-border">
        <div
          class="texture-grid pointer-events-none absolute inset-0 opacity-60"
          :style="{ '--texture-tint': hue.base }"
          aria-hidden="true"
        />
        <div
          class="pointer-events-none absolute inset-x-0 top-0 h-px"
          :style="{ background: hue.line }"
          aria-hidden="true"
        />

        <header class="relative flex items-start gap-4 px-5 py-3.5">
          <div class="min-w-0 flex-1">
            <p class="eyebrow flex items-center gap-1.5">
              <span class="h-2.5 w-[3px] shrink-0 rounded-full bg-accent" aria-hidden="true" />
              Session
            </p>
            <h2 class="flex items-center gap-2 truncate text-[19px] font-semibold tracking-[-0.015em] text-text">
              <span class="truncate">{{ selectedTask.title }}</span>
              <span
                v-if="activeSession"
                class="inline-flex shrink-0 items-center gap-1.5 rounded-full bg-surface-raised px-2 py-0.5 text-[10.5px] font-semibold tracking-[0.02em]"
                :class="statusTone(activeSession.status)"
                data-testid="claude-session-status"
              >
                <span
                  v-if="activeSession.live"
                  class="relative grid size-2 place-items-center"
                  aria-hidden="true"
                >
                  <span class="absolute inline-flex size-2 animate-ping rounded-full bg-current/50" />
                  <span class="relative inline-flex size-1.5 rounded-full bg-current" />
                </span>
                {{ claudeStatusLabel(activeSession.status) }}
              </span>
            </h2>
            <p class="anchor-chip mt-1.5 inline-flex items-center gap-2 px-2.5 py-1 font-mono text-[11.5px]">
              <span class="opacity-60">/rk</span>
              <span>{{ selectedTask.anchor }}</span>
            </p>
          </div>

          <div class="flex shrink-0 items-center gap-1.5">
            <div v-if="elsewhere.length" class="relative">
              <button
                class="focus-ring inline-flex h-7 items-center gap-1.5 rounded-[var(--radius-control)] border border-border-strong px-2.5 text-[11.5px] text-text-muted transition-colors hover:border-text-subtle hover:text-text"
                :aria-expanded="menuOpen"
                data-testid="claude-elsewhere-toggle"
                @click="menuOpen = !menuOpen"
              >
                <span class="size-1.5 rounded-full bg-accent" aria-hidden="true" />
                {{ elsewhere.length }} live elsewhere
              </button>
              <ul
                v-if="menuOpen"
                class="rise absolute right-0 top-9 z-(--z-overlay) w-[260px] overflow-hidden rounded-[var(--radius-card)] border border-border-strong bg-surface shadow-modal"
              >
                <li v-for="session in elsewhere" :key="session.id">
                  <button
                    class="focus-ring flex w-full items-center gap-2 px-3 py-2 text-left transition-colors hover:bg-surface-raised"
                    @click="jumpTo(session); menuOpen = false"
                  >
                    <span class="size-1.5 shrink-0 rounded-full bg-accent" aria-hidden="true" />
                    <span class="min-w-0 flex-1">
                      <span class="block truncate text-[12px] text-text">{{ session.taskTitle }}</span>
                      <span class="block truncate font-mono text-[10px] text-anchor/80">{{ session.anchor }}</span>
                    </span>
                    <span class="shrink-0 text-[10px] text-text-subtle">{{ claudeStatusLabel(session.status) }}</span>
                  </button>
                </li>
              </ul>
            </div>

            <button
              v-if="activeSession?.live"
              class="focus-ring inline-flex h-7 items-center gap-1.5 rounded-[var(--radius-control)] border border-border-strong px-2.5 text-[11.5px] text-text-muted transition-colors hover:border-text-subtle hover:text-text"
              data-testid="claude-clear"
              title="Drop the conversation so far and reload /rk. Keeps the session and its warm cache."
              @click="clear"
            >
              Clear
            </button>
            <button
              v-if="activeSession?.live"
              class="focus-ring inline-flex h-7 items-center gap-1.5 rounded-[var(--radius-control)] border border-danger/40 px-2.5 text-[11.5px] text-danger transition-colors hover:bg-danger-soft hover:border-danger"
              data-testid="claude-stop"
              @click="stop"
            >
              Stop
            </button>
            <button
              v-if="!taskSessions.some((session) => session.live)"
              class="focus-ring inline-flex h-7 items-center gap-1.5 rounded-[var(--radius-control)] border border-accent bg-accent-soft px-2.5 text-[11.5px] font-medium text-accent transition-colors hover:bg-accent hover:text-accent-ink"
              data-testid="claude-new"
              @click="startNew"
            >
              New session
            </button>
          </div>
        </header>
      </div>

      <div
        class="flex shrink-0 flex-wrap items-center gap-x-3 gap-y-1.5 border-b border-border bg-surface px-5 py-2"
      >
        <span v-if="activeSession" class="truncate font-mono text-[10.5px] text-text-subtle">
          {{ activeSession.workingDir }}
        </span>
        <span
          v-if="activeSession && modelLabel"
          class="inline-flex items-center gap-1 rounded-full px-1.5 py-px text-[10px] font-semibold tracking-[0.02em]"
          :class="modelTone"
          data-testid="claude-session-model"
          title="The model this session is running"
        >
          <svg class="size-2.5" viewBox="0 0 10 10" fill="none" aria-hidden="true">
            <path d="M5 1 8.5 3v4L5 9 1.5 7V3Z" stroke="currentColor" stroke-width="1" stroke-linejoin="round" />
          </svg>
          {{ modelLabel }}
        </span>
        <span
          v-if="activeSession && effortLabel"
          class="inline-flex items-center gap-1 rounded-full bg-surface-raised px-1.5 py-px text-[10px] font-semibold tracking-[0.02em] text-text-muted"
          data-testid="claude-session-effort"
          title="The reasoning effort this session runs at"
        >
          <svg class="size-2.5" viewBox="0 0 10 10" fill="none" aria-hidden="true">
            <path d="M1.5 8.5a3.5 3.5 0 0 1 7 0" stroke="currentColor" stroke-width="1" stroke-linecap="round" />
            <path d="M5 8 6.8 4.2" stroke="currentColor" stroke-width="1" stroke-linecap="round" />
          </svg>
          {{ effortLabel }} effort
        </span>
        <span
          v-if="activeSession"
          class="rounded-full px-1.5 py-px text-[10px] font-semibold tracking-[0.02em]"
          :class="activeSession.skipPermissions ? 'bg-warn-soft text-warn' : 'bg-safe-soft text-safe'"
        >
          {{ activeSession.skipPermissions ? 'permissions skipped' : 'permissions on' }}
        </span>
        <span v-if="!activeSession" class="text-[11.5px] text-text-muted">
          Runs Claude Code in this project's folder, in the app. It loads
          <code class="text-anchor/80">/rk {{ selectedTask.anchor }}</code> first, then it is yours to prompt.
        </span>
      </div>

      <div
        v-if="taskSessions.length > 1"
        class="flex shrink-0 items-center gap-1.5 overflow-x-auto border-b border-border bg-surface px-5 py-2"
        data-testid="claude-session-switcher"
      >
        <button
          v-for="session in taskSessions"
          :key="session.id"
          class="focus-ring inline-flex shrink-0 items-center gap-1.5 rounded-full border px-2.5 py-1 text-[11px] transition-colors"
          :class="
            session.id === activeSessionId
              ? 'border-accent bg-accent-soft text-accent'
              : 'border-border text-text-subtle hover:border-border-strong hover:text-text-muted'
          "
          @click="select(session.id)"
        >
          <span
            class="size-1.5 rounded-full"
            :class="session.live ? 'bg-accent' : 'bg-text-subtle'"
            aria-hidden="true"
          />
          {{ relativeTime(session.startedAt) }}
        </button>
      </div>

      <template v-if="activeSession">
        <ClaudeTranscript :messages="activeMessages" :working="working" />

        <p
          v-if="!activeSession.live && activeSession.detail"
          class="shrink-0 border-t border-border bg-surface px-5 py-2 text-[11.5px] text-text-subtle"
          data-testid="claude-ended-detail"
        >
          {{ activeSession.detail }}
        </p>

        <ClaudeComposer :disabled="composerDisabled" :hint="composerHint" @send="send" />
      </template>

      <div v-else class="min-h-0 flex-1 overflow-y-auto px-9 py-12">
        <div class="max-w-[560px]">
          <p class="eyebrow">Session</p>
          <h3 class="mb-1.5 mt-1.5 text-[21px] font-semibold leading-tight tracking-[-0.02em] text-text">
            Run Claude Code here, not in a terminal
          </h3>
          <p class="mb-5 text-[13px] leading-relaxed text-text-muted">
            A session starts in this project's folder, loads
            <code class="text-anchor/80">/rk {{ selectedTask.anchor }}</code> so Claude has the task's
            context, and stays open for you to keep prompting. It is tied to this task, and you can
            run one per task at the same time.
          </p>
          <p class="mb-5 text-[12px] leading-relaxed text-text-subtle">
            {{
              skipsPermissions()
                ? 'Permission prompts are skipped (set in Settings), so the session can edit files and run commands in the folder without asking.'
                : 'Permission prompts are on, so tool use is denied unless you turn on “skip permissions” in Settings. In-app sessions have no way to answer a prompt yet.'
            }}
          </p>
          <p class="mb-5 text-[12px] leading-relaxed text-text-subtle" data-testid="claude-next-model">
            Model <span class="text-text-muted">{{ nextModelLabel }}</span>, effort
            <span class="text-text-muted">{{ nextEffortLabel }}</span>. Change both in Settings.
          </p>
          <button
            class="focus-ring rounded-[var(--radius-control)] border border-accent bg-accent-soft px-3.5 py-2 text-[12.5px] font-medium text-accent transition-colors hover:bg-accent hover:text-accent-ink"
            data-testid="claude-start-first"
            @click="startNew"
          >
            Start a session
          </button>
        </div>
      </div>
    </template>

    <AppConfirm
      v-if="removing"
      title="Remove this session?"
      body="Deletes the session and its transcript. If it is still running, it is stopped first."
      blast="one session and its transcript · not recoverable"
      confirm-label="Remove session"
      @cancel="removing = null"
      @confirm="confirmRemove"
    />
  </section>
</template>
