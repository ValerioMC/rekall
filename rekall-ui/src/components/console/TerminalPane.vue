<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import CopyGlyph from '@/components/ui/CopyGlyph.vue'
import { storeToRefs } from 'pinia'
import { Terminal as Xterm } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import '@xterm/xterm/css/xterm.css'
import { useConsoleStore } from '@/stores/console.store'
import { useTerminalStore } from '@/stores/terminal.store'
import { useTerminalSocket } from '@/composables/useTerminalSocket'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { identityHue } from '@/common/identity'
import { rkCommand } from '@/common/format/rk-command'
import { preferredEffort, preferredModel, skipsPermissions } from '@/common/config/claude-launch'
import type { Terminal } from '@/model/terminal'
import type { TerminalId } from '@/model/branded'

const TERMINAL_THEME = {
  background: '#0b0b0d',
  foreground: '#d6d6d6',
  cursor: '#d6d6d6',
  selectionBackground: '#3a3a44',
  black: '#1c1c22',
  red: '#e0708a',
  green: '#8fcf9d',
  yellow: '#d9b57a',
  blue: '#7aa2d9',
  magenta: '#b78ad9',
  cyan: '#79c7c7',
  white: '#c9c9c9',
  brightBlack: '#5a5a66'
}

const store = useConsoleStore()
const terminals = useTerminalStore()
const { selectedTask, tasks } = storeToRefs(store)
const { terminals: allTerminals, activeTerminal, activeTerminalId } = storeToRefs(terminals)
const { run, isRunning } = useAsyncAction()

const host = ref<HTMLElement | null>(null)
const ended = ref<{ exitCode: number; detail: string } | null>(null)
const switcherOpen = ref(false)

const hue = computed(() => identityHue(selectedTask.value?.projectId ?? ''))
const folder = computed(() => selectedTask.value?.projectRepoFolder ?? null)
const taskTerminal = computed(() => terminals.terminalForTask(selectedTask.value?.id ?? null))

const otherSessions = computed(() =>
  allTerminals.value
    .filter((terminal) => terminal.live && terminal.id !== activeTerminalId.value)
    .sort((a, b) => b.startedAt.localeCompare(a.startedAt))
)

function projectIdOf(terminal: Terminal): string {
  return tasks.value.find((task) => task.id === terminal.taskId)?.projectId ?? terminal.taskId
}

/** Switch the pane to another live session without leaving the terminal view. */
function switchTo(terminal: Terminal): void {
  store.selectTask(terminal.taskId)
  store.openTerminal()
  terminals.select(terminal.id)
  switcherOpen.value = false
}

function onSwitcherKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape' && switcherOpen.value) {
    event.stopPropagation()
    switcherOpen.value = false
  }
}

let xterm: Xterm | null = null
let fit: FitAddon | null = null
let resizeObserver: ResizeObserver | null = null
let dataBinding: { dispose: () => void } | null = null
let resizeBinding: { dispose: () => void } | null = null
let connectedId: TerminalId | null = null

const socket = useTerminalSocket({
  onOutput: (bytes) => xterm?.write(bytes),
  onReady: () => refitWhenReady(),
  onEnded: (frame) => {
    ended.value = { exitCode: frame.exitCode, detail: frame.detail }
    if (activeTerminalId.value) terminals.markEnded(activeTerminalId.value)
    xterm?.write(`\r\n\x1b[2m[ ${frame.detail || 'session ended'} ]\x1b[0m\r\n`)
  },
  onClose: () => {}
})

const { connected } = socket

function ensureXterm(): Xterm {
  if (!xterm) {
    const term = new Xterm({
      fontFamily: "'Fira Code Variable', 'Fira Code', ui-monospace, SFMono-Regular, Menlo, monospace",
      fontSize: 12.5,
      lineHeight: 1.25,
      cursorBlink: true,
      theme: TERMINAL_THEME,
      scrollback: 5000
    })
    fit = new FitAddon()
    term.loadAddon(fit)
    dataBinding = term.onData((data) => socket.sendInput(data))
    resizeBinding = term.onResize(({ cols, rows }) => socket.sendResize(cols, rows))
    xterm = term
  }
  if (!xterm.element && host.value) xterm.open(host.value)
  return xterm
}

/** Re-measure xterm against its host on the next frame, once layout has settled. */
function refit(): void {
  requestAnimationFrame(() => {
    if (!fit || !xterm) return
    try {
      fit.fit()
    } catch {
      // The host has no size yet; the ResizeObserver will catch the next layout.
      return
    }
    socket.sendResize(xterm.cols, xterm.rows)
  })
}

/**
 * The first fit on a fresh terminal runs before the Fira Code web font has
 * loaded, so xterm sizes its cell against fallback metrics and the grid lands a
 * few cols/rows short until the next resize nudges it. Fit now for layout, then
 * again once the real font is in.
 */
