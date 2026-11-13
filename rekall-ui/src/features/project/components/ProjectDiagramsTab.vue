<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import AppButton from '@/components/ui/AppButton.vue'
import AppEmptyState from '@/components/ui/AppEmptyState.vue'
import DiagramCard from '@/features/diagram/components/gallery/DiagramCard.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useDiagramStore } from '@/features/diagram/store/diagram.store'
import { useConsoleStore } from '@/stores/console.store'
import type { Project } from '@/features/catalog/model/catalog'
import type { DiagramId } from '@/model/branded'

/** The diagrams drawn for this project; opening one gives it the whole Diagrams screen. */
const props = defineProps<{ project: Project }>()

const diagrams = useDiagramStore()
const store = useConsoleStore()
const router = useRouter()
const { run } = useAsyncAction()

onMounted(() => {
  if (!diagrams.loaded) void run(() => diagrams.load())
})

const ofProject = computed(() => diagrams.summaries.filter((summary) => summary.projectId === props.project.id))

function taskAnchorOf(taskId: string | null): string | null {
  const task = taskId ? store.tasks.find((candidate) => candidate.id === taskId) : null
  return task ? `task:${task.code}` : null
}

function open(id: DiagramId): void {
  void router.push({ path: '/diagrams', query: { diagram: id } })
}
</script>

<template>
  <div data-testid="project-diagrams">
    <div v-if="ofProject.length" class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
      <DiagramCard
        v-for="summary in ofProject"
        :key="summary.id"
        :summary="summary"
        :task-anchor="taskAnchorOf(summary.taskId)"
        :arrived="false"
        @open="open(summary.id)"
      />
    </div>
    <AppEmptyState v-else title="No diagram yet" description="A session draws one from the code.">
      <RouterLink to="/diagrams"><AppButton variant="secondary" size="sm">Generate one</AppButton></RouterLink>
    </AppEmptyState>
  </div>
</template>
