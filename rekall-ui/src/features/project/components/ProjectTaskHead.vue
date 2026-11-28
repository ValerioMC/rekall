<script setup lang="ts">
import { storeToRefs } from 'pinia'
import ProjectTaskSortKey from '@/features/project/components/ProjectTaskSortKey.vue'
import { useProjectTasksStore } from '@/features/project/store/projectTasks.store'

/** The column heads of the task list; a head with something to order by sorts the list. */
const projectTasks = useProjectTasksStore()
const { columns, sort } = storeToRefs(projectTasks)
</script>

<template>
  <div
    class="ledger-columns sticky top-0 z-(--z-sticky) h-9 border-b border-border bg-canvas text-caption text-text-subtle"
    role="row"
  >
    <div
      v-for="column in columns"
      :key="column.id"
      class="flex min-w-0"
      :class="column.alignEnd && 'justify-end'"
      role="columnheader"
      :aria-sort="column.sortKey === sort.key ? (sort.descending ? 'descending' : 'ascending') : undefined"
      :data-testid="`task-head-${column.id}`"
    >
      <ProjectTaskSortKey
        v-if="column.sortKey"
        :label="column.label"
        :active="column.sortKey === sort.key"
        :descending="sort.descending"
        :align-end="column.alignEnd"
        @sort="projectTasks.sortBy(column.sortKey)"
      />
      <span v-else class="flex h-7 items-center truncate">{{ column.label }}</span>
    </div>
  </div>
</template>
