<script setup lang="ts">
import { computed } from 'vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import ChatFact from './ChatFact.vue'
import TerminalConnectionChips from '@/features/connection/components/TerminalConnectionChips.vue'
import { contextPercent, contextTokens, describeContext } from '@/features/chat/lib/context-readout'
import { formatCost } from '@/features/chat/lib/format-cost'
import { sessionStep } from '@/features/chat/lib/step-chip'
import type { TurnContext } from '@/features/chat/model/chat-event'
import type { TaskStep } from '@/features/steps/model/task-step'
import type { Terminal } from '@/features/claude/model/terminal'

/**
 * What describes a full-size chat's session: a column beside the conversation from 1060 px of a
 * container named `chat`, one strip under the title below that.
 */
const props = defineProps<{
  session: Terminal
  steps: readonly TaskStep[]
  context: TurnContext | null
  cost: number
}>()

const step = computed(() => sessionStep(props.session.stepId, props.steps))
</script>

<template>
  <dl
    class="flex shrink-0 flex-wrap items-center gap-x-2 gap-y-1.5 border-b border-border bg-surface px-5 py-2 text-small text-text @min-[1060px]/chat:order-last @min-[1060px]/chat:block @min-[1060px]/chat:w-[272px] @min-[1060px]/chat:overflow-y-auto @min-[1060px]/chat:border-b-0 @min-[1060px]/chat:border-l @min-[1060px]/chat:bg-transparent @min-[1060px]/chat:py-1"
    data-testid="chat-facts"
  >
    <ChatFact v-if="step" label="Step">
      <AppBadge tone="accent" class="min-w-0 max-w-full" :title="step.name" data-testid="chat-step">
        <span class="truncate">{{ step.name }}</span>
      </AppBadge>
      <small v-if="step.coverage" class="hidden basis-full text-caption text-text-subtle @min-[1060px]/chat:block">
        {{ step.coverage }}
      </small>
    </ChatFact>
    <ChatFact v-if="session.model || session.effort" label="Model">
      <AppBadge v-if="session.model" mono>{{ session.model }}</AppBadge>
      <AppBadge v-if="session.effort" mono>{{ session.effort }}</AppBadge>
    </ChatFact>
    <ChatFact label="Permissions">
      <AppBadge :tone="session.skipPermissions ? 'warn' : 'safe'" data-testid="chat-permissions">
        {{ session.skipPermissions ? 'permissions skipped' : 'permissions on' }}
      </AppBadge>
    </ChatFact>
    <ChatFact v-if="session.connections.length" label="Connections">
      <TerminalConnectionChips :terminal-id="session.id" :project-id="session.projectId" :connections="session.connections" />
    </ChatFact>
    <ChatFact v-if="context" label="Context" class="ml-auto">
      <span
        class="font-mono text-micro text-text-subtle tabular-nums @min-[1060px]/chat:text-small @min-[1060px]/chat:text-text"
        :title="describeContext(context)"
        data-testid="chat-readout-context"
      >
        <span class="@min-[1060px]/chat:hidden">context </span>{{ contextPercent(context) }}%
      </span>
      <small class="hidden basis-full text-caption text-text-subtle @min-[1060px]/chat:block" data-testid="chat-context-tokens">
        {{ contextTokens(context) }}
      </small>
    </ChatFact>
    <ChatFact v-if="cost > 0" label="Cost" :class="!context && 'ml-auto'">
      <span
        class="font-mono text-micro text-text-subtle tabular-nums @min-[1060px]/chat:text-small @min-[1060px]/chat:text-text"
        data-testid="chat-readout-cost"
      >
        {{ formatCost(cost) }}
      </span>
      <small class="hidden basis-full text-caption text-text-subtle @min-[1060px]/chat:block">this session so far</small>
    </ChatFact>
  </dl>
</template>
