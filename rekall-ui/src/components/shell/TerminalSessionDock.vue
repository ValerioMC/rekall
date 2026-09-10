<script setup lang="ts">
import { computed, ref } from 'vue'
import { storeToRefs } from 'pinia'
import { useRoute, useRouter } from 'vue-router'
import { useConsoleStore } from '@/stores/console.store'
import { useTerminalStore } from '@/stores/terminal.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { identityHue } from '@/common/identity'
import type { Terminal } from '@/model/terminal'
import type { TerminalId } from '@/model/branded'

const store = useConsoleStore()
const terminals = useTerminalStore()
const { terminals: allTerminals } = storeToRefs(terminals)
const { paneFocus } = storeToRefs(store)
const { run } = useAsyncAction()
const route = useRoute()
const router = useRouter()

const expanded = ref(false)

const live = computed(() => allTerminals.value.filter((terminal) => terminal.live))

const onTerminalPane = computed(() => route.name === 'console' && paneFocus.value === 'terminal')

const visible = computed(() => live.value.length > 0 && !onTerminalPane.value)

const sorted = computed(() =>
  [...live.value].sort((a, b) => b.startedAt.localeCompare(a.startedAt))
)

function projectIdOf(terminal: Terminal): string {
  return store.tasks.find((task) => task.id === terminal.taskId)?.projectId ?? terminal.taskId
}

function jumpTo(terminal: Terminal): void {
  store.selectTask(terminal.taskId)
  store.openTerminal()
  terminals.select(terminal.id)
  expanded.value = false
  if (router.currentRoute.value.name !== 'console') void router.push({ name: 'console' })
}

async function stop(id: TerminalId): Promise<void> {
  await run(() => terminals.close(id))
}
</script>

<template>
  <div
    v-if="visible"
    class="dock-lane-above fixed right-4 z-(--z-sticky) flex flex-col items-end gap-2"
    data-testid="terminal-dock"
  >
    <div
      v-if="expanded"
      class="rise w-[300px] overflow-hidden rounded-[var(--radius-card)] border border-border-strong bg-surface shadow-modal"
    >
      <header class="flex items-center justify-between border-b border-border px-3.5 py-2.5">
        <span class="eyebrow flex items-center gap-1.5">
          <span class="h-2.5 w-[3px] shrink-0 rounded-full bg-anchor" aria-hidden="true" />
          Sessions
        </span>
        <button
          class="focus-ring grid size-6 place-items-center rounded-full text-text-subtle transition-colors hover:bg-surface-raised hover:text-text"
          aria-label="Collapse"
          @click="expanded = false"
        >
          &times;
        </button>
      </header>

      <ul class="max-h-[280px] overflow-y-auto p-1.5">
        <li v-for="terminal in sorted" :key="terminal.id" data-testid="terminal-dock-row">
          <div class="flex items-center gap-2.5 rounded-[var(--radius-control)] px-2 py-2 hover:bg-surface-raised">
            <button
              class="focus-ring flex min-w-0 flex-1 items-center gap-2.5 rounded-[var(--radius-control)] text-left"
              :title="`Jump to ${terminal.taskTitle}`"
              data-testid="terminal-dock-jump"
              @click="jumpTo(terminal)"
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
                  {{ terminal.anchors }}
                </span>
              </span>
            </button>
            <span class="shrink-0 font-mono text-[10.5px] text-text-subtle">
              {{ terminal.skipPermissions ? 'skip perms' : 'live' }}
            </span>
            <button
              class="focus-ring grid size-6 shrink-0 place-items-center rounded-full text-text-subtle transition-colors hover:bg-danger-soft hover:text-danger"
              :aria-label="`Stop ${terminal.taskTitle}`"
              data-testid="terminal-dock-stop"
              @click="stop(terminal.id)"
            >
              <svg class="size-3" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
                <rect x="4" y="4" width="8" height="8" rx="1.5" />
              </svg>
            </button>
          </div>
        </li>
      </ul>
    </div>

    <button
      class="focus-ring glass flex h-10 w-[184px] items-center gap-2.5 overflow-hidden rounded-full border border-border-strong pl-2.5 pr-3.5 shadow-lift transition-colors hover:border-anchor/50"
      data-testid="terminal-dock-toggle"
      :aria-expanded="expanded"
      @click="expanded = !expanded"
    >
      <span class="session-caret session-caret-busy shrink-0" aria-hidden="true" />
      <span class="min-w-0 truncate text-[12.5px] font-medium text-text">
        {{ live.length }} {{ live.length === 1 ? 'session' : 'sessions' }}
      </span>
    </button>
  </div>
</template>
