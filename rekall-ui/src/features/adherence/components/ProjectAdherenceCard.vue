<script setup lang="ts">
import { computed, watch } from 'vue'
import AdherenceFigure from '@/features/adherence/components/AdherenceFigure.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { ADHERENCE_HELP, shareDetail } from '@/features/adherence/lib/adherence-figures'
import { useAdherenceStore } from '@/features/adherence/store/adherence.store'
import type { ProjectId, TaskId } from '@/model/branded'

/**
 * The adherence numbers of every task in a project, and the tasks with the most steps sent back,
 * each a link to the task. Read when the project opens and again when its task list changes.
 */
const props = defineProps<{ projectId: ProjectId; taskCount: number }>()
const emit = defineEmits<{ open: [taskId: TaskId] }>()

const store = useAdherenceStore()
const { run } = useAsyncAction()

const adherence = computed(() => store.ofProject(props.projectId))

watch(
  () => [props.projectId, props.taskCount],
  () => void run(() => store.loadProject(props.projectId)),
  { immediate: true }
)
</script>

<template>
  <section class="rounded-card border border-border bg-surface px-4 py-3" aria-label="Plan adherence of the project" data-testid="project-adherence">
    <div class="flex flex-wrap items-baseline gap-x-3">
      <span class="eyebrow">Plan adherence</span>
      <span class="text-caption text-text-subtle">{{ ADHERENCE_HELP }}</span>
    </div>

    <p v-if="adherence && !adherence.stepsTotal" class="mt-2 text-small text-text-muted" data-testid="project-adherence-empty">
      No task of this project has steps yet.
    </p>

    <template v-else-if="adherence">
      <div class="mt-2 grid grid-cols-1 gap-4 sm:grid-cols-3">
        <AdherenceFigure
          label="Accepted first time"
          :value="`${adherence.acceptedFirstTime} of ${adherence.stepsDone}`"
          :detail="shareDetail(adherence.firstTimeShare)"
          testid="project-adherence-first-time"
        />
        <AdherenceFigure
          label="Sent back"
          :value="String(adherence.sentBack)"
          :detail="`${adherence.passesTotal} ${adherence.passesTotal === 1 ? 'pass' : 'passes'} in all`"
          testid="project-adherence-sent-back"
        />
        <AdherenceFigure
          label="Deviation notes"
          :value="String(adherence.deviationNotes)"
          detail="across the project's tasks"
          testid="project-adherence-deviations"
        />
      </div>

      <div v-if="adherence.mostSentBack.length" class="mt-3">
        <span class="eyebrow">Most steps sent back</span>
        <ul class="mt-1 flex flex-col gap-0.5">
          <li v-for="task in adherence.mostSentBack" :key="task.taskId">
            <button
              type="button"
              class="focus-ring flex w-full items-baseline gap-2 rounded-key px-1 py-0.5 text-left text-small hover:bg-surface-raised"
              data-testid="project-adherence-task"
              @click="emit('open', task.taskId)"
            >
              <span class="min-w-0 flex-1 truncate text-text">{{ task.title }}</span>
              <span class="shrink-0 font-mono text-caption text-text-muted">{{ task.sentBack }} sent back</span>
              <span class="shrink-0 font-mono text-caption text-text-subtle">{{ task.passesTotal }} passes</span>
            </button>
          </li>
        </ul>
      </div>
    </template>
  </section>
</template>
