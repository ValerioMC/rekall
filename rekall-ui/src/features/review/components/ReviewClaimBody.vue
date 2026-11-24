<script setup lang="ts">
import { computed, toRef } from 'vue'
import { RouterLink } from 'vue-router'
import AppMarkdownEditor from '@/components/ui/AppMarkdownEditor.vue'
import { useTaskAdherence } from '@/features/adherence/composables/useTaskAdherence'
import ReviewCheckList from '@/features/review/components/ReviewCheckList.vue'
import ReviewCommitList from '@/features/review/components/ReviewCommitList.vue'
import StepPassHistory from '@/features/steps/components/StepPassHistory.vue'
import { adherenceLine } from '@/features/review/lib/review-adherence-line'
import { useConsoleStore } from '@/stores/console.store'
import type { Task } from '@/features/catalog/model/catalog'
import type { ReviewClaim } from '@/features/review/composables/useReviewClaim'

/**
 * What an open claim shows to be decided from: what the step asked for (or the wrapup of a task
 * handed in whole), where the work left the plan, the checks and commits found, and the task's adherence.
 */
const props = defineProps<{ task: Task; claim: ReviewClaim }>()

const store = useConsoleStore()
const taskSteps = computed(() => store.steps.filter((step) => step.taskId === props.task.id))
const adherence = useTaskAdherence(toRef(() => props.task.id), taskSteps)
const brief = computed(() => props.claim.step.value?.bodyMarkdown?.trim() ?? props.claim.wrapup.value?.bodyMarkdown.trim() ?? '')
</script>

<template>
  <div class="mt-3.5 border-t border-border pt-3.5" data-testid="review-claim-body">
    <div class="grid grid-cols-1 gap-x-10 gap-y-4 min-[1180px]:grid-cols-[minmax(0,1.1fr)_minmax(0,1fr)]">
      <div class="flex min-w-0 flex-col gap-4">
        <StepPassHistory v-if="claim.step.value?.passes.length" :passes="claim.step.value.passes" />
        <section aria-label="Brief">
          <h3 class="mb-1.5 text-caption font-medium text-text-subtle">{{ claim.step.value ? 'The step asked for' : 'The wrapup says' }}</h3>
          <div v-if="brief" class="max-h-72 min-w-0 overflow-auto" data-testid="review-brief">
            <AppMarkdownEditor :model-value="brief" readonly compact :read-settings="false" />
          </div>
          <p v-else class="text-small text-text-subtle" data-testid="review-brief-empty">
            {{ claim.step.value ? 'Nothing written under this step.' : 'This task has no wrapup yet.' }}
          </p>
        </section>
        <section v-if="claim.deviation.value" aria-label="Deviation" data-testid="review-deviation">
          <h3 class="mb-1.5 text-caption font-medium text-warn">Deviation</h3>
          <div class="max-h-60 min-w-0 overflow-auto">
            <AppMarkdownEditor :model-value="claim.deviation.value.bodyMarkdown" readonly compact :read-settings="false" />
          </div>
        </section>
      </div>
      <div class="flex min-w-0 flex-col gap-4">
        <section v-if="claim.runs.value.length" aria-label="Checks">
          <h3 class="mb-1 text-caption font-medium text-text-subtle">Checks</h3>
          <ReviewCheckList :runs="claim.runs.value" />
        </section>
        <section v-if="claim.commits.value.length" aria-label="Commits">
          <h3 class="mb-1 text-caption font-medium text-text-subtle">Commits</h3>
          <ReviewCommitList :references="claim.commits.value" />
        </section>
      </div>
    </div>
    <div class="mt-4 flex flex-wrap items-center gap-x-4 border-t border-border pt-2.5 text-small text-text-subtle">
      <span v-if="adherence" data-testid="review-adherence">
        Plan adherence of <span class="font-medium text-text-muted">{{ task.code }}</span>: {{ adherenceLine(adherence) }}
      </span>
      <RouterLink :to="{ name: 'task', params: { id: task.id, tab: 'plan' } }" class="focus-ring ml-auto rounded-key text-anchor hover:underline" data-testid="review-task-link">
        Open the task
      </RouterLink>
    </div>
  </div>
</template>
