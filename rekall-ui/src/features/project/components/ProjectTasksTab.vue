<script setup lang="ts">
import { computed, onBeforeUnmount, watch } from 'vue'
import { storeToRefs } from 'pinia'
import SubNav from '@/components/ui/SubNav.vue'
import type { SubNavItem } from '@/components/ui/SubNav.vue'
import ProjectAdherenceCard from '@/features/adherence/components/ProjectAdherenceCard.vue'
import QuickTaskBar from '@/features/catalog/components/QuickTaskBar.vue'
import WorkLedger from '@/features/work/components/WorkLedger.vue'
import { useWorkStore } from '@/features/work/store/work.store'
import { WORK_VIEW_LABEL } from '@/features/work/model/work-view'
import type { WorkView } from '@/features/work/model/work-view'
import type { Project } from '@/features/catalog/model/catalog'
import type { TaskId } from '@/model/branded'

/** The project's tasks as the Work ledger, narrowed to this project; the Running view is Work's own. */
const props = defineProps<{ project: Project }>()
const emit = defineEmits<{ openTask: [taskId: TaskId] }>()

const VIEWS: readonly WorkView[] = ['needs', 'open', 'filed']

const work = useWorkStore()
const { view, counts, rows } = storeToRefs(work)

watch(
  () => props.project.id,
  (id) => work.focusProject(id),
  { immediate: true }
)
watch(
  view,
  (current) => {
    if (!VIEWS.includes(current)) view.value = 'open'
  },
  { immediate: true }
)
onBeforeUnmount(() => work.focusProject(null))

const tabs = computed<SubNavItem<WorkView>[]>(() =>
  VIEWS.map((id) => ({ id, label: WORK_VIEW_LABEL[id], count: counts.value[id], hot: id === 'needs' }))
)
</script>

<template>
  <div class="flex flex-col gap-4 pb-8" data-testid="project-tasks">
    <QuickTaskBar :project="project" />
    <ProjectAdherenceCard :project-id="project.id" :task-count="rows.length" @open="emit('openTask', $event)" />
    <SubNav v-model="view" :items="tabs" label="Project tasks" />
    <WorkLedger />
  </div>
</template>
