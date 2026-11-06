<script setup lang="ts">
import { computed } from 'vue'
import AppCheckbox from '@/components/ui/AppCheckbox.vue'
import type { CompanyId, ProjectId } from '@/model/branded'

/** The projects of the chosen companies, to tick the ones that use the new connection. */
const props = defineProps<{
  companies: readonly { readonly id: CompanyId; readonly name: string }[]
  projects: readonly { readonly id: ProjectId; readonly title: string; readonly companyId: CompanyId }[]
  modelValue: readonly ProjectId[]
}>()
const emit = defineEmits<{ 'update:modelValue': [projectIds: readonly ProjectId[]] }>()

const groups = computed(() =>
  props.companies
    .map((company) => ({ company, projects: props.projects.filter((project) => project.companyId === company.id) }))
    .filter((group) => group.projects.length > 0)
)

function toggle(projectId: ProjectId, on: boolean): void {
  const rest = props.modelValue.filter((known) => known !== projectId)
  emit('update:modelValue', on ? [...rest, projectId] : rest)
}
</script>

<template>
  <div class="flex flex-col gap-2" data-testid="project-picker">
    <p v-if="!groups.length" class="text-small text-text-subtle" data-testid="project-picker-empty">
      The chosen companies have no active project yet.
    </p>
    <ul v-else class="flex max-h-56 flex-col divide-y divide-border overflow-y-auto rounded-control border border-border">
      <li v-for="group in groups" :key="group.company.id" class="px-3 py-2">
        <p v-if="companies.length > 1" class="mb-1 text-caption text-text-subtle">{{ group.company.name }}</p>
        <ul class="flex flex-col gap-1.5">
          <li v-for="project in group.projects" :key="project.id" data-testid="project-option">
            <AppCheckbox :model-value="modelValue.includes(project.id)" :label="project.title" @update:model-value="toggle(project.id, $event)" />
          </li>
        </ul>
      </li>
    </ul>
  </div>
</template>
