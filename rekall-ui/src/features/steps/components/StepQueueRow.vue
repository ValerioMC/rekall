<script setup lang="ts">
import { computed, toRef } from 'vue'
import StepPassBars from '@/features/steps/components/StepPassBars.vue'
import StepSeal from '@/features/steps/components/StepSeal.vue'
import { checksLine } from '@/features/check/lib/check-results'
import { useStepCheckRuns } from '@/features/check/composables/useStepCheckRuns'
import { queueNote } from '@/features/steps/lib/plan-overview'
import { useConsoleStore } from '@/stores/console.store'
import type { TaskStep } from '@/features/steps/model/task-step'

/** One step in the overview, with what it holds under its title; pressing it opens the step. */
const props = defineProps<{ step: TaskStep; number: number; complete: boolean }>()

defineEmits<{ select: [] }>()

const store = useConsoleStore()
const { runs } = useStepCheckRuns(toRef(props, 'step'))
const note = computed(() => queueNote(props.step, { checks: checksLine(runs.value), commits: store.stepCommitReferences(props.step.id).length }))
</script>

<template>
  <li>
    <button
      type="button"
      class="focus-ring -mx-2 grid w-[calc(100%+1rem)] cursor-pointer grid-cols-[22px_minmax(0,1fr)_auto] items-center gap-3 rounded-control px-2 py-2.5 text-left transition-colors duration-(--motion-quick) hover:bg-surface-hover"
      data-testid="queue-step"
      :data-step-state="step.state"
      @click="$emit('select')"
    >
      <span class="size-5.5" aria-hidden="true"><StepSeal :state="step.state" :complete="complete" /></span>
      <span class="min-w-0">
        <span class="block text-body text-text">{{ number }}. {{ step.title }}</span>
        <span class="mt-0.5 block truncate text-small text-text-subtle" data-testid="queue-note">{{ note }}</span>
      </span>
      <span class="flex items-center gap-3 text-text-subtle">
        <StepPassBars :step="step" />
        <svg class="size-2.5" viewBox="0 0 24 24" fill="none" aria-hidden="true">
          <path d="M9 6l6 6-6 6" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
      </span>
    </button>
  </li>
</template>
