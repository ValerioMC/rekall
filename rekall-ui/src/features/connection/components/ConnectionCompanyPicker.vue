<script setup lang="ts">
import { computed, ref } from 'vue'
import { RouterLink } from 'vue-router'
import AppCheckbox from '@/components/ui/AppCheckbox.vue'
import AppInput from '@/components/ui/AppInput.vue'
import { plural, projectsOf } from '@/features/connection/lib/connection-impact'
import type { ConnectionProject } from '@/features/connection/model/connection'
import type { CompanyId } from '@/model/branded'

/**
 * A checklist of companies with a search. A company whose projects still bind the connection is
 * checked and locked, with those projects named as links: the way to release it is to unbind them.
 */
const props = withDefaults(
  defineProps<{
    companies: readonly { readonly id: CompanyId; readonly name: string }[]
    modelValue: readonly CompanyId[]
    blockedBy?: ReadonlyMap<CompanyId, readonly ConnectionProject[]>
  }>(),
  { blockedBy: () => new Map() }
)
const emit = defineEmits<{ 'update:modelValue': [companyIds: readonly CompanyId[]] }>()

const search = ref('')
const visible = computed(() => {
  const needle = search.value.trim().toLowerCase()
  return props.companies.filter((company) => company.name.toLowerCase().includes(needle))
})

function toggle(companyId: CompanyId, on: boolean): void {
  const rest = props.modelValue.filter((known) => known !== companyId)
  emit('update:modelValue', on ? [...rest, companyId] : rest)
}

function lockOf(companyId: CompanyId): readonly ConnectionProject[] {
  return props.blockedBy.get(companyId) ?? []
}
</script>

<template>
  <div class="flex flex-col gap-2" data-testid="company-picker">
    <AppInput v-model="search" type="search" placeholder="Search companies" data-testid="company-search" aria-label="Search companies" />
    <ul class="flex max-h-64 flex-col divide-y divide-border overflow-y-auto rounded-control border border-border">
      <li v-for="company in visible" :key="company.id" class="px-3 py-2" data-testid="company-option">
        <AppCheckbox
          :model-value="modelValue.includes(company.id)"
          :label="company.name"
          :disabled="lockOf(company.id).length > 0"
          @update:model-value="toggle(company.id, $event)"
        />
        <p v-if="lockOf(company.id).length" class="mt-1 pl-6.5 text-caption text-text-subtle" data-testid="company-locked">
          {{ plural(projectsOf(lockOf(company.id)).length, 'project') }} of {{ company.name }} bind this connection:
          <template v-for="(project, at) in projectsOf(lockOf(company.id))" :key="project.projectId">
            <RouterLink :to="`/projects/${project.projectId}`" class="focus-ring rounded-key text-accent hover:underline">{{
              project.title
            }}</RouterLink
            ><template v-if="at < projectsOf(lockOf(company.id)).length - 1">, </template>
          </template>
          . Unbind them to stop sharing.
        </p>
      </li>
      <li v-if="!visible.length" class="px-3 py-3 text-small text-text-subtle">No company matches.</li>
    </ul>
  </div>
</template>
