<script setup lang="ts">
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import { useRouter } from 'vue-router'
import SubNav, { type SubNavItem } from '@/components/ui/SubNav.vue'
import ProjectAdherenceCard from '@/features/adherence/components/ProjectAdherenceCard.vue'
import TaskComposer from '@/features/catalog/components/TaskComposer.vue'
import { useTaskEditor } from '@/features/catalog/composables/useTaskEditor'
import WorkBoard from '@/features/work/components/WorkBoard.vue'
import WorkLedger from '@/features/work/components/WorkLedger.vue'
import WorkTools from '@/features/work/components/WorkTools.vue'
import { useComposerTarget } from '@/features/work/composables/useComposerTarget'
import { useWorkLayout } from '@/features/work/composables/useWorkLayout'
import { useWorkStore } from '@/features/work/store/work.store'
import { WORK_VIEWS, WORK_VIEW_LABEL, type WorkView } from '@/features/work/model/work-view'
import { useConsoleStore } from '@/stores/console.store'
import type { TaskId } from '@/model/branded'

/**
 * The tasks of whatever the scope shows, with the one place to start work above them. Work and a
 * project's Tasks page are this same section: the project in view adds its plan adherence and
 * drops the project headers. Only Work picks the layout, because the layout is a route.
 */
defineProps<{ choosesLayout: boolean }>()

const store = useConsoleStore()
const work = useWorkStore()
const router = useRouter()
const editor = useTaskEditor()
const layout = useWorkLayout()
const { isLoading } = storeToRefs(store)
const { view, counts, rows, projectInView } = storeToRefs(work)
const { target, choices, choose } = useComposerTarget()

const isBoard = computed(() => layout.value === 'board')
const tabs = computed<SubNavItem<WorkView>[]>(() =>
  WORK_VIEWS.map((id) => ({ id, label: WORK_VIEW_LABEL[id], count: counts.value[id], hot: id === 'needs' }))
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

    <SubNav v-if="!isBoard" v-model="view" class="mt-3" :items="tabs" label="Work views">
      <template #trailing><WorkTools :chooses-layout="choosesLayout" /></template>
    </SubNav>
    <div v-else class="flex items-center justify-end gap-2 pt-3"><WorkTools :chooses-layout="choosesLayout" /></div>

    <WorkBoard v-if="isBoard && !isLoading" />
    <div v-else-if="isLoading" class="flex flex-col gap-2 pt-6" aria-hidden="true">
      <div v-for="placeholder in 6" :key="placeholder" class="skeleton h-11 w-full rounded-control" />
    </div>
    <WorkLedger v-else />
  </div>
</template>
