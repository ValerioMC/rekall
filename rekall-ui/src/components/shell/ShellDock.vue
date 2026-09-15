<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import { useRoute } from 'vue-router'
import { useConsoleStore } from '@/stores/console.store'
import { useTerminalStore } from '@/stores/terminal.store'
import { useDockLane } from '@/composables/useDockLane'
import { useNow } from '@/composables/useNow'
import { formatClock } from '@/common/format/duration'
import RunningTasksPanel from './RunningTasksPanel.vue'
import TerminalSessionsPanel from './TerminalSessionsPanel.vue'

/**
 * The bottom right corner: one bar, two segments, one panel open at a time.
 *
 * Running timers and live Claude sessions used to be two pills stacked in this corner, each
 * with its own sheet, so opening both put one sheet across the other and only the lower pill
 * told the rest of the interface how much room it took. Here they are two segments of a single
 * bar, the bar is the one thing that publishes the lane (`useDockLane`), and the sheet above
 * it belongs to whichever segment was pressed last. Pressing the other switches; pressing the
 * same one again, Escape, or a click anywhere else closes it. A segment that has nothing to
 * show leaves, and takes its sheet with it; when neither has anything, so does the bar.
 */
type DockPanel = 'running' | 'sessions'

const store = useConsoleStore()
const terminals = useTerminalStore()
const { runningEntries, paneFocus } = storeToRefs(store)
const { terminals: allTerminals } = storeToRefs(terminals)
const route = useRoute()
const now = useNow()
const bar = useDockLane()
const root = ref<HTMLElement | null>(null)

const open = ref<DockPanel | null>(null)

const liveSessions = computed(() => allTerminals.value.filter((terminal) => terminal.live))

// The sessions segment steps aside while the terminal pane is on screen: it is what the pane shows.
const onTerminalPane = computed(() => route.name === 'console' && paneFocus.value === 'terminal')

const showsRunning = computed(() => runningEntries.value.length > 0)
const showsSessions = computed(() => liveSessions.value.length > 0 && !onTerminalPane.value)
const visible = computed(() => showsRunning.value || showsSessions.value)

const latestRunning = computed(
  () => [...runningEntries.value].sort((a, b) => b.startedAt.localeCompare(a.startedAt))[0] ?? null
)

const latestClock = computed(() =>
  latestRunning.value ? formatClock((now.value - Date.parse(latestRunning.value.startedAt)) / 1000) : null
)

const sessionsLabel = computed(
  () => `${liveSessions.value.length} ${liveSessions.value.length === 1 ? 'session' : 'sessions'}`
)

function toggle(panel: DockPanel): void {
  open.value = open.value === panel ? null : panel
}

function close(): void {
  open.value = null
}

watch([showsRunning, showsSessions], ([running, sessions]) => {
  if (open.value === 'running' && !running) close()
  if (open.value === 'sessions' && !sessions) close()
})

function onPointerDown(event: PointerEvent): void {
  if (!open.value || !root.value) return
  if (!root.value.contains(event.target as Node)) close()
}

function onKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape' && open.value) close()
}

onMounted(() => {
  document.addEventListener('pointerdown', onPointerDown)
  document.addEventListener('keydown', onKeydown)
})

onUnmounted(() => {
  document.removeEventListener('pointerdown', onPointerDown)
  document.removeEventListener('keydown', onKeydown)
})
</script>

<template>
  <div
    v-if="visible"
    ref="root"
    class="fixed bottom-2.5 right-4 z-(--z-sticky) flex flex-col items-end gap-2"
    data-testid="shell-dock"
  >
    <RunningTasksPanel v-if="open === 'running'" @close="close" />
    <TerminalSessionsPanel v-else-if="open === 'sessions'" @close="close" />

    <div
      ref="bar"
      class="dock-bar glass flex h-10 items-stretch overflow-hidden rounded-full border border-border-strong shadow-lift"
      role="toolbar"
      aria-label="Live work"
      data-testid="shell-dock-bar"
    >
      <button
        v-if="showsRunning"
        class="dock-segment dock-segment-time"
        :aria-expanded="open === 'running'"
        aria-controls="dock-panel-running"
        data-testid="running-dock-toggle"
        @click="toggle('running')"
      >
        <span class="time-dial shrink-0" aria-hidden="true" />
        <span class="whitespace-nowrap">{{ runningEntries.length }} running</span>
        <span v-if="latestClock" class="font-mono text-[12px] tabular-nums text-accent">
          {{ latestClock }}
        </span>
      </button>

      <span v-if="showsRunning && showsSessions" class="dock-divider" aria-hidden="true" />

      <button
        v-if="showsSessions"
        class="dock-segment dock-segment-session"
        :aria-expanded="open === 'sessions'"
        aria-controls="dock-panel-sessions"
        data-testid="terminal-dock-toggle"
        @click="toggle('sessions')"
      >
        <span class="session-caret session-caret-busy shrink-0" aria-hidden="true" />
        <span class="whitespace-nowrap">{{ sessionsLabel }}</span>
      </button>
    </div>
  </div>
</template>
