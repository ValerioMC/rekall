<script setup lang="ts">
import { storeToRefs } from 'pinia'
import FormPanel from '@/components/ui/FormPanel.vue'
import FormRow from '@/components/ui/FormRow.vue'
import SegmentedControl, { type SegmentOption } from '@/components/ui/SegmentedControl.vue'
import { usePreferencesStore } from '@/features/settings/store/preferences.store'
import { useProjectTasksStore } from '@/features/project/store/projectTasks.store'
import { SCALES, TASK_WIDTHS, type Motion, type TaskWidth } from '@/common/config/ui-preferences'
import { TASK_WIDTH_LABEL } from '@/features/task/model/task-width'
import type { ProjectTaskDensity } from '@/features/project/model/project-task-list'

const preferences = usePreferencesStore()
const projectTasks = useProjectTasksStore()
const { motion, scale, taskWidth } = storeToRefs(preferences)
const { density } = storeToRefs(projectTasks)

const DENSITY: readonly SegmentOption<ProjectTaskDensity>[] = [
  { value: 'comfortable', label: 'Comfortable' },
  { value: 'compact', label: 'Compact' }
]
const MOTION: readonly SegmentOption<Motion>[] = [
  { value: 'full', label: 'Full' },
  { value: 'reduced', label: 'Reduced' }
]
const WIDTH_OPTIONS: readonly SegmentOption<TaskWidth>[] = TASK_WIDTHS.map((value) => ({ value, label: TASK_WIDTH_LABEL[value] }))
const SCALE_OPTIONS: readonly SegmentOption<string>[] = SCALES.map((value) => ({ value: String(value), label: `${value}%` }))

function chooseScale(value: string): void {
  scale.value = SCALES.find((candidate) => String(candidate) === value) ?? scale.value
}
</script>

<template>
  <FormPanel title="Appearance" aria-label="Appearance" data-testid="appearance-section">
    <FormRow label="Density" hint="How tall a row of a project's task list is.">
      <SegmentedControl v-model="density" :options="DENSITY" label="Density" />
    </FormRow>
    <FormRow label="Motion" hint="Reduced stops the loops, the running comet, the tube sheen and the session pulse. Every state change still shows.">
      <SegmentedControl v-model="motion" :options="MOTION" label="Motion" />
    </FormRow>
    <FormRow label="Text scale" hint="Scales every size in the console.">
      <SegmentedControl :model-value="String(scale)" :options="SCALE_OPTIONS" label="Text scale" @update:model-value="chooseScale" />
    </FormRow>
    <FormRow label="Page width" hint="How wide a document reads. One choice for every document.">
      <SegmentedControl v-model="taskWidth" :options="WIDTH_OPTIONS" label="Page width" />
    </FormRow>
  </FormPanel>
</template>