function refitWhenReady(): void {
  refit()
  document.fonts?.ready.then(() => refit())
}

/** Wire xterm + the socket to `id`. No-ops until the DOM is ready; `syncConnection` retries on mount. */
function connectTo(id: TerminalId): void {
  if (!host.value) return
  ended.value = null
  const term = ensureXterm()
  term.reset()
  socket.connect(id)
  connectedId = id
  refitWhenReady()
  term.focus()
}

/** Bring the live connection in line with `activeTerminalId`. The single place the pane (re)connects. */
function syncConnection(): void {
  const id = activeTerminalId.value
  if (id) {
    if (id !== connectedId) connectTo(id)
  } else if (connectedId) {
    socket.disconnect()
    connectedId = null
    xterm?.reset()
  }
}

async function openHere(): Promise<void> {
  const task = selectedTask.value
  if (!task || !folder.value) return
  await run(
    () =>
      terminals.openForTask(task.id, {
        skipPermissions: skipsPermissions(),
        model: preferredModel(),
        effort: preferredEffort()
      }),
    'Terminal opened'
  )
  syncConnection()
}

async function restart(): Promise<void> {
  const task = selectedTask.value
  const current = activeTerminalId.value
  if (!task || !current) return
  await run(async () => {
    socket.disconnect()
    connectedId = null
    await terminals.close(current)
    await terminals.openForTask(task.id, {
      skipPermissions: skipsPermissions(),
      model: preferredModel(),
      effort: preferredEffort()
    })
  }, 'Terminal restarted')
  syncConnection()
}

async function closeActive(): Promise<void> {
  const current = activeTerminalId.value
  if (!current) return
  socket.disconnect()
  connectedId = null
  await run(() => terminals.close(current))
}

function pickForTask(): void {
  const live = taskTerminal.value
  terminals.select(live ? live.id : null)
}

const copied = ref(false)

async function copyAnchor(): Promise<void> {
  if (!selectedTask.value) return
  await navigator.clipboard?.writeText(rkCommand(selectedTask.value.anchor))
  copied.value = true
  setTimeout(() => (copied.value = false), 1400)
}

onMounted(async () => {
  await run(() => terminals.load())
  pickForTask()
  syncConnection()
  resizeObserver = new ResizeObserver(() => refit())
  if (host.value) resizeObserver.observe(host.value)
})

watch(
  () => selectedTask.value?.id ?? null,
  async (taskId) => {
    if (!taskId) {
      pickForTask()
      syncConnection()
      return
    }
    await run(() => terminals.load())
    pickForTask()
    syncConnection()
  }
)

watch(activeTerminalId, () => syncConnection())

