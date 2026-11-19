<script setup lang="ts">
import { computed, watch } from 'vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { ADHERENCE_HELP, shareDetail } from '@/features/adherence/lib/adherence-figures'
import { useAdherenceStore } from '@/features/adherence/store/adherence.store'
import type { ProjectId, TaskId } from '@/model/branded'

/**
 * One line of the adherence numbers of every task in a project, and the task with the most steps
 * sent back as a link to it. Read when the project opens and again when its task list changes.
 */
const props = defineProps<{ projectId: ProjectId; taskCount: number }>()
const emit = defineEmits<{ open: [taskId: TaskId] }>()

const store = useAdherenceStore()
const { run } = useAsyncAction()

const adherence = computed(() => store.ofProject(props.projectId))
const mostSentBack = computed(() => adherence.value?.mostSentBack[0] ?? null)

watch(
  () => [props.projectId, props.taskCount],
  () => void run(() => store.loadProject(props.projectId)),
  { immediate: true }
)
</script>

<template>
  <section
    class="flex min-h-10 flex-wrap items-center gap-x-5 gap-y-1 rounded-card border border-border bg-surface px-3.5 py-1.5 text-small text-text-muted"
    aria-label="Plan adherence of the project"
    data-testid="project-adherence"
  >
    <span class="font-medium text-text" :title="ADHERENCE_HELP">Plan adherence</span>

    <span v-if="adherence && !adherence.stepsTotal" data-testid="project-adherence-empty">
      No task of this project has steps yet.
    </span>

    <template v-else-if="adherence">
      <span>
        Accepted first time
        <b class="ml-1 font-mono text-body font-normal text-text" data-testid="project-adherence-first-time">{{ `${adherence.acceptedFirstTime} of ${adherence.stepsDone}` }}</b>
        <span class="ml-1.5 text-text-subtle">{{ shareDetail(adherence.firstTimeShare) }}</span>
      </span>
      <span>
        Sent back
        <b class="ml-1 font-mono text-body font-normal text-text" data-testid="project-adherence-sent-back">{{ adherence.sentBack }}</b>
        <span class="ml-1.5 text-text-subtle">{{ adherence.passesTotal }} {{ adherence.passesTotal === 1 ? 'pass' : 'passes' }} in all</span>
      </span>
      <span>
        Deviations
        <b class="ml-1 font-mono text-body font-normal text-text" data-testid="project-adherence-deviations">{{ adherence.deviations }}</b>
      </span>

      <button
        v-if="mostSentBack"
        type="button"
        class="focus-ring ml-auto flex min-w-0 cursor-pointer items-baseline gap-2 rounded-key px-1.5 py-0.5 hover:bg-surface-raised"
        data-testid="project-adherence-task"
        @click="emit('open', mostSentBack.taskId)"
      >
        <span class="text-text-subtle">Most sent back</span>
        <span class="min-w-0 max-w-64 truncate text-text">{{ mostSentBack.title }}</span>
        <span class="font-mono text-caption text-text-muted">{{ mostSentBack.sentBack }}</span>
      </button>
    </template>
  </section>
</template>
