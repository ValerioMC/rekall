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

function lockedProjects(companyId: CompanyId): ConnectionProject[] {
  return projectsOf(lockOf(companyId))
}
</script>

<template>
  <div class="flex flex-col gap-2" data-testid="company-picker">
    <AppInput v-model="search" type="search" placeholder="Search companies" data-testid="company-search" aria-label="Search companies" />
    <ul class="flex max-h-64 flex-col divide-y divide-border overflow-y-auto rounded-control border border-border">
      <li v-for="company in visible" :key="company.id" class="px-3 py-2" data-testid="company-option">
        <div v-if="lockOf(company.id).length" class="flex items-start gap-2.5" data-testid="company-locked-row">
          <span
            class="mt-0.5 grid size-4 shrink-0 place-items-center text-text-subtle"
            role="checkbox"
            aria-checked="true"
            aria-disabled="true"
            :aria-label="`${company.name}, locked`"
          >
            <svg viewBox="0 0 16 16" class="size-4" fill="none" aria-hidden="true">
              <rect x="3.5" y="7" width="9" height="6.5" rx="1.5" stroke="currentColor" stroke-width="1.4" />
              <path d="M5.5 7V5.2a2.5 2.5 0 0 1 5 0V7" stroke="currentColor" stroke-width="1.4" />
            </svg>
          </span>
          <div class="min-w-0">
            <p class="text-body text-text">{{ company.name }}</p>
            <p class="mt-0.5 text-small text-text-muted" data-testid="company-locked">
              Locked: {{ plural(lockedProjects(company.id).length, 'project') }} of {{ company.name }}
              {{ lockedProjects(company.id).length === 1 ? 'binds' : 'bind' }} this connection:
              <template v-for="(project, at) in lockedProjects(company.id)" :key="project.projectId"
                ><RouterLink
                  :to="`/projects/${project.projectId}`"
                  target="_blank"
                  rel="noopener"
                  class="focus-ring rounded-key text-accent hover:underline"
                  >{{ project.title }}</RouterLink
                ><template v-if="at < lockedProjects(company.id).length - 1">, </template
                ><template v-else>.</template></template
              > Unbind them to stop sharing.
            </p>
          </div>
        </div>
        <AppCheckbox
          v-else
          :model-value="modelValue.includes(company.id)"
          :label="company.name"
          @update:model-value="toggle(company.id, $event)"
        />
      </li>
      <li v-if="!visible.length" class="px-3 py-3 text-small text-text-subtle">No company matches.</li>
    </ul>
  </div>
</template>
