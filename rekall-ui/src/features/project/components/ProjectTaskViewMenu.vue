<script setup lang="ts">
import { computed, ref } from 'vue'
import { storeToRefs } from 'pinia'
import AppButton from '@/components/ui/AppButton.vue'
import DropPanel from '@/components/ui/DropPanel.vue'
import SegmentedControl, { type SegmentOption } from '@/components/ui/SegmentedControl.vue'
import { OPTIONAL_COLUMNS } from '@/features/project/lib/task-columns'
import type { ProjectTaskDensity } from '@/features/project/model/project-task-list'
import { useProjectTasksStore } from '@/features/project/store/projectTasks.store'

/** What changes how the list is drawn, not what it lists: row density, the side preview and the columns. */
const DENSITY_OPTIONS: readonly SegmentOption<ProjectTaskDensity>[] = [
  { value: 'comfortable', label: 'Comfortable' },
  { value: 'compact', label: 'Compact' }
]

const projectTasks = useProjectTasksStore()
const { density, sidePreview, sidePreviewFits, hiddenColumns } = storeToRefs(projectTasks)
const isOpen = ref(false)

const columns = computed(() =>
  OPTIONAL_COLUMNS.map((column) => ({
    id: column.id,
    label: column.label,
    shown: !hiddenColumns.value.has(column.id),
    absence: hiddenColumns.value.has(column.id) ? null : projectTasks.absenceOf(column.id)
  }))
)
</script>

<template>
  <DropPanel v-model:open="isOpen" label="View options" data-testid="task-view-menu">
    <template #trigger="{ open, toggle }">
      <AppButton variant="ghost" aria-haspopup="true" :aria-expanded="open" data-testid="task-view-trigger" @click="toggle">
        <svg class="size-4" viewBox="0 0 16 16" fill="none" aria-hidden="true">
          <path d="M2.5 5h6M11.5 5h2M2.5 11h2M7.5 11h6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
          <circle cx="10" cy="5" r="1.5" stroke="currentColor" stroke-width="1.5" />
          <circle cx="6" cy="11" r="1.5" stroke="currentColor" stroke-width="1.5" />
        </svg>
        View
      </AppButton>
    </template>

    <div class="flex w-72 flex-col gap-4 p-4" data-testid="task-view-panel">
      <div class="flex flex-col gap-2">
        <p class="text-caption font-medium text-text-subtle">Row density</p>
        <SegmentedControl v-model="density" :options="DENSITY_OPTIONS" label="Density" />
      </div>
      <label class="flex cursor-pointer items-center gap-2 text-small text-text">
        <input v-model="sidePreview" type="checkbox" class="focus-ring size-3.5 accent-(--color-accent)" data-testid="task-side-preview" />
        Side preview
        <span v-if="!sidePreviewFits" class="ml-auto text-caption text-text-subtle">needs more width</span>
      </label>
      <fieldset class="flex flex-col gap-1.5">
        <legend class="mb-2 text-caption font-medium text-text-subtle">Columns</legend>
        <label v-for="column in columns" :key="column.id" class="flex h-6 cursor-pointer items-center gap-2 text-small text-text">
          <input
            type="checkbox"
            class="focus-ring size-3.5 accent-(--color-accent)"
            :checked="column.shown"
            :data-testid="`task-column-${column.id}`"
            @change="projectTasks.toggleColumn(column.id)"
          />
          {{ column.label }}
          <span v-if="column.absence" class="ml-auto text-caption text-text-subtle" data-testid="task-column-absence">{{ column.absence }}</span>
        </label>
      </fieldset>
    </div>
  </DropPanel>
</template>
