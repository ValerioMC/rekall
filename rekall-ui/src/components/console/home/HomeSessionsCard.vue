<script setup lang="ts">
/** The Claude sessions open right now, each one click from its terminal, and what the run queue starts next. */
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import { useRunQueueStore } from '@/features/queue/store/runQueue.store'
import { useNow } from '@/composables/useNow'
import { formatDuration } from '@/common/format/duration'
import { sessionTitle } from '@/common/format/session-title'
import { useOpenSession } from '@/features/claude/composables/useOpenSession'
import type { Terminal } from '@/features/claude/model/terminal'

const terminals = useTerminalStore()
const runQueue = useRunQueueStore()
const { terminals: allTerminals } = storeToRefs(terminals)
const { queue } = storeToRefs(runQueue)
const now = useNow(30_000)

const live = computed(() => allTerminals.value.filter((terminal) => terminal.live))
const nextQueued = computed(() => queue.value.items.find((item) => item.state === 'QUEUED') ?? null)

const openSession = useOpenSession()

function elapsed(terminal: Terminal): string {
  return formatDuration((now.value - Date.parse(terminal.startedAt)) / 1000)
}
</script>

<template>
  <section
    class="flex flex-col rounded-card border border-border bg-surface p-1.5 shadow-(--shadow-raised)"
    aria-labelledby="home-sessions-title"
    data-testid="home-sessions"
  >
    <header class="flex items-baseline justify-between gap-3 px-2.5 pb-1.5 pt-2">
      <h2 id="home-sessions-title" class="eyebrow uppercase tracking-[0.12em]">Running now</h2>
      <span class="text-caption text-text-subtle">{{ live.length }} open</span>
    </header>

    <ul v-if="live.length" class="flex flex-col">
      <li v-for="terminal in live" :key="terminal.id">
        <button
          type="button"
          class="focus-ring flex w-full items-center gap-3 rounded-control px-2.5 py-2 text-left transition-colors duration-(--motion-quick) hover:bg-surface-raised"
          data-testid="home-session"
          @click="openSession(terminal)"
        >
          <span class="mx-1.5 size-2 shrink-0 rounded-full bg-safe shadow-[0_0_0_4px_var(--color-safe-soft)]" aria-hidden="true" />
          <span class="min-w-0 flex-1">
            <span class="block truncate text-body text-text">{{ sessionTitle(terminal) }}</span>
            <span class="mt-0.5 block truncate font-mono text-caption text-anchor">{{ terminal.anchors }}</span>
          </span>
          <span class="shrink-0 font-mono text-caption tabular-nums text-safe">{{ elapsed(terminal) }}</span>
        </button>
      </li>
    </ul>

    <p v-else class="px-2.5 pb-2 pt-1 text-small leading-relaxed text-text-muted">
      No session open. Start one with Run here on a task's steps.
    </p>

    <button
      v-if="nextQueued"
      type="button"
      class="focus-ring mx-2.5 mb-2 mt-1 rounded-control border border-dashed border-border-strong px-2.5 py-2 text-left text-small text-text-muted transition-colors duration-(--motion-quick) hover:border-border-field hover:text-text"
      data-testid="home-next-queued"
      @click="runQueue.openPanel()"
    >
      Next in the run queue:
      <span class="font-mono text-anchor">{{ nextQueued.anchor }}</span>
    </button>
  </section>
</template>
