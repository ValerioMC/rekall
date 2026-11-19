<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import ProjectsSubNav from '@/features/catalog/components/ProjectsSubNav.vue'
import AppCatalogHeader from '@/features/catalog/components/AppCatalogHeader.vue'
import RecordDialog from '@/features/catalog/components/RecordDialog.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import AppEmptyState from '@/components/ui/AppEmptyState.vue'
import AppInput from '@/components/ui/AppInput.vue'
import CompanyMark from '@/components/ui/CompanyMark.vue'
import CompanyDetail from '@/features/catalog/components/CompanyDetail.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { projectComposition } from '@/features/catalog/lib/project-composition'
import { workRows } from '@/features/work/lib/work-rows'
import { useConsoleStore } from '@/stores/console.store'
import { useConnectionsStore } from '@/features/connection/store/connections.store'
import { plural } from '@/features/connection/lib/connection-impact'
import { companyDraft, projectDraft } from '@/features/catalog/model/record-draft'
import type { RecordDraft } from '@/features/catalog/model/record-draft'
import type { Company } from '@/features/catalog/model/catalog'

const store = useConsoleStore()
const router = useRouter()
const route = useRoute()
const connections = useConnectionsStore()
const { run } = useAsyncAction()

onMounted(() => run(() => connections.ensureLoaded()))

const search = ref('')
const editing = ref<RecordDraft | null>(null)
const deleting = ref<Company | null>(null)

const visible = computed(() => {
  const needle = search.value.toLowerCase().trim()
  if (!needle) return store.companies
  return store.companies.filter((company) => company.name.toLowerCase().includes(needle))
})

const selected = computed<Company | null>(
  () => visible.value.find((company) => company.id === route.params.id) ?? visible.value[0] ?? null
)

function select(company: Company): void {
  router.push(`/companies/${company.id}`)
}

function openConnectionsOf(company: Company): void {
  router.push({ name: 'connections', query: { company: company.id } })
}

function connectionsLabelOf(company: Company): string {
  if (!connections.loaded) return 'Connections'
  const count = connections.connectionsOf(company.id).length
  return count === 0 ? 'No connections' : plural(count, 'connection')
}

function edit(company: Company): void {
  editing.value = companyDraft(company)
}

const projectsOfSelected = computed(() => store.projects.filter((project) => project.companyId === selected.value?.id))

const compositionOfSelected = computed(() => {
  const projectIds = new Set(projectsOfSelected.value.map((project) => project.id))
  const tasks = store.tasks.filter((task) => projectIds.has(task.projectId))
  return projectComposition(workRows(tasks, store.steps, () => 0))
})

function newProjectOfSelected(): void {
  if (selected.value) editing.value = projectDraft(selected.value.id)
}

function blastOf(company: Company): string {
  return `${company.projectCount} project${company.projectCount === 1 ? '' : 's'} · ${company.taskCount} task${company.taskCount === 1 ? '' : 's'} · every note left on nothing`
}

async function confirmDelete(): Promise<void> {
  const company = deleting.value
  if (!company) return
  await run(() => store.deleteCompany(company.id), `Deleted ${company.name}`)
  deleting.value = null
}
</script>

<template>
  <div class="flex-1 min-h-0 bg-canvas">
    <AppCatalogHeader title="Companies">
      <template #actions>
        <AppButton
          variant="primary"
          size="sm"
          data-testid="new-company"
          @click="editing = companyDraft()"
        >
          + New company
        </AppButton>
      </template>
    </AppCatalogHeader>

    <div class="page-column pt-4">
      <ProjectsSubNav active="companies" />
    </div>

    <div class="page-column py-6">
      <div class="mb-5 w-full max-w-[320px]">
        <AppInput v-model="search" type="search" placeholder="Find a company" />
      </div>

      <div v-if="store.isLoading" class="flex flex-col gap-2" aria-hidden="true">
        <div v-for="row in 5" :key="row" class="skeleton h-[64px] rounded-control" />
      </div>

      <AppEmptyState
        v-else-if="!visible.length && !store.companies.length"
        title="No company yet"
        description="A company is the outermost record: it holds the projects, and those hold the work."
      >
        <AppButton variant="secondary" size="sm" @click="editing = companyDraft()">
          + New company
        </AppButton>
      </AppEmptyState>

      <AppEmptyState v-else-if="!visible.length" title="No company matches" description="Try a different search." />

      <div v-else class="grid overflow-hidden rounded-card border border-border bg-surface min-[900px]:grid-cols-[320px_minmax(0,1fr)]">
        <nav class="border-border min-[900px]:border-r" aria-label="Companies">
          <button
            v-for="(company, index) in visible"
            :key="company.id"
            data-testid="company-card"
            class="focus-ring flex w-full items-center gap-3 px-4 py-3 text-left transition-colors hover:bg-surface-raised"
            :class="[index > 0 ? 'border-t border-border' : '', company.id === selected?.id ? 'bg-accent-soft' : '']"
            :aria-current="company.id === selected?.id ? 'true' : undefined"
            @click="select(company)"
          >
            <CompanyMark
              :company-id="company.id"
              :name="company.name"
              :image-version="company.logoImageVersion"
              :size="32"
            />
            <span class="min-w-0 flex-1">
              <span class="block truncate text-body font-semibold text-text">{{ company.name }}</span>
              <span class="block text-caption text-text-subtle">
                {{ plural(company.projectCount, 'project') }} · {{ plural(company.taskCount, 'task') }}
              </span>
            </span>
          </button>
        </nav>
        <CompanyDetail
          v-if="selected"
          :key="selected.id"
          :company="selected"
          :projects="projectsOfSelected"
          :composition="compositionOfSelected"
          :connections-label="connectionsLabelOf(selected)"
          :connections-empty="connections.loaded && connections.connectionsOf(selected.id).length === 0"
          @edit="edit(selected)"
          @delete="deleting = selected"
          @new-project="newProjectOfSelected"
          @open-connections="openConnectionsOf(selected)"
        />
      </div>
    </div>

    <Transition name="dialog">
      <RecordDialog v-if="editing" :draft="editing" @close="editing = null" />
    </Transition>

    <Transition name="dialog">
      <AppConfirm
        v-if="deleting"
        :title="`Delete ${deleting.name}?`"
        body="Everything underneath goes with it. This is not recoverable."
        :blast="blastOf(deleting)"
        confirm-label="Delete company"
        @cancel="deleting = null"
        @confirm="confirmDelete"
      />
    </Transition>
  </div>
</template>
