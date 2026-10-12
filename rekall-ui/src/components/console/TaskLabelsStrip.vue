<script setup lang="ts">
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import SkillField from '@/components/skills/SkillField.vue'
import TagBadge from '@/components/ui/TagBadge.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useConsoleStore } from '@/stores/console.store'
import { skillsOfTag } from '@/model/skill-assignment'
import type { Task } from '@/model/catalog'

/**
 * The task's tag and its skills, in the description's header. The tag is chosen when the task
 * is edited; its skills arrive here dashed, since they are the tag's to change. The task's own
 * skills are changed in place.
 */
const props = defineProps<{ task: Task }>()

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
  <div class="mt-2 flex min-w-0 flex-wrap items-center gap-x-3 gap-y-1.5" data-testid="task-labels">
    <TagBadge
      v-if="task.tagId && task.tagName && task.tagIcon && task.tagColor"
      :name="task.tagName"
      :icon="task.tagIcon"
      :color="task.tagColor"
      size="md"
    />
    <span v-else class="text-caption text-text-subtle" data-testid="task-no-tag">No tag</span>
    <span class="h-4 w-px bg-border" aria-hidden="true" />
    <SkillField :model-value="own" :inherited="fromTag" owner="task" @update:model-value="save" />
  </div>
</template>
