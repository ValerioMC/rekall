<script setup lang="ts">
import { useId } from 'vue'
import AppCollapse from '@/components/ui/AppCollapse.vue'
import NavigatorEpicRow from '@/features/catalog/components/epic/NavigatorEpicRow.vue'
import NavigatorTaskRow from '@/components/console/NavigatorTaskRow.vue'
import type { EpicBranch } from '@/features/catalog/lib/partition-tasks'
import type { Task } from '@/features/catalog/model/catalog'
import type { TaskId } from '@/model/branded'

/**
 * An epic and its tasks in the navigator. The epic is closed until opened; its tasks hang under a
 * hairline rail in the epic tone, and are listed here and in no other part of the list.
 */
withDefaults(
  defineProps<{
    branch: EpicBranch
    expanded: boolean
    selectedTaskId: TaskId | null
    runningTaskIds: ReadonlySet<TaskId>
    showContext?: boolean
    showCompany?: boolean
    filed?: boolean
  }>(),
  { showContext: false, showCompany: false, filed: false }
)

defineEmits<{ select: [id: TaskId]; edit: [task: Task]; toggle: [] }>()

const bodyId = useId()
</script>

<template>
  <div :class="filed && 'filed-row'" data-testid="epic-branch">
    <NavigatorEpicRow
      :epic="branch.epic"
      :tasks="branch.children"
      :selected="branch.epic.id === selectedTaskId"
      :expanded="expanded"
      :controls="bodyId"
      @select="$emit('select', branch.epic.id)"
      @toggle="$emit('toggle')"
    />
    <AppCollapse :open="expanded">
      <div
        :id="bodyId"
        role="group"
        :aria-label="`Tasks of ${branch.epic.title}`"
        class="epic-rail ml-[15px] mt-0.5 flex flex-col gap-0.5 border-l border-epic-line pl-2"
      >
        <NavigatorTaskRow
          v-for="task in branch.children"
          :key="task.id"
          :task="task"
          :selected="task.id === selectedTaskId"
          :running="runningTaskIds.has(task.id)"
          :show-context="showContext"
          :show-company="showCompany"
          @select="$emit('select', task.id)"
          @edit="$emit('edit', task)"
        />
      </div>
    </AppCollapse>
  </div>
</template>
