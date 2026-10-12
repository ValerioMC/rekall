import { computed, type Ref } from 'vue'
import { storeToRefs } from 'pinia'
import { useConsoleStore } from '@/stores/console.store'
import { stepSkills, taskSkills } from '@/model/skill-assignment'
import type { Task, TaskStep } from '@/model/catalog'

/** What a step is worked with: its own skills, then the task's and the tag's. */
export function useStepSkills(step: Ref<TaskStep>, task: Ref<Task>) {
  const { taskSkills: owned, tagSkills: tagged } = storeToRefs(useConsoleStore())

  const taskLevel = computed(() => taskSkills(task.value.id, task.value.tagId, owned.value, tagged.value))
  const effective = computed(() => stepSkills(step.value.id, taskLevel.value, owned.value))
  const own = computed(() =>
    owned.value.filter((skill) => skill.stepId === step.value.id).map((skill) => skill.skillName)
  )

  return { taskLevel, effective, own }
}
