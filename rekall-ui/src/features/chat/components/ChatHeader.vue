<script setup lang="ts">
import { computed, ref } from 'vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import TerminalConnectionChips from '@/features/connection/components/TerminalConnectionChips.vue'
import { stepChip } from '@/features/chat/lib/step-chip'
import type { ChatStatus } from '@/features/chat/lib/chat-status'
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
}>()

const emit = defineEmits<{
  'select-task': [taskId: TaskId]
  switch: [id: TerminalId]
  'toggle-expand': []
  hide: []
  stop: []
}>()

interface StatusLook {
  readonly label: string
  readonly tone: string
  readonly pulses: boolean
}

const STATUS_LOOKS: Readonly<Record<ChatStatus, StatusLook>> = {
  connecting: { label: 'connecting', tone: 'text-accent', pulses: false },
  working: { label: 'working', tone: 'text-safe', pulses: true },
  idle: { label: 'idle', tone: 'text-safe', pulses: false },
  waiting: { label: 'waiting for approval', tone: 'text-warn', pulses: true },
  reconnecting: { label: 'reconnecting', tone: 'text-warn', pulses: false },
  ended: { label: 'ended', tone: 'text-text-subtle', pulses: false }
}

const switcherOpen = ref(false)
const confirmingStop = ref(false)

const look = computed(() => STATUS_LOOKS[props.status])
const title = computed(() => props.session.taskTitle ?? `Project ${props.session.projectLabel}`)
const step = computed(() => stepChip(props.session.stepId, props.steps))

function selectTask(): void {
  if (props.session.taskId) emit('select-task', props.session.taskId)
}

function switchTo(terminal: Terminal): void {
  switcherOpen.value = false
  emit('switch', terminal.id)
}

function confirmStop(): void {
  confirmingStop.value = false
  emit('stop')
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
          <span class="h-2.5 w-[3px] shrink-0 rounded-full bg-accent" aria-hidden="true" />
          Chat
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
          <span
            class="inline-flex shrink-0 items-center gap-1.5 rounded-key bg-surface-raised px-2 py-0.5 text-micro font-semibold tracking-[0.02em]"
            :class="look.tone"
            data-testid="chat-status"
            :data-status="status"
          >
            <span v-if="look.pulses" class="relative grid size-2 place-items-center" aria-hidden="true">
              <span class="absolute inline-flex size-2 animate-ping rounded-full bg-current/50" />
              <span class="relative inline-flex size-1.5 rounded-full bg-current" />
            </span>
            {{ look.label }}
          </span>
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
        <AppButton variant="quiet" size="sm" data-testid="chat-hide" @click="emit('hide')">Hide</AppButton>
        <AppButton variant="danger" size="sm" data-testid="chat-stop-session" @click="confirmingStop = true">
          Stop session
        </AppButton>
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
    </div>

    <AppConfirm
      v-if="confirmingStop"
      title="Stop this session?"
      body="Claude stops where it is. The conversation stays in the task's history, but this window cannot resume it."
      :blast="session.anchors"
      confirm-label="Stop session"
      @cancel="confirmingStop = false"
      @confirm="confirmStop"
    />
  </header>
</template>
