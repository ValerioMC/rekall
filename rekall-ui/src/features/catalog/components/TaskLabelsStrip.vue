<script setup lang="ts">
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import SkillField from '@/features/skills/components/SkillField.vue'
import TaskDependenciesField from '@/features/dependency/components/TaskDependenciesField.vue'
import TaskTagControl from '@/features/tags/components/TaskTagControl.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useConsoleStore } from '@/stores/console.store'
import { skillsOfTag } from '@/features/skills/model/skill-assignment'
import type { Task } from '@/features/catalog/model/catalog'

/**
 * The task's tag, skills and dependencies. The tag and the task's own skills are changed in place;
 * the tag's skills arrive dashed, since they are the tag's. `stacked` lays them out as labelled
 * fields one under the other, for the properties bar; otherwise they run along one line.
 */
const props = defineProps<{ task: Task; stacked?: boolean }>()

const store = useConsoleStore()
const { taskSkills: owned, tagSkills: tagged } = storeToRefs(store)
const { run } = useAsyncAction()

const own = computed(() =>
  owned.value
    .filter((skill) => skill.taskId === props.task.id && skill.stepId === null)
    .map((skill) => skill.skillName)
)
const fromTag = computed(() => skillsOfTag(props.task.tagId, tagged.value))

function save(names: readonly string[]): void {
  void run(() => store.assignTaskSkills(props.task.id, names))
}
</script>

<template>
  <div v-if="stacked" class="grid min-w-0 gap-3" data-testid="task-labels">
    <div class="grid gap-1">
      <span class="text-caption font-medium text-text-subtle">Tag</span>
      <TaskTagControl :task="task" />
    </div>
    <div class="grid gap-1">
      <span class="text-caption font-medium text-text-subtle">Skills</span>
      <SkillField :model-value="own" :inherited="fromTag" owner="task" @update:model-value="save" />
    </div>
    <TaskDependenciesField :task="task" stacked />
  </div>
  <div v-else class="flex min-w-0 flex-wrap items-center gap-x-3 gap-y-1.5" data-testid="task-labels">
    <TaskTagControl :task="task" />
    <span class="h-4 w-px bg-border" aria-hidden="true" />
    <SkillField :model-value="own" :inherited="fromTag" owner="task" @update:model-value="save" />
    <TaskDependenciesField :task="task" />
  </div>
</template>
