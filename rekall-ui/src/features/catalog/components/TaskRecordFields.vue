<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import ProjectPicker from '@/features/catalog/components/ProjectPicker.vue'
import RecordIdentityFields from '@/features/catalog/components/record/RecordIdentityFields.vue'
import RecordStatusField from '@/features/catalog/components/record/RecordStatusField.vue'
import TaskEpicField from '@/features/catalog/components/record/TaskEpicField.vue'
import TaskKindField from '@/features/catalog/components/record/TaskKindField.vue'
import TaskSkillsField from '@/features/catalog/components/record/TaskSkillsField.vue'
import TaskTagField from '@/features/catalog/components/record/TaskTagField.vue'
import { TASK_STATUSES, TASK_STATUS_LABEL } from '@/features/catalog/model/task-status'
import { epicChoices } from '@/features/catalog/model/epic'
import { taskCodeOf } from '@/features/catalog/model/task-code'
import type { TaskDraft } from '@/features/catalog/model/record-draft'
import { useConsoleStore } from '@/stores/console.store'
import type { ProjectId } from '@/model/branded'

/**
 * The fields of a task: its project, name, status, kind, epic, tag and, when new, its skills. The
 * description is written in the task's own pane. `picking` says whether a picker is open, so the
 * dialog does not dismiss on the click that closes it.
 */
defineProps<{ titleError: string | null }>()

const emit = defineEmits<{ picking: [open: boolean] }>()

const draft = defineModel<TaskDraft>({ required: true })

const store = useConsoleStore()
const isPickingProject = ref(false)
const isPickingEpic = ref(false)
const statusOptions = TASK_STATUSES.map((value) => ({ value, label: TASK_STATUS_LABEL[value] }))

const epics = computed(() => epicChoices(draft.value.projectId, store.tasks, draft.value.id))

const code = computed(() => taskCodeOf(draft.value, store.projects, store.tasks))

watch([isPickingProject, isPickingEpic], ([project, epic]) => emit('picking', project || epic))

function moveToProject(projectId: string): void {
  if (draft.value.projectId !== projectId) draft.value.epicId = null
  draft.value.projectId = projectId as ProjectId
}

function setSkills(skills: readonly string[]): void {
  draft.value.skills = [...skills]
}
</script>

<template>
  <label for="record-parent" class="mb-1.5 block eyebrow text-caption">Project</label>
  <ProjectPicker
    id="record-parent"
    :model-value="draft.projectId"
    data-testid="record-parent"
    :projects="store.projects"
    @update:model-value="moveToProject"
    @open-change="isPickingProject = $event"
  />
  <p class="mb-5 mt-1.5 text-caption text-text-subtle">
    <template v-if="draft.id === null">Where this lands. Its key opens the task code.</template>
    <template v-else>Changing this moves the task, and gives it the next code of that project.</template>
  </p>

  <RecordIdentityFields
    v-model:title="draft.title"
    kind="task"
    :is-new="draft.id === null"
    :parent-id="draft.projectId"
    :task-code="code"
    :title-error="titleError"
    autofocus
  />
  <RecordStatusField v-model="draft.status" :options="statusOptions" />
  <TaskKindField v-model="draft.taskKind" />
  <TaskEpicField
    v-if="draft.taskKind === 'TASK'"
    v-model="draft.epicId"
    :epics="epics"
    @open-change="isPickingEpic = $event"
  />
  <TaskTagField v-model="draft.tagId" />
  <TaskSkillsField
    v-if="draft.id === null"
    :model-value="draft.skills"
    :tag-id="draft.tagId"
    @update:model-value="setSkills"
  />
</template>
