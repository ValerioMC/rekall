<script setup lang="ts">
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import { useRouter } from 'vue-router'
import SubNav, { type SubNavItem } from '@/components/ui/SubNav.vue'
import ProjectAdherenceCard from '@/features/adherence/components/ProjectAdherenceCard.vue'
import TaskComposer from '@/features/catalog/components/TaskComposer.vue'
import { useTaskEditor } from '@/features/catalog/composables/useTaskEditor'
import ProjectTaskLedger from '@/features/project/components/ProjectTaskLedger.vue'
import ProjectTaskTools from '@/features/project/components/ProjectTaskTools.vue'
import { useComposerTarget } from '@/features/project/composables/useComposerTarget'
import { useProjectTasksStore } from '@/features/project/store/projectTasks.store'
import { PROJECT_TASK_VIEWS, PROJECT_TASK_VIEW_LABEL, type ProjectTaskView } from '@/features/project/model/project-task-list'
import { useConsoleStore } from '@/stores/console.store'
import type { TaskId } from '@/model/branded'

/** The tasks of the project in view, with the one place to start work above them and its plan adherence. */
const store = useConsoleStore()
const projectTasks = useProjectTasksStore()
const router = useRouter()
const editor = useTaskEditor()
const { isLoading } = storeToRefs(store)
const { view, counts, rows, projectInView } = storeToRefs(projectTasks)
const { target, choices, choose } = useComposerTarget()

const tabs = computed<SubNavItem<ProjectTaskView>[]>(() =>
  PROJECT_TASK_VIEWS.map((id) => ({ id, label: PROJECT_TASK_VIEW_LABEL[id], count: counts.value[id], hot: id === 'needs' }))
)

function planTask(title: string): void {
  if (target.value) editor.beginCreate({ projectId: target.value.id, title })
}

function openTask(taskId: TaskId): void {
  void router.push(`/task/${taskId}`)
}
</script>

<template>
  <div class="flex min-h-0 flex-1 flex-col" data-testid="tasks-section">
    <TaskComposer v-if="target" class="mt-4" :project="target" :choices="choices" @choose="choose" @plan="planTask" />
    <ProjectAdherenceCard v-if="projectInView" class="mt-3" :project-id="projectInView" :task-count="rows.length" @open="openTask" />

    <SubNav v-model="view" class="mt-3" :items="tabs" label="Task views">
      <template #trailing><ProjectTaskTools /></template>
    </SubNav>

    <div v-if="isLoading" class="flex flex-col gap-2 pt-6" aria-hidden="true">
      <div v-for="placeholder in 6" :key="placeholder" class="skeleton h-11 w-full rounded-control" />
    </div>
    <ProjectTaskLedger v-else />
  </div>
</template>
