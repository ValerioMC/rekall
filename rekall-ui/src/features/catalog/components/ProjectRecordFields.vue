<script setup lang="ts">
import { computed, ref } from 'vue'
import AppSelect from '@/components/ui/AppSelect.vue'
import ProjectFolderField from '@/features/catalog/components/record/ProjectFolderField.vue'
import ProjectIconField from '@/features/catalog/components/record/ProjectIconField.vue'
import RecordDescriptionField from '@/features/catalog/components/record/RecordDescriptionField.vue'
import RecordIdentityFields from '@/features/catalog/components/record/RecordIdentityFields.vue'
import RecordStatusField from '@/features/catalog/components/record/RecordStatusField.vue'
import { PROJECT_STATUSES, PROJECT_STATUS_LABEL } from '@/features/catalog/model/catalog'
import type { ProjectDraft } from '@/features/catalog/model/record-draft'
import { useConsoleStore } from '@/stores/console.store'
import type { CompanyId, ProjectId } from '@/model/branded'

/** The fields of a project: its company, name, status, folder (when new), picture and description. */
defineProps<{ titleError: string | null; labelError: string | null }>()

const draft = defineModel<ProjectDraft>({ required: true })

const store = useConsoleStore()
const iconField = ref<InstanceType<typeof ProjectIconField> | null>(null)
const originalLabel = draft.value.label
const statusOptions = PROJECT_STATUSES.map((value) => ({ value, label: PROJECT_STATUS_LABEL[value] }))

const companyOptions = computed(() =>
  store.companies.map((company) => ({
    value: company.id as string,
    label: `${company.name}  ·  company:${company.name}`
  }))
)

function moveToCompany(companyId: string): void {
  draft.value.companyId = companyId as CompanyId
}

defineExpose({
  async applyImage(projectId: ProjectId): Promise<void> {
    await iconField.value?.apply(projectId)
  }
})
</script>

<template>
  <template v-if="companyOptions.length">
    <label for="record-parent" class="mb-1.5 block eyebrow text-caption">Company</label>
    <AppSelect
      id="record-parent"
      :model-value="draft.companyId"
      data-testid="record-parent"
      :options="companyOptions"
      @update:model-value="moveToCompany"
    />
    <p class="mb-5 mt-1.5 text-caption text-text-subtle">
      <template v-if="draft.id === null">Where this lands. It opens the anchor.</template>
      <template v-else>Changing this moves the record, and its anchor with it.</template>
    </p>
  </template>

  <RecordIdentityFields
    v-model:title="draft.title"
    v-model:label="draft.label"
    kind="project"
    :is-new="draft.id === null"
    :original-label="originalLabel"
    :title-error="titleError"
    :label-error="labelError"
    autofocus
  />
  <RecordStatusField v-model="draft.status" :options="statusOptions" />
  <ProjectFolderField v-if="draft.id === null" v-model="draft.repoFolder" />
  <ProjectIconField ref="iconField" v-model="draft.icon" :project-id="draft.id" />
  <RecordDescriptionField v-model="draft.description" />
</template>
