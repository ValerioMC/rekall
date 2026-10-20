<script setup lang="ts">
import { computed } from 'vue'
import StepSeal from '@/features/steps/components/StepSeal.vue'
import { relativeTime } from '@/common/format/relative-time'
import { routesInto } from '@/features/flow/lib/step-flow'
import type { TaskStep } from '@/features/catalog/model/catalog'
import type { RouteTarget } from '@/features/flow/lib/route-target'
import type { StepLinkId, TaskStepId } from '@/model/branded'
import FlowRouteEditor from './FlowRouteEditor.vue'
import FlowVisitBudget from './FlowVisitBudget.vue'

/**
 * The step picked on the canvas: where it stands, every time the work entered it and how it
 * left, the routes that lead into it, the editor for the routes out of it and its visit budget.
 */
const props = defineProps<{
  step: TaskStep
  steps: readonly TaskStep[]
  current: boolean
  busy: boolean
  drawn?: { to: RouteTarget; nonce: number } | null
}>()

const emit = defineEmits<{
  select: [id: TaskStepId]
  'open-detail': []
  'add-route': [to: RouteTarget, outcome: string]
  'edit-route': [id: StepLinkId, patch: { toStepId?: TaskStepId; endsFlow?: boolean; outcome?: string }]
  'remove-route': [id: StepLinkId]
  'limit-visits': [maxVisits: number | null]
}>()

const incoming = computed(() => routesInto(props.step, props.steps))
const visitsNewestFirst = computed(() => [...props.step.visits].reverse())
const stateLabel = computed(() => (props.current ? 'the work stands here' : props.step.state.toLowerCase()))
</script>

<template>
  <aside
    class="flex min-h-0 flex-col gap-5 overflow-y-auto p-5"
    aria-label="Selected step"
    data-testid="flow-inspector"
  >
    <header class="flex items-start gap-3">
      <span class="relative mt-1 block size-[22px] shrink-0" aria-hidden="true">
        <StepSeal :state="step.state" :next="current && step.state === 'OPEN'" />
      </span>
      <div class="min-w-0 flex-1">
        <p class="eyebrow">Step {{ step.position + 1 }}</p>
        <h3 class="text-body font-semibold leading-snug text-text">{{ step.title }}</h3>
        <p class="mt-0.5 text-caption" :class="current ? 'text-accent' : 'text-text-subtle'">
          {{ stateLabel }}
        </p>
      </div>
      <button
        type="button"
        class="key-quiet focus-ring shrink-0 cursor-pointer rounded-control px-2.5 py-1 text-caption"
        data-testid="flow-open-detail"
        @click="emit('open-detail')"
      >
        Detail
      </button>
    </header>

    <FlowRouteEditor
      :step="step"
      :steps="steps"
      :busy="busy"
      :drawn="drawn"
      @add="(to, outcome) => emit('add-route', to, outcome)"
      @edit="(id, patch) => emit('edit-route', id, patch)"
      @remove="(id) => emit('remove-route', id)"
    />

    <FlowVisitBudget :step="step" :busy="busy" @limit="(max) => emit('limit-visits', max)" />

    <section aria-labelledby="flow-routes-in">
      <h4 id="flow-routes-in" class="eyebrow mb-2">Reached from</h4>
      <ul v-if="incoming.length" class="flex flex-wrap gap-1.5">
        <li v-for="{ link, source } in incoming" :key="link.id">
          <button
            type="button"
            class="focus-ring inline-flex cursor-pointer items-center gap-1.5 rounded-full border border-border px-2 py-0.5 text-caption text-text-muted transition-colors hover:border-border-strong hover:text-text"
            @click="emit('select', source.id)"
          >
            <span class="font-mono text-micro tabular-nums opacity-70">{{ source.position + 1 }}</span>
            <span class="max-w-[16ch] truncate">{{ source.title }}</span>
            <span class="font-mono text-micro text-accent">{{ link.outcome }}</span>
          </button>
        </li>
      </ul>
      <p v-else class="text-caption text-text-subtle">No route leads here.</p>
    </section>

    <section aria-labelledby="flow-visits">
      <h4 id="flow-visits" class="eyebrow mb-2">
        Visits <span class="font-mono tabular-nums text-text-subtle">{{ step.visits.length }}</span>
      </h4>
      <ol v-if="step.visits.length" class="space-y-1" data-testid="flow-visit-list">
        <li v-for="visit in visitsNewestFirst" :key="visit.id" class="flex items-center gap-2 text-caption">
          <span class="w-8 shrink-0 font-mono text-micro tabular-nums text-text-subtle"
            >#{{ visit.ordinal }}</span
          >
          <span v-if="visit.leftAt === null" class="text-accent">here now</span>
          <span v-else-if="visit.outcome" class="font-mono text-text-muted">left by {{ visit.outcome }}</span>
          <span v-else class="text-text-subtle">left without an outcome</span>
          <span class="ml-auto text-micro text-text-subtle">{{ relativeTime(visit.enteredAt) }}</span>
        </li>
      </ol>
      <p v-else class="text-caption text-text-subtle">The work has not reached this step.</p>
    </section>
  </aside>
</template>
