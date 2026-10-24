<script setup lang="ts">
import { computed } from 'vue'
import AppTextarea from '@/components/ui/AppTextarea.vue'
import type { Direction } from '@/features/graph/lib/graph-draft'
import type { ProjectConnection } from '@/features/connection/model/connection'
import type { DraftRoute, DraftStep } from '@/features/graph/model/deployment-graph'
import type { ConnectionId } from '@/model/branded'
import GraphConnectionPicker from './GraphConnectionPicker.vue'
import GraphRouteList from './GraphRouteList.vue'

/**
 * Everything one step of a graph holds, in the order a person sets it up: what it is called, the
 * system it reaches, what the session does there, where each outcome leads and how often the
 * flow may come back. Moving it first makes it where the flow begins.
 */
const props = defineProps<{
  step: DraftStep
  position: number
  steps: readonly DraftStep[]
  bindings: readonly ProjectConnection[]
  focusRoute: number | null
}>()

const emit = defineEmits<{
  update: [patch: Partial<Omit<DraftStep, 'key'>>]
  'add-route': []
  'update-route': [index: number, patch: Partial<DraftRoute>]
  'remove-route': [index: number]
  move: [direction: Direction]
  remove: []
}>()

const isFirst = computed(() => props.position === 0)
const isLast = computed(() => props.position === props.steps.length - 1)

function setBudget(raw: string): void {
  const visits = Number.parseInt(raw, 10)
  emit('update', { maxVisits: Number.isNaN(visits) ? null : Math.min(99, Math.max(1, visits)) })
}

function setConnection(connectionId: ConnectionId | null): void {
  emit('update', { connectionId })
}
</script>

<template>
  <div class="flex flex-col divide-y divide-border" data-testid="graph-step-inspector">
    <div class="flex flex-col gap-2 px-4 pb-4 pt-3">
      <div class="flex items-center justify-between gap-2">
        <p class="eyebrow">
          Step {{ position + 1 }}
          <span v-if="isFirst" class="ml-1 normal-case tracking-normal text-accent">· where the flow begins</span>
        </p>
        <div class="flex items-center gap-0.5">
          <button
            type="button"
            class="key-quiet focus-ring grid size-7 cursor-pointer place-items-center rounded-key disabled:cursor-not-allowed disabled:opacity-40"
            :disabled="isFirst"
            aria-label="Move the step earlier"
            title="Move earlier"
            data-testid="graph-step-up"
            @click="emit('move', 'up')"
          >
            <svg class="size-3.5" viewBox="0 0 14 14" fill="none" aria-hidden="true">
              <path d="M7 11V3M3.5 6.5L7 3l3.5 3.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
          </button>
          <button
            type="button"
            class="key-quiet focus-ring grid size-7 cursor-pointer place-items-center rounded-key disabled:cursor-not-allowed disabled:opacity-40"
            :disabled="isLast"
            aria-label="Move the step later"
            title="Move later"
            data-testid="graph-step-down"
            @click="emit('move', 'down')"
          >
            <svg class="size-3.5" viewBox="0 0 14 14" fill="none" aria-hidden="true">
              <path d="M7 3v8M3.5 7.5L7 11l3.5-3.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
          </button>
          <button
            type="button"
            class="focus-ring grid size-7 cursor-pointer place-items-center rounded-key text-text-subtle transition-colors hover:bg-danger-soft hover:text-danger disabled:cursor-not-allowed disabled:opacity-40"
            :disabled="steps.length === 1"
            :aria-label="`Delete ${step.title || 'the step'}`"
            title="Delete step"
            data-testid="graph-step-delete"
            @click="emit('remove')"
          >
            <svg class="size-3.5" viewBox="0 0 14 14" fill="none" aria-hidden="true">
              <path d="M2.5 4h9M5.5 4V2.5h3V4M4 4l.6 7.5h4.8L10 4" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
          </button>
        </div>
      </div>
      <label class="sr-only" :for="`graph-step-title-${step.key}`">Step title</label>
      <input
        :id="`graph-step-title-${step.key}`"
        :value="step.title"
        maxlength="200"
        autocomplete="off"
        placeholder="What this step delivers"
        class="field h-(--spacing-control) w-full rounded-control px-3 text-title font-medium text-text"
        data-testid="graph-step-title"
        @input="emit('update', { title: ($event.target as HTMLInputElement).value })"
      />
    </div>

    <section class="flex flex-col gap-2 px-4 py-3" aria-labelledby="graph-step-reach">
      <h3 id="graph-step-reach" class="eyebrow">Reaches</h3>
      <GraphConnectionPicker
        :model-value="step.connectionId"
        :bindings="bindings"
        :name="step.title || 'this step'"
        @update:model-value="setConnection"
      />
    </section>

    <section class="flex flex-col gap-2 px-4 py-3" aria-labelledby="graph-step-detail">
      <h3 id="graph-step-detail" class="eyebrow">What the session does</h3>
      <AppTextarea
        :id="`graph-step-detail-${step.key}`"
        :model-value="step.detail"
        :rows="7"
        placeholder="The instructions, and the outcome to claim when it is done."
        @update:model-value="emit('update', { detail: $event })"
      />
    </section>

    <section class="flex flex-col gap-2 px-4 py-3" aria-labelledby="graph-step-routes">
      <div>
        <h3 id="graph-step-routes" class="eyebrow">When it ends</h3>
        <p class="mt-1 text-caption leading-relaxed text-text-subtle">
          The session claims the step with one of these outcomes, and the flow follows its route.
        </p>
      </div>
      <GraphRouteList
        :step="step"
        :steps="steps"
        :focus-index="focusRoute"
        @add="emit('add-route')"
        @update="(index, patch) => emit('update-route', index, patch)"
        @remove="emit('remove-route', $event)"
      />
    </section>

    <section class="flex items-center gap-3 px-4 py-3" aria-labelledby="graph-step-budget">
      <div class="min-w-0 flex-1">
        <h3 id="graph-step-budget" class="eyebrow">Visit budget</h3>
        <p class="mt-1 text-caption leading-relaxed text-text-subtle">
          How often the flow may enter it. A loop that comes back more often stops and reports.
        </p>
      </div>
      <label class="sr-only" :for="`graph-step-budget-${step.key}`">Visit budget</label>
      <input
        :id="`graph-step-budget-${step.key}`"
        :value="step.maxVisits ?? ''"
        type="number"
        min="1"
        max="99"
        inputmode="numeric"
        placeholder="∞"
        class="field h-8 w-16 rounded-control px-2 text-center font-mono text-small text-text"
        data-testid="graph-step-budget"
        @change="setBudget(($event.target as HTMLInputElement).value)"
      />
    </section>
  </div>
</template>
