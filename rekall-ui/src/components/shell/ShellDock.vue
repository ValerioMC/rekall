<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import { useRoute } from 'vue-router'
import { useConsoleStore } from '@/stores/console.store'
import { useChatWindowShown } from '@/features/chat/composables/useChatWindowShown'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import RunningTasksPanel from './RunningTasksPanel.vue'
import TerminalSessionsPanel from './TerminalSessionsPanel.vue'

/**
 * Live work: one capsule in the bottom left corner of every screen, two segments, one panel open
 * at a time. The sheet belongs to whichever segment was pressed last: pressing the other
 * switches; pressing the same one again, Escape, or a click anywhere else closes it. A segment
 * that has nothing to show leaves, and takes its sheet with it; when neither has anything, so
 * does the capsule.
 */
type DockPanel = 'running' | 'sessions'

const store = useConsoleStore()
const terminals = useTerminalStore()
const { runningEntries, paneFocus } = storeToRefs(store)
const { terminals: allTerminals } = storeToRefs(terminals)
const route = useRoute()
const root = ref<HTMLElement | null>(null)
const chatWindowShown = useChatWindowShown()

const open = ref<DockPanel | null>(null)

const liveSessions = computed(() => allTerminals.value.filter((terminal) => terminal.live))

// The sessions segment steps aside while the terminal pane or the chat window is on screen: they show the sessions.
const onTerminalPane = computed(() => route.name === 'console' && paneFocus.value === 'terminal')

const showsRunning = computed(() => runningEntries.value.length > 0)
const showsSessions = computed(() => liveSessions.value.length > 0 && !onTerminalPane.value && !chatWindowShown.value)
const visible = computed(() => showsRunning.value || showsSessions.value)

const awaitingApproval = computed(() => liveSessions.value.some((terminal) => terminal.awaitingPermission))
const sessionsLabel = computed(() =>
  awaitingApproval.value
    ? 'approve'
    : `${liveSessions.value.length} ${liveSessions.value.length === 1 ? 'session' : 'sessions'}`
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
    class="fixed bottom-2.5 left-4 z-(--z-sticky) flex flex-col items-start gap-2"
    data-testid="shell-dock"
  >
    <Transition name="popover" mode="out-in">
      <RunningTasksPanel v-if="open === 'running'" class="w-[320px]" @close="close" />
      <TerminalSessionsPanel v-else-if="open === 'sessions'" class="w-[320px]" @close="close" />
    </Transition>

    <div
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
        <span class="whitespace-nowrap">{{ runningEntries.length }} in progress</span>
      </button>

      <span v-if="showsRunning && showsSessions" class="dock-divider" aria-hidden="true" />

      <button
        v-if="showsSessions"
        class="dock-segment dock-segment-session"
        :class="awaitingApproval && 'text-warn'"
        :aria-expanded="open === 'sessions'"
        aria-controls="dock-panel-sessions"
        data-testid="terminal-dock-toggle"
        @click="toggle('sessions')"
      >
        <span class="session-caret shrink-0" aria-hidden="true" />
        <span class="whitespace-nowrap">{{ sessionsLabel }}</span>
      </button>
    </div>
  </div>
</template>
