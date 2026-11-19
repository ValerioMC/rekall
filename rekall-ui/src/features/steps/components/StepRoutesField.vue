<script setup lang="ts">
import { routeDestination } from '@/features/flow/lib/step-flow'
import type { TaskStep } from '@/features/steps/model/task-step'

/** The routes a draft from a proposed flow leaves by, named against the task's steps. They are edited in the Flow view. */
defineProps<{ step: TaskStep; taskSteps: readonly TaskStep[] }>()
</script>

<template>
  <div v-if="step.links.length" data-testid="step-routes">
    <span class="eyebrow mb-1.5 block">Routes</span>
    <ul class="grid grid-cols-[repeat(auto-fit,minmax(22rem,1fr))] gap-x-10">
      <li v-for="link in step.links" :key="link.id" class="flex min-h-7 items-baseline gap-2 text-small text-text-muted">
        <span class="min-w-[3.2ch] font-mono text-text">{{ link.outcome }}</span>
        <span class="text-text-subtle" aria-hidden="true">→</span>
        <span class="min-w-0 truncate">{{ routeDestination(link, taskSteps) }}</span>
      </li>
    </ul>
    <p class="mt-2 text-caption text-text-subtle">From the proposed flow. Edit them in the Flow view.</p>
  </div>
</template>
