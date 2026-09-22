<script setup lang="ts">
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import { useRouter } from 'vue-router'
import { useConsoleStore } from '@/stores/console.store'
import { useTerminalStore } from '@/stores/terminal.store'
import { useToastStore } from '@/stores/toast.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { identityHue } from '@/common/identity'
import { recordLatestCommit } from '@/api/commitReference.api'
import DockPanel from './DockPanel.vue'
import type { Terminal } from '@/model/terminal'
import type { TerminalId } from '@/model/branded'

const emit = defineEmits<{ close: [] }>()

const store = useConsoleStore()
const terminals = useTerminalStore()
const toast = useToastStore()
const { terminals: allTerminals } = storeToRefs(terminals)
const { run } = useAsyncAction()
const router = useRouter()

const live = computed(() => allTerminals.value.filter((terminal) => terminal.live))

const sorted = computed(() => [...live.value].sort((a, b) => b.startedAt.localeCompare(a.startedAt)))

function projectIdOf(terminal: Terminal): string {
  return store.tasks.find((task) => task.id === terminal.taskId)?.projectId ?? terminal.taskId
}

function jumpTo(terminal: Terminal): void {
  store.selectTask(terminal.taskId)
  store.openTerminal()
  terminals.select(terminal.id)
  emit('close')
  if (router.currentRoute.value.name !== 'console') void router.push({ name: 'console' })
}

async function stop(id: TerminalId): Promise<void> {
  await run(() => terminals.close(id))
}

// The commit's own subject line is the comment: same lookup `rekall_record_commit` uses from
// inside the session, triggered here instead of after a `git commit` the console never sees.
async function logCommit(terminal: Terminal): Promise<void> {
  const logged = await run(() => recordLatestCommit(terminal.taskId, terminal.stepId))
  if (logged) toast.notify(`Logged “${logged.comment}”.`)
}
</script>

<template>
  <DockPanel
    panel-id="dock-panel-sessions"
    title="Claude sessions"
    :count="live.length"
    tint="session"
    @close="emit('close')"
  >
    <li v-for="terminal in sorted" :key="terminal.id" data-testid="terminal-dock-row">
      <div
        class="flex items-center gap-2.5 rounded-[var(--radius-control)] px-2 py-2 hover:bg-surface-raised"
      >
        <button
          class="focus-ring flex min-w-0 flex-1 items-center gap-2.5 rounded-[var(--radius-control)] text-left"
          :title="`Jump to ${terminal.taskTitle}`"
          data-testid="terminal-dock-jump"
          @click="jumpTo(terminal)"
        >
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
        <span class="shrink-0 font-mono text-[10.5px] text-text-subtle">
          {{ terminal.skipPermissions ? 'skip perms' : 'live' }}
        </span>
        <button
          class="focus-ring grid size-6 shrink-0 place-items-center rounded-full text-text-subtle transition-colors hover:bg-surface-raised hover:text-anchor"
          :aria-label="`Log the latest commit for ${terminal.taskTitle}`"
          title="Log the latest commit"
          data-testid="terminal-dock-log-commit"
          @click="logCommit(terminal)"
        >
          <svg
            class="size-3"
            viewBox="0 0 16 16"
            fill="none"
            stroke="currentColor"
            stroke-width="1.4"
            aria-hidden="true"
          >
            <circle cx="8" cy="8" r="2.75" />
            <path d="M1 8h3.2M11.8 8H15" stroke-linecap="round" />
          </svg>
        </button>
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
  </DockPanel>
</template>
