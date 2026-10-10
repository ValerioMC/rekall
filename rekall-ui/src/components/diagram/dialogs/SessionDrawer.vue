<script setup lang="ts">
import { computed, ref } from 'vue'
import TerminalScreen from '@/components/claude/TerminalScreen.vue'
import CloseGlyph from '@/components/ui/CloseGlyph.vue'
import { sessionTitle } from '@/common/format/session-title'
import type { Terminal } from '@/model/terminal'

/**
 * The session drawing a diagram, live, in a sheet over the bottom of the screen: to watch it
 * work or answer what it asks. Closing the sheet leaves the session running.
 */
const props = defineProps<{ terminal: Terminal }>()

const emit = defineEmits<{ close: []; stop: [] }>()

const connected = ref(false)
const ended = ref<string | null>(null)
const title = computed(() => sessionTitle(props.terminal))

function onKeydown(event: KeyboardEvent): void {
  if (event.key !== 'Escape' || !event.altKey) return
  event.preventDefault()
  emit('close')
}
</script>

<template>
  <section
    class="glass absolute inset-x-4 bottom-4 z-(--z-overlay) flex h-[min(46vh,460px)] flex-col overflow-hidden rounded-card border border-border-strong shadow-modal"
    role="dialog"
    aria-modal="false"
    :aria-label="`Session: ${title}`"
    data-testid="session-drawer"
    @keydown="onKeydown"
  >
    <header class="flex shrink-0 items-center gap-3 border-b border-border px-4 py-2.5">
      <span class="size-2 shrink-0 rounded-full" :class="connected ? 'bg-safe' : ended ? 'bg-text-subtle' : 'bg-accent'" aria-hidden="true" />
      <span class="min-w-0 flex-1">
        <span class="block truncate text-small font-semibold text-text">{{ title }}</span>
        <span class="block truncate font-mono text-micro text-anchor/80">{{ terminal.anchors }}</span>
      </span>
      <span class="text-caption text-text-subtle" role="status">{{ connected ? 'connected' : ended ?? 'connecting' }}</span>
      <button class="key-danger focus-ring inline-flex h-7 items-center rounded-key px-2.5 text-caption" data-testid="session-stop" @click="emit('stop')">Stop</button>
      <button class="focus-ring grid size-7 place-items-center rounded-control text-text-subtle transition-colors hover:bg-surface-raised hover:text-text" aria-label="Hide the session (Alt+Esc)" title="Hide the session (Alt+Esc); it keeps running" data-testid="session-hide" @click="emit('close')">
        <CloseGlyph />
      </button>
    </header>
    <div class="min-h-0 flex-1 bg-[#0b0b0d] px-3 py-2">
      <TerminalScreen :terminal-id="terminal.id" @connection="connected = $event" @ended="ended = $event.detail || 'ended'" />
    </div>
  </section>
</template>
