<script setup lang="ts">
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import { useToastStore } from '@/stores/toast.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useOpenSession } from '@/features/claude/composables/useOpenSession'
import { sessionTitle } from '@/common/format/session-title'
import { recordLatestCommit } from '@/features/commits/api/commitReference.api'
import SessionProviderMark from '@/features/claude/components/SessionProviderMark.vue'
import { SESSION_PROVIDER_LABEL } from '@/features/claude/model/session-provider'
import DockEntry from './DockEntry.vue'
import DockPanel from './DockPanel.vue'
import TerminalConnectionChips from '@/features/connection/components/TerminalConnectionChips.vue'
import type { Terminal } from '@/features/claude/model/terminal'
import type { TerminalId } from '@/model/branded'

const emit = defineEmits<{ close: []; float: [terminal: Terminal] }>()

const terminals = useTerminalStore()
const toast = useToastStore()
const { terminals: allTerminals } = storeToRefs(terminals)
const { run } = useAsyncAction()
const openSession = useOpenSession()

const live = computed(() => allTerminals.value.filter((terminal) => terminal.live))

const sorted = computed(() => [...live.value].sort((a, b) => b.startedAt.localeCompare(a.startedAt)))

const groups = computed(() =>
  [
    { id: 'task', title: 'Working on a task', terminals: sorted.value.filter((terminal) => terminal.taskId) },
    { id: 'project', title: 'Not on a task', terminals: sorted.value.filter((terminal) => !terminal.taskId) }
  ].filter((group) => group.terminals.length)
)

function jumpTo(terminal: Terminal): void {
  if (terminal.kind === 'TERMINAL' && terminal.taskId) emit('float', terminal)
  else openSession(terminal)
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
  const logged = await run(() => recordLatestCommit(taskId, null))
  if (logged) toast.notify(`Logged “${logged.comment}”.`)
}
</script>

<template>
  <DockPanel
    panel-id="dock-panel-sessions"
    title="Sessions"
    :count="live.length"
    tint="session"
    @close="emit('close')"
  >
    <template v-for="group in groups" :key="group.id">
      <li
        class="px-2.5 pb-1 pt-2.5 text-caption font-medium text-text-subtle"
        :data-testid="`terminal-dock-group-${group.id}`"
      >
        {{ group.title }}
      </li>
      <DockEntry
        v-for="terminal in group.terminals"
        :key="terminal.id"
        :project-id="terminal.projectId"
        :title="sessionTitle(terminal)"
        jump-test-id="terminal-dock-jump"
        data-testid="terminal-dock-row"
        @jump="jumpTo(terminal)"
      >
        <p class="mt-0.5 truncate font-mono text-caption text-anchor">{{ terminal.anchors }}</p>
        <div class="mt-1.5 flex min-h-6 items-center gap-2 text-caption text-text-muted">
          <span class="flex items-center gap-1.5 whitespace-nowrap" data-testid="terminal-dock-provider">
            <SessionProviderMark :provider="terminal.provider" :size="12" />
            {{ SESSION_PROVIDER_LABEL[terminal.provider] }}
          </span>
          <span
            v-if="terminal.kind === 'CHAT'"
            class="shrink-0 rounded-key px-1.5 py-px text-micro font-semibold"
            :class="
              terminal.awaitingPermission
                ? 'bg-warn-soft text-warn'
                : 'bg-surface-raised text-text-muted group-hover:bg-surface-hover'
            "
            data-testid="terminal-dock-kind"
          >
            {{ terminal.awaitingPermission ? 'approve' : 'chat' }}
          </span>
          <span class="whitespace-nowrap text-text-subtle">
            {{ terminal.skipPermissions ? 'skip perms' : 'live' }}
          </span>
          <span class="flex-1" />
          <button
            v-if="terminal.taskId"
            class="focus-ring relative grid size-6 shrink-0 place-items-center rounded-full text-text-subtle transition-colors hover:bg-surface-hover hover:text-anchor"
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
            class="focus-ring relative grid size-6 shrink-0 place-items-center rounded-full text-text-subtle transition-colors hover:bg-danger-soft hover:text-danger"
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
          class="relative mt-1"
          :terminal-id="terminal.id"
          :project-id="terminal.projectId"
          :connections="terminal.connections"
        />
      </DockEntry>
    </template>
  </DockPanel>
</template>
