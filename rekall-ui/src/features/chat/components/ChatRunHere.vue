<script setup lang="ts">
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import AppButton from '@/components/ui/AppButton.vue'
import ChatGlyph from './ChatGlyph.vue'
import { launchChoices } from '@/common/config/session-launch'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useRunProvider } from '@/features/claude/composables/useRunProvider'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import { useConsoleStore } from '@/stores/console.store'
import type { Terminal } from '@/features/claude/model/terminal'

/** The selected task has no chat session: what it would show, and the way to start one. */
const props = defineProps<{
  /** Called, not emitted: the new chat can take this view off screen before the server answers. */
  onOpened: (session: Terminal) => void
}>()

const terminals = useTerminalStore()
const { selectedTask } = storeToRefs(useConsoleStore())
const { isRunning, run } = useAsyncAction()
const { provider } = useRunProvider(() => selectedTask.value?.id ?? null)
const folder = computed(() => selectedTask.value?.projectRepoFolder ?? null)

async function openHere(): Promise<void> {
  const task = selectedTask.value
  if (!task || !folder.value || isRunning.value) return
  const opened = await run(
    () => terminals.openForTask(task.id, { ...launchChoices(provider.value), kind: 'CHAT' }),
    'Session opened'
  )
  if (opened) props.onOpened(opened)
}
</script>

<template>
  <div class="grid min-h-0 flex-1 place-items-center px-6 py-6 text-center" data-testid="chat-empty">
    <div class="flex flex-col items-center">
      <span
        class="grid h-[52px] w-16 place-items-center rounded-card border border-border-strong bg-surface text-text-subtle"
        aria-hidden="true"
      >
        <ChatGlyph class="size-7" />
      </span>
      <h3 class="mt-4 text-title font-semibold text-text">No chat session on this task</h3>
      <p class="mt-2 mb-4 max-w-[46ch] text-small leading-relaxed text-text-muted">
        Follow the conversation, tool calls and approval requests here.
      </p>
      <AppButton
        v-if="selectedTask && folder"
        variant="primary"
        class="h-9 px-4"
        :loading="isRunning"
        data-testid="chat-open"
        @click="openHere"
      >
        Run here
      </AppButton>
      <RouterLink
        v-else-if="selectedTask"
        :to="`/projects/${selectedTask.projectId}`"
        class="focus-ring inline-flex items-center gap-1.5 rounded-control border border-warn/40 bg-warn-soft px-3 py-1.5 text-caption font-medium text-warn"
        data-testid="chat-set-folder"
      >
        Set this project's folder on its page first
        <span aria-hidden="true">&rarr;</span>
      </RouterLink>
    </div>
  </div>
</template>
