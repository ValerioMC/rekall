<script setup lang="ts">
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import { useToastStore } from '@/stores/toast.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useOpenSession } from '@/features/claude/composables/useOpenSession'
import { sessionTitle } from '@/common/format/session-title'
import { identityHue } from '@/common/identity'
import { recordLatestCommit } from '@/features/commits/api/commitReference.api'
import DockPanel from './DockPanel.vue'
import TerminalConnectionChips from '@/features/connection/components/TerminalConnectionChips.vue'
import type { Terminal } from '@/features/claude/model/terminal'
import type { TerminalId } from '@/model/branded'

const emit = defineEmits<{ close: [] }>()

const terminals = useTerminalStore()
const toast = useToastStore()
const { terminals: allTerminals } = storeToRefs(terminals)
const { run } = useAsyncAction()
const openSession = useOpenSession()

const live = computed(() => allTerminals.value.filter((terminal) => terminal.live))

const sorted = computed(() => [...live.value].sort((a, b) => b.startedAt.localeCompare(a.startedAt)))

function jumpTo(terminal: Terminal): void {
  openSession(terminal)
  emit('close')
}

async function stop(id: TerminalId): Promise<void> {
  await run(() => terminals.close(id))
}

// The commit's own subject line is the comment: same lookup `rekall_record_commit` uses from
// inside the session, triggered here instead of after a `git commit` the console never sees.
async function logCommit(terminal: Terminal): Promise<void> {
  const taskId = terminal.taskId
  if (!taskId) return
  const logged = await run(() => recordLatestCommit(taskId, terminal.stepId))
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
        class="flex items-center gap-2.5 rounded-control px-2 py-2 hover:bg-surface-raised"
      >
        <button
          class="focus-ring flex min-w-0 flex-1 items-center gap-2.5 rounded-control text-left"
          :title="`Jump to ${sessionTitle(terminal)}`"
          data-testid="terminal-dock-jump"
          @click="jumpTo(terminal)"
        >
          <span class="min-w-0 flex-1">
            <span class="block truncate text-small font-medium text-text">
              {{ sessionTitle(terminal) }}
            </span>
            <span class="mt-0.5 flex items-center gap-1.5 truncate font-mono text-micro text-anchor/80">
              <span
                class="size-1 shrink-0 rounded-full"
                :style="{ backgroundColor: identityHue(terminal.projectId).base }"
                aria-hidden="true"
              />
              <span class="min-w-0 truncate">{{ terminal.anchors }}</span>
            </span>
          </span>
        </button>
        <span class="shrink-0 font-mono text-micro text-text-subtle">
          {{ terminal.skipPermissions ? 'skip perms' : 'live' }}
        </span>
        <button
          v-if="terminal.taskId"
          class="focus-ring grid size-6 shrink-0 place-items-center rounded-full text-text-subtle transition-colors hover:bg-surface-raised hover:text-anchor"
          :aria-label="`Log the latest commit for ${sessionTitle(terminal)}`"
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
          :aria-label="`Stop ${sessionTitle(terminal)}`"
          data-testid="terminal-dock-stop"
          @click="stop(terminal.id)"
        >
          <svg class="size-3" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
            <rect x="4" y="4" width="8" height="8" rx="1.5" />
          </svg>
        </button>
      </div>
      <TerminalConnectionChips
        v-if="terminal.connections.length"
        class="px-2 pb-1.5"
        :terminal-id="terminal.id"
        :project-id="terminal.projectId"
        :connections="terminal.connections"
      />
    </li>
  </DockPanel>
</template>
