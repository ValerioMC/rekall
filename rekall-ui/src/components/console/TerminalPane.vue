<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import { Terminal as Xterm } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import '@xterm/xterm/css/xterm.css'
import { useConsoleStore } from '@/stores/console.store'
import { useTerminalStore } from '@/stores/terminal.store'
import { useTerminalSocket } from '@/composables/useTerminalSocket'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { identityHue } from '@/common/identity'
import { preferredEffort, preferredModel, skipsPermissions } from '@/common/config/claude-launch'
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
const { selectedTask } = storeToRefs(store)
const { activeTerminal, activeTerminalId } = storeToRefs(terminals)
const { run, isRunning } = useAsyncAction()

const host = ref<HTMLElement | null>(null)
const ended = ref<{ exitCode: number; detail: string } | null>(null)

const hue = computed(() => identityHue(selectedTask.value?.projectId ?? ''))
const folder = computed(() => selectedTask.value?.projectRepoFolder ?? null)
const taskTerminal = computed(() => terminals.terminalForTask(selectedTask.value?.id ?? null))

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
            <p class="anchor-chip mt-1.5 inline-flex items-center gap-2 px-2.5 py-1 font-mono text-[11.5px]">
              <span class="opacity-60">/rk</span>
              <span>{{ selectedTask.anchor }}</span>
            </p>
          </div>

          <div class="flex shrink-0 items-center gap-1.5">
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
          <div class="max-w-[460px]">
            <h3 class="text-[17px] font-semibold tracking-[-0.015em] text-text">
              No terminal open on this task
            </h3>
            <p class="mt-2 text-[12.5px] leading-relaxed text-text-muted">
              Opening one starts <code class="text-anchor/80">claude</code> in a pseudo-terminal in
              <span class="font-mono text-[11.5px]">{{ folder ?? 'this project’s folder' }}</span
              >, loads the task context with <code class="text-anchor/80">/rk</code>, and hands it to you.
            </p>
            <p v-if="!folder" class="mt-3 text-[11.5px] text-warn">
              Set this project's folder on its page first.
            </p>
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
