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
import CompanyDetail from '@/features/catalog/components/CompanyDetail.vue'
import CompanyRailItem from '@/features/catalog/components/CompanyRailItem.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useProjectWork } from '@/features/catalog/composables/useProjectWork'
import { projectComposition, reviewWaiting } from '@/features/catalog/lib/project-composition'
import type { ProjectComposition } from '@/features/catalog/lib/project-composition'
import { useConsoleStore } from '@/stores/console.store'
import { useConnectionsStore } from '@/features/connection/store/connections.store'
import { plural } from '@/features/connection/lib/connection-impact'
import { companyDraft, projectDraft } from '@/features/catalog/model/record-draft'
import type { RecordDraft } from '@/features/catalog/model/record-draft'
import type { Company, Project } from '@/features/catalog/model/catalog'
import type { ProjectLedgerEntry } from '@/features/catalog/model/project-ledger-entry'
import type { CompanyId } from '@/model/branded'

const store = useConsoleStore()
const router = useRouter()
const route = useRoute()
const connections = useConnectionsStore()
const { run } = useAsyncAction()
const { rowsOf, tokensOf } = useProjectWork()

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

const entriesOfSelected = computed<readonly ProjectLedgerEntry[]>(() =>
  projectsOfSelected.value.map((project) => ({
    project,
    composition: projectComposition(rowsOf(project.id)),
    reviewing: reviewWaiting(rowsOf(project.id)),
    tokens: tokensOf(project.id)
  }))
)

interface CompanyWork {
  readonly composition: ProjectComposition
  readonly reviewing: number
}

const workByCompany = computed(() => {
  const byCompany = new Map<CompanyId, CompanyWork>()
  for (const company of store.companies) {
    const rows = store.projects.filter((project) => project.companyId === company.id).flatMap((project) => rowsOf(project.id))
    byCompany.set(company.id, { composition: projectComposition(rows), reviewing: reviewWaiting(rows) })
  }
  return byCompany
})

const NO_WORK: CompanyWork = { composition: { filed: 0, review: 0, active: 0, todo: 0 }, reviewing: 0 }

function workOf(company: Company): CompanyWork {
  return workByCompany.value.get(company.id) ?? NO_WORK
}

function openProject(project: Project): void {
  router.push(`/projects/${project.id}`)
}

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
  <div class="flex-1 min-h-0 page-scroll bg-canvas">
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
      <ProjectsSubNav active="companies">
        <div class="w-70 max-w-full">
          <AppInput v-model="search" type="search" placeholder="Find a company" />
        </div>
      </ProjectsSubNav>
    </div>

    <div class="page-column @container py-5">
      <div v-if="store.isLoading" class="flex flex-col gap-2" aria-hidden="true">
        <div v-for="row in 5" :key="row" class="skeleton h-16 rounded-control" />
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

      <div v-else class="grid grid-cols-1 items-start gap-4 @min-[56rem]:grid-cols-[340px_minmax(0,1fr)] @min-[56rem]:gap-6">
        <nav
          class="flex gap-2 overflow-x-auto pb-1 @min-[56rem]:grid @min-[56rem]:gap-1 @min-[56rem]:overflow-visible @min-[56rem]:pb-0"
          aria-label="Companies"
        >
          <CompanyRailItem
            v-for="company in visible"
            :key="company.id"
            :company="company"
            :composition="workOf(company).composition"
            :reviewing="workOf(company).reviewing"
            :selected="company.id === selected?.id"
            @select="select(company)"
          />
        </nav>
        <CompanyDetail
          v-if="selected"
          :key="selected.id"
          :company="selected"
          :entries="entriesOfSelected"
          :composition="workOf(selected).composition"
          :connections-label="connectionsLabelOf(selected)"
          :connections-empty="connections.loaded && connections.connectionsOf(selected.id).length === 0"
          @edit="edit(selected)"
          @delete="deleting = selected"
          @new-project="newProjectOfSelected"
          @open-connections="openConnectionsOf(selected)"
          @open-project="openProject"
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
