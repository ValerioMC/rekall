<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import { useRouter } from 'vue-router'
import AppButton from '@/components/ui/AppButton.vue'
import ChatConversation from './ChatConversation.vue'
import ChatPaneHeader from './ChatPaneHeader.vue'
import ChatRunHere from './ChatRunHere.vue'
import ChatSessionFacts from './ChatSessionFacts.vue'
import ChatStatusBadge from './ChatStatusBadge.vue'
import ChatStopButton from './ChatStopButton.vue'
import SessionProviderMark from '@/features/claude/components/SessionProviderMark.vue'
import SessionSwitcher from '@/features/claude/components/SessionSwitcher.vue'
import { sessionSubject } from '@/common/format/session-title'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useShownChat } from '@/features/chat/composables/useShownChat'
import { useChatStore } from '@/features/chat/store/chat.store'
import { useOpenSessionInPane } from '@/features/claude/composables/useOpenSessionInPane'
import { SESSION_PROVIDER_LABEL } from '@/features/claude/model/session-provider'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import { useConsoleStore } from '@/stores/console.store'
import type { Terminal } from '@/features/claude/model/terminal'

/**
 * A chat full-size on the console: the selected task's, or one on a project alone. It reads the
 * session list when it appears and when what it shows changes, as the terminal pane does.
 */
const chat = useChatStore()
const terminals = useTerminalStore()
const router = useRouter()
const openInPane = useOpenSessionInPane()
const { selectedTask, projectChatId } = storeToRefs(useConsoleStore())
const { activeTimeline } = storeToRefs(chat)
const { session, others, status, sessionSteps } = useShownChat()
const { run } = useAsyncAction()

const listRead = ref(false)

const title = computed(() => (session.value ? sessionSubject(session.value) : (selectedTask.value?.title ?? null)))
const taskId = computed(() => (session.value ? session.value.taskId : (selectedTask.value?.id ?? null)))

/** The selected task's chat, never another's; with no task selected, the chat named on its own. */
function pickShown(): void {
  const task = selectedTask.value
  if (!task) {
    chat.select(projectChatId.value)
    return
  }
  const found = chat.sessionFor(task.id)
  chat.select(found?.taskId === task.id ? found.id : null)
}

async function readSessions(): Promise<void> {
  pickShown()
  await run(() => terminals.load())
  listRead.value = true
  pickShown()
}

async function stopSession(stopped: Terminal): Promise<void> {
  await run(() => terminals.close(stopped.id))
  chat.select(null)
  if (!stopped.taskId) await router.push(`/projects/${stopped.projectId}`)
}

watch([() => selectedTask.value?.id ?? null, projectChatId], readSessions, { immediate: true })
</script>

<template>
  <section
    class="@container/chat flex min-h-0 w-full min-w-0 flex-1 flex-col bg-canvas"
    aria-label="Chat"
    data-testid="chat-pane"
  >
    <ChatPaneHeader v-if="title" :title="title" :task-id="taskId">
      <template #status>
        <ChatStatusBadge v-if="session" :status="status" />
      </template>
      <template #detail>
        <template v-if="session">
          <SessionProviderMark :provider="session.provider" :size="12" />
          <span>{{ SESSION_PROVIDER_LABEL[session.provider] }} chat</span>
          <span class="h-[11px] w-px shrink-0 bg-border-strong" aria-hidden="true" />
        </template>
        <span class="truncate font-mono text-anchor">{{ session?.anchors ?? selectedTask?.anchor }}</span>
      </template>
      <template v-if="session" #actions>
        <SessionSwitcher v-if="others.length > 0" :sessions="others" @switch="openInPane" />
        <ChatStopButton :session="session" @stop="stopSession(session)" />
      </template>
    </ChatPaneHeader>

    <div v-if="session" class="flex min-h-0 flex-1 flex-col @min-[1060px]/chat:flex-row">
      <ChatSessionFacts
        :session="session"
        :steps="sessionSteps"
        :context="activeTimeline.context"
        :cost="activeTimeline.cost"
      />
      <div class="@container/talk flex min-h-0 min-w-0 flex-1 flex-col">
        <ChatConversation :session="session" :status="status" />
      </div>
    </div>

    <ChatRunHere v-else-if="listRead && selectedTask" @opened="openInPane" />

    <div v-else-if="listRead" class="grid min-h-0 flex-1 place-items-center px-6 text-center" data-testid="chat-closed">
      <div class="flex flex-col items-center gap-4">
        <h2 class="text-title font-semibold text-text">This chat session has closed</h2>
        <AppButton data-testid="chat-closed-projects" @click="router.push('/projects')">Go to projects</AppButton>
      </div>
    </div>
  </section>
</template>
