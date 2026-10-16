<script setup lang="ts">
import { toRef } from 'vue'
import SkillField from '@/features/skills/components/SkillField.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useStepSkills } from '@/features/skills/composables/useStepSkills'
import { useConsoleStore } from '@/stores/console.store'
import type { Task, TaskStep } from '@/features/catalog/model/catalog'

/**
 * The skills for one step, opened with the step. The task's and the tag's skills are listed
 * first and dashed, since every step already runs with them; the step's own are added here.
 */
const props = defineProps<{ step: TaskStep; task: Task }>()

const store = useConsoleStore()
const { run } = useAsyncAction()
const { taskLevel, own } = useStepSkills(toRef(props, 'step'), toRef(props, 'task'))

function save(names: readonly string[]): void {
  void run(() => store.assignStepSkills(props.step.id, names))
}
</script>

<template>
  <div class="mb-3" data-testid="step-skills">
    <span class="eyebrow mb-1.5 block">Skills</span>
    <SkillField
      :model-value="own"
      :inherited="taskLevel"
      owner="step"
      own-origin="step"
      @update:model-value="save"
    />
  </div>
</template>
