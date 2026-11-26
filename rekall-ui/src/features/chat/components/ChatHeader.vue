<script setup lang="ts">
import { computed, ref } from 'vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import ChatSessionReadout from './ChatSessionReadout.vue'
import ChatStatusBadge from './ChatStatusBadge.vue'
import ChatStopButton from './ChatStopButton.vue'
import TerminalConnectionChips from '@/features/connection/components/TerminalConnectionChips.vue'
import SessionProviderMark from '@/features/claude/components/SessionProviderMark.vue'
import { SESSION_PROVIDER_LABEL } from '@/features/claude/model/session-provider'
import { stepChip } from '@/features/chat/lib/step-chip'
import { sessionSubject } from '@/common/format/session-title'
import type { ChatStatus } from '@/features/chat/lib/chat-status'
import type { TurnContext } from '@/features/chat/model/chat-event'
import type { TaskStep } from '@/features/steps/model/task-step'
import type { Terminal } from '@/features/claude/model/terminal'
import type { TaskId, TerminalId } from '@/model/branded'

/**
 * Who the session is for and what it is doing: the task it works on, the step it covers, how it
 * was started, and the keys that move, enlarge, hide or stop the window.
 */
const props = defineProps<{
  session: Terminal
  status: ChatStatus
  others: readonly Terminal[]
  steps: readonly TaskStep[]
  expanded: boolean
  context: TurnContext | null
  cost: number
}>()

const emit = defineEmits<{
  'select-task': [taskId: TaskId]
  switch: [id: TerminalId]
  'toggle-expand': []
  hide: []
  stop: []
}>()

const switcherOpen = ref(false)

const providerLabel = computed(() => SESSION_PROVIDER_LABEL[props.session.provider])
const title = computed(() => sessionSubject(props.session))
const step = computed(() => stepChip(props.session.stepId, props.steps))

function selectTask(): void {
  if (props.session.taskId) emit('select-task', props.session.taskId)
}

function switchTo(terminal: Terminal): void {
  switcherOpen.value = false
  emit('switch', terminal.id)
}

function onSwitcherKeydown(event: KeyboardEvent): void {
  if (event.key !== 'Escape' || !switcherOpen.value) return
  event.stopPropagation()
  switcherOpen.value = false
}
</script>

<template>
  <header class="shrink-0 border-b border-border" data-testid="chat-header">
    <div class="flex min-h-(--spacing-header) items-start gap-2 px-4 py-2">
      <div class="min-w-0 flex-1">
        <p class="eyebrow flex items-center gap-1.5">
          <SessionProviderMark :provider="session.provider" :size="12" />
          {{ providerLabel }} Chat
        </p>
        <div class="flex min-w-0 items-center gap-2">
          <button
            v-if="session.taskId"
            type="button"
            class="focus-ring min-w-0 cursor-pointer truncate rounded-control text-left text-title font-semibold text-text hover:text-accent"
            data-testid="chat-task-link"
            @click="selectTask"
          >
            {{ title }}
          </button>
          <h2 v-else class="min-w-0 truncate text-title font-semibold text-text">{{ title }}</h2>
          <ChatStatusBadge :status="status" />
        </div>
        <p class="truncate font-mono text-micro text-anchor/80">{{ session.anchors }}</p>
      </div>

      <div class="flex shrink-0 items-center gap-1.5">
        <div v-if="others.length > 0" class="relative" @keydown="onSwitcherKeydown">
          <AppButton
            variant="quiet"
            size="sm"
            :pressed="switcherOpen || undefined"
            aria-haspopup="true"
            :aria-expanded="switcherOpen"
            data-testid="chat-switch"
            @click="switcherOpen = !switcherOpen"
          >
            Switch
          </AppButton>
          <div
            v-if="switcherOpen"
            class="rise absolute right-0 top-[calc(100%+6px)] z-(--z-overlay) w-[280px] overflow-hidden rounded-card border border-border-strong bg-surface-raised shadow-modal"
            data-testid="chat-switch-list"
          >
            <ul class="max-h-[280px] overflow-y-auto p-1.5">
              <li v-for="other in others" :key="other.id">
                <button
                  type="button"
                  class="focus-ring flex w-full min-w-0 cursor-pointer flex-col rounded-control px-2 py-2 text-left transition-colors hover:bg-surface"
                  data-testid="chat-switch-row"
                  @click="switchTo(other)"
                >
                  <span class="truncate text-small font-medium text-text">{{ other.taskTitle ?? other.projectLabel }}</span>
                  <span class="truncate font-mono text-micro text-anchor/80">{{ other.anchors }}</span>
                </button>
              </li>
            </ul>
          </div>
          <button
            v-if="switcherOpen"
            type="button"
            class="fixed inset-0 z-(--z-sticky) cursor-default"
            tabindex="-1"
            aria-hidden="true"
            @click="switcherOpen = false"
          />
        </div>
        <AppButton variant="quiet" size="sm" data-testid="chat-expand" @click="emit('toggle-expand')">
          {{ expanded ? 'Restore' : 'Expand' }}
        </AppButton>
        <AppButton variant="quiet" size="sm" data-testid="chat-hide" @click="emit('hide')">
          Hide
          <svg class="size-3 shrink-0" viewBox="0 0 12 12" fill="none" aria-hidden="true">
            <path d="M2.5 2.5 6 6l-3.5 3.5M6.5 2.5 10 6 6.5 9.5" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
        </AppButton>
        <ChatStopButton :session="session" @stop="emit('stop')" />
      </div>
    </div>

    <div class="flex flex-wrap items-center gap-x-2 gap-y-1 border-t border-border bg-surface px-4 py-2 text-caption max-[420px]:px-3">
      <AppBadge v-if="step" tone="accent" data-testid="chat-step">{{ step }}</AppBadge>
      <AppBadge v-if="session.model" mono>{{ session.model }}</AppBadge>
      <AppBadge v-if="session.effort" mono>{{ session.effort }}</AppBadge>
      <AppBadge :tone="session.skipPermissions ? 'warn' : 'safe'" data-testid="chat-permissions">
        {{ session.skipPermissions ? 'permissions skipped' : 'permissions on' }}
      </AppBadge>
      <TerminalConnectionChips :terminal-id="session.id" :project-id="session.projectId" :connections="session.connections" />
      <ChatSessionReadout class="ml-auto" :context="context" :cost="cost" />
    </div>
  </header>
</template>
