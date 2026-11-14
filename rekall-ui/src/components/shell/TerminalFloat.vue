<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'
import TerminalScreen from '@/features/claude/components/TerminalScreen.vue'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import { useOpenSession } from '@/features/claude/composables/useOpenSession'
import { sessionTitle } from '@/common/format/session-title'
import type { Terminal } from '@/features/claude/model/terminal'

/**
 * One terminal session in a panel above the dock instead of the full terminal pane. It never closes
 * on Escape or an outside press: both belong to the TUI running inside it.
 */
const props = defineProps<{ terminal: Terminal }>()

const emit = defineEmits<{ close: [] }>()

const terminals = useTerminalStore()
const openSession = useOpenSession()
const router = useRouter()

const connected = ref(false)
const ended = ref(false)

const status = computed(() => (connected.value ? 'connected' : ended.value || !props.terminal.live ? 'ended' : 'connecting'))

function onEnded(): void {
  ended.value = true
  terminals.markEnded(props.terminal.id)
}

function openTask(): void {
  if (props.terminal.taskId) void router.push(`/task/${props.terminal.taskId}`)
  emit('close')
}

function openInPane(): void {
  openSession(props.terminal)
  emit('close')
}
</script>

<template>
  <section
    class="terminal-float flex h-[420px] max-h-[calc(100vh-7rem)] w-[640px] max-w-[calc(100vw-2rem)] flex-col overflow-hidden rounded-card border border-border-strong bg-canvas shadow-modal"
    aria-label="Terminal session"
    data-testid="terminal-float"
  >
    <header class="flex h-10 shrink-0 items-center gap-2.5 border-b border-border bg-surface pl-4 pr-3">
      <span class="session-caret shrink-0" aria-hidden="true" />
      <span
        class="shrink-0 rounded-key bg-surface-raised px-2 py-0.5 text-micro font-semibold"
        :class="status === 'connected' ? 'text-safe' : status === 'ended' ? 'text-text-subtle' : 'text-accent'"
        data-testid="terminal-float-status"
      >
        {{ status }}
      </span>
      <h2 class="min-w-0 flex-1 truncate text-small font-medium text-text-muted">
        {{ sessionTitle(terminal) }}
      </h2>
      <button
        v-if="terminal.taskId"
        class="key-quiet rounded-key focus-ring inline-flex h-7 shrink-0 items-center px-2.5 text-caption"
        data-testid="terminal-float-task"
        @click="openTask"
      >
        Open task
      </button>
      <button
        class="key-quiet rounded-key focus-ring inline-flex h-7 shrink-0 items-center px-2.5 text-caption"
        data-testid="terminal-float-expand"
        @click="openInPane"
      >
        Open in pane
      </button>
      <button
        class="focus-ring grid size-6 shrink-0 place-items-center rounded-full text-text-subtle transition-colors hover:bg-surface-raised hover:text-text"
        aria-label="Close"
        data-testid="terminal-float-close"
        @click="emit('close')"
      >
        <svg class="size-3" viewBox="0 0 12 12" fill="none" aria-hidden="true">
          <path d="M3 3l6 6M9 3l-6 6" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
        </svg>
      </button>
    </header>

    <div class="min-h-0 flex-1 bg-[#0b0b0d] px-3 py-2">
      <TerminalScreen :terminal-id="terminal.id" @ended="onEnded" @connection="connected = $event" />
    </div>
  </section>
</template>
