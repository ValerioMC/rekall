<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { Terminal as Xterm } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import '@xterm/xterm/css/xterm.css'
import { useTerminalSocket } from '@/features/claude/composables/useTerminalSocket'
import type { TerminalEndedFrame } from '@/features/claude/model/terminal'
import type { TerminalId } from '@/model/branded'

/**
 * One live terminal drawn with xterm and wired to its PTY over the socket. It follows
 * `terminalId`: a new id reconnects, null disconnects. Whoever hosts it decides the chrome.
 */
const props = defineProps<{ terminalId: TerminalId | null }>()

const emit = defineEmits<{ ended: [frame: TerminalEndedFrame]; connection: [connected: boolean] }>()

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

const host = ref<HTMLElement | null>(null)

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
    xterm?.write(`\r\n\x1b[2m[ ${frame.detail || 'session ended'} ]\x1b[0m\r\n`)
    emit('ended', frame)
  },
  onClose: () => {}
})

watch(
  () => socket.connected.value ?? false,
  (connected) => emit('connection', connected)
)

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
 * The first fit on a fresh terminal runs before the Fira Code web font has loaded, so xterm
 * sizes its cell against fallback metrics. Fit now for layout, then again once the font is in.
 */
function refitWhenReady(): void {
  refit()
  document.fonts?.ready.then(() => refit())
}

function connectTo(id: TerminalId): void {
  if (!host.value) return
  const term = ensureXterm()
  term.reset()
  socket.connect(id)
  connectedId = id
  refitWhenReady()
  term.focus()
}

/** Brings the connection in line with `terminalId`: the single place it (re)connects. */
function syncConnection(): void {
  const id = props.terminalId
  if (id) {
    if (id !== connectedId) connectTo(id)
  } else if (connectedId) {
    socket.disconnect()
    connectedId = null
    xterm?.reset()
  }
}

watch(() => props.terminalId, syncConnection)

onMounted(() => {
  syncConnection()
  resizeObserver = new ResizeObserver(() => refit())
  if (host.value) resizeObserver.observe(host.value)
})

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

defineExpose({ focus: () => xterm?.focus() })
</script>

<template>
  <div ref="host" class="terminal-screen h-full w-full" data-testid="terminal-host" />
</template>

<style scoped>
.terminal-screen :deep(.xterm) {
  height: 100%;
}
.terminal-screen :deep(.xterm-viewport) {
  background-color: transparent !important;
}
</style>