onBeforeUnmount(() => {
  resizeObserver?.disconnect()
  resizeObserver = null
  dataBinding?.dispose()
  resizeBinding?.dispose()
  socket.disconnect()
  fit?.dispose()
  xterm?.dispose()
  xterm = null
  fit = null
})
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
            <h2 class="flex items-center gap-2 truncate text-[19px] font-semibold tracking-[-0.015em] text-text">
              <span class="truncate">{{ selectedTask.title }}</span>
              <span
                v-if="activeTerminal"
                class="inline-flex shrink-0 items-center gap-1.5 rounded-full bg-surface-raised px-2 py-0.5 text-[10.5px] font-semibold tracking-[0.02em]"
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
            </h2>
            <button
              class="anchor-chip focus-ring mt-1.5 inline-flex items-center gap-2 px-2.5 py-1 text-[11.5px] transition-colors hover:border-anchor"
              :class="copied && 'flash'"
              :title="copied ? 'Copied' : `Copy ${rkCommand(selectedTask.anchor)}`"
              data-testid="copy-terminal-anchor"
              @click="copyAnchor"
            >
              <span class="opacity-60">/rk</span>
              <span>{{ selectedTask.anchor }}</span>
              <CopyGlyph :copied="copied" />
            </button>
          </div>

          <div class="flex shrink-0 items-center gap-1.5">
            <div v-if="otherSessions.length > 0" class="relative" @keydown="onSwitcherKeydown">
              <button
                class="focus-ring inline-flex h-7 items-center gap-1.5 rounded-[var(--radius-control)] border border-border-strong px-2.5 text-[11.5px] text-text-muted transition-colors hover:border-anchor/50 hover:text-text"
                :class="switcherOpen && 'border-anchor/50 text-text'"
                data-testid="terminal-other-sessions"
                aria-haspopup="true"
                :aria-expanded="switcherOpen"
                @click="switcherOpen = !switcherOpen"
              >
                <span class="session-caret session-caret-busy shrink-0" aria-hidden="true" />
                Other {{ otherSessions.length }} {{ otherSessions.length === 1 ? 'session' : 'sessions' }}
              </button>

              <div
                v-if="switcherOpen"
                class="rise absolute right-0 top-[calc(100%+6px)] z-(--z-overlay) w-[300px] overflow-hidden rounded-[var(--radius-card)] border border-border-strong bg-surface-raised shadow-modal"
                data-testid="terminal-other-sessions-list"
              >
                <ul class="max-h-[280px] overflow-y-auto p-1.5">
                  <li v-for="terminal in otherSessions" :key="terminal.id">
                    <button
                      class="focus-ring flex w-full min-w-0 items-center gap-2.5 rounded-[var(--radius-control)] px-2 py-2 text-left transition-colors hover:bg-surface"
                      :title="`Switch to ${terminal.taskTitle}`"
                      data-testid="terminal-other-sessions-row"
                      @click="switchTo(terminal)"
                    >
                      <span class="session-caret session-caret-busy shrink-0" aria-hidden="true" />
                      <span class="min-w-0 flex-1">
                        <span class="block truncate text-[12.5px] font-medium text-text">
                          {{ terminal.taskTitle }}
                        </span>
                        <span class="mt-0.5 flex items-center gap-1.5 truncate font-mono text-[10px] text-anchor/80">
                          <span
                            class="size-1 shrink-0 rounded-full"
                            :style="{ backgroundColor: identityHue(projectIdOf(terminal)).base }"
                            aria-hidden="true"
                          />
                          <span class="min-w-0 truncate">{{ terminal.anchors }}</span>
                        </span>
                      </span>
                    </button>
                  </li>
                </ul>
              </div>

              <button
                v-if="switcherOpen"
                class="fixed inset-0 z-(--z-sticky) cursor-default"
                tabindex="-1"
                aria-hidden="true"
                @click="switcherOpen = false"
              />
            </div>
            <button
              v-if="activeTerminal"
              class="focus-ring inline-flex h-7 items-center gap-1.5 rounded-[var(--radius-control)] border border-border-strong px-2.5 text-[11.5px] text-text-muted transition-colors hover:border-text-subtle hover:text-text"
              data-testid="terminal-restart"
              :disabled="isRunning"
              @click="restart"
            >
              Restart
            </button>
            <button
              v-if="activeTerminal"
              class="focus-ring inline-flex h-7 items-center gap-1.5 rounded-[var(--radius-control)] border border-danger/40 px-2.5 text-[11.5px] text-danger transition-colors hover:border-danger hover:bg-danger-soft"
              data-testid="terminal-close"
              :disabled="isRunning"
              @click="closeActive"
            >
              Close
            </button>
            <button
              v-else
              class="focus-ring inline-flex h-7 items-center gap-1.5 rounded-[var(--radius-control)] border border-accent bg-accent-soft px-2.5 text-[11.5px] font-medium text-accent transition-colors hover:bg-accent hover:text-accent-ink disabled:cursor-not-allowed disabled:opacity-60"
              data-testid="terminal-open"
              :disabled="isRunning || !folder"
              @click="openHere"
            >
              Run here
            </button>
          </div>
        </header>
        <div class="relative flex flex-wrap items-center gap-x-3 gap-y-1 border-t border-border bg-surface px-5 py-2">
          <span v-if="activeTerminal" class="truncate font-mono text-[10.5px] text-text-subtle">
            {{ activeTerminal.workingDir }}
          </span>
          <span
            v-if="activeTerminal"
            class="rounded-full px-1.5 py-px text-[10px] font-semibold tracking-[0.02em]"
            :class="activeTerminal.skipPermissions ? 'bg-warn-soft text-warn' : 'bg-safe-soft text-safe'"
          >
            {{ activeTerminal.skipPermissions ? 'permissions skipped' : 'permissions on' }}
          </span>
          <span v-else class="text-[11.5px] text-text-muted">
            A real terminal running <code class="text-anchor/80">claude</code> in this project's folder. It
            loads <code class="text-anchor/80">/rk {{ selectedTask.anchor }}</code> first. Caching and
            context behave exactly as they do in your own terminal.
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
              Opening one starts <code class="text-anchor/80">claude</code> in a pseudo-terminal in
              <span v-if="folder" class="font-mono text-[11.5px] text-text">{{ folder }}</span>
              <template v-else>this project's folder</template>, loads the task context with
              <code class="text-anchor/80">/rk</code>, and hands it to you.
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
        <div ref="host" class="absolute inset-0 px-3 py-2" data-testid="terminal-host" />
      </div>
    </template>
  </section>
</template>

<style scoped>
:deep(.xterm) {
  height: 100%;
}
:deep(.xterm-viewport) {
  background-color: transparent !important;
}
</style>
