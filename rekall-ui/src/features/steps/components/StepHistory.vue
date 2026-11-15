<script setup lang="ts">
import { computed } from 'vue'
import { relativeTime } from '@/common/format/relative-time'
import { historyOf, type StepEventKind } from '@/features/steps/lib/step-history'
import type { TaskStep } from '@/features/steps/model/task-step'

/** What happened to the step and when, as a short ledger; every step has at least its creation. */
const props = defineProps<{ step: TaskStep }>()

const events = computed(() => historyOf(props.step))

const DOT: Readonly<Record<StepEventKind, string>> = {
  created: 'bg-border-strong',
  'sent-back': 'bg-warn',
  running: 'bg-session',
  claimed: 'bg-session',
  done: 'bg-safe'
}
</script>

<template>
  <section aria-label="History" data-testid="step-history">
    <span class="eyebrow">History</span>
    <ol class="mt-1.5 flex flex-col">
      <li
        v-for="event in events"
        :key="`${event.kind}-${event.at}`"
        class="grid grid-cols-[14px_minmax(0,1fr)_auto] items-baseline gap-2 py-1 text-small text-text-muted"
        data-testid="step-event"
      >
        <span class="size-1.5 justify-self-center rounded-full" :class="DOT[event.kind]" aria-hidden="true" />
        <span>{{ event.label }}</span>
        <span class="font-mono text-caption text-text-subtle">{{ relativeTime(event.at) }}</span>
      </li>
    </ol>
  </section>
</template>
