<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import ProjectsSubNav from '@/features/catalog/components/ProjectsSubNav.vue'
import AppCatalogHeader from '@/features/catalog/components/AppCatalogHeader.vue'
import RecordDialog from '@/features/catalog/components/RecordDialog.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import AppEmptyState from '@/components/ui/AppEmptyState.vue'
import AppInput from '@/components/ui/AppInput.vue'
import CompanyMark from '@/components/ui/CompanyMark.vue'
import ProjectLedgerHead from '@/features/catalog/components/ProjectLedgerHead.vue'
import ProjectListToolbar from '@/features/catalog/components/ProjectListToolbar.vue'
import ProjectRow from '@/features/catalog/components/ProjectRow.vue'
import type { SegmentOption } from '@/components/ui/SegmentedControl.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useConsoleStore } from '@/stores/console.store'
import { useProjectWork } from '@/features/catalog/composables/useProjectWork'
import { projectComposition, reviewWaiting } from '@/features/catalog/lib/project-composition'
import { projectDraft } from '@/features/catalog/model/record-draft'
import { PROJECT_STATUSES, PROJECT_STATUS_LABEL } from '@/features/catalog/model/project-status'
import type { ProjectStatus, ProjectStatusChoice } from '@/features/catalog/model/project-status'
import type { RecordDraft } from '@/features/catalog/model/record-draft'
import type { Project } from '@/features/catalog/model/catalog'
import type { CompanyId } from '@/model/branded'

const store = useConsoleStore()
const route = useRoute()
const router = useRouter()
const { run } = useAsyncAction()
const { rowsOf, tokensOf } = useProjectWork()

const search = ref('')
const statusChoice = ref<ProjectStatusChoice>('ALL')
const statusFilter = computed(() => (statusChoice.value === 'ALL' ? null : statusChoice.value))
const companyFilter = ref<CompanyId | null>((route.query.company as CompanyId) ?? null)

const editing = ref<RecordDraft | null>(null)
const deleting = ref<Project | null>(null)

function matchesOutsideStatus(project: Project): boolean {
  if (companyFilter.value && project.companyId !== companyFilter.value) return false
  if (!search.value.trim()) return true
  const needle = search.value.toLowerCase().trim()
  const hay = `${project.title} ${project.label} ${project.companyName}`.toLowerCase()
  return hay.includes(needle)
}

/** Archived projects are listed only under their own filter, never among `All`. */
function matches(project: Project): boolean {
  if (statusFilter.value ? project.status !== statusFilter.value : project.status === 'ARCHIVED') return false
  return matchesOutsideStatus(project)
}

/**
 * How many projects each status option would show, given the search and the company filter.
 * Counted without the status filter itself, so every option says what pressing it gives.
 */
const statusCounts = computed(() => {
  const inScope = store.projects.filter(matchesOutsideStatus)
  const counts = new Map<ProjectStatus | null, number>([
    [null, inScope.filter((project) => project.status !== 'ARCHIVED').length]
  ])
  for (const status of PROJECT_STATUSES) {
    counts.set(status, inScope.filter((project) => project.status === status).length)
  }
  return counts
})

const statusOptions = computed<readonly SegmentOption<ProjectStatusChoice>[]>(() => [
  { value: 'ALL', label: 'All', count: statusCounts.value.get(null) ?? 0 },
  ...PROJECT_STATUSES.map((status) => ({
    value: status,
    label: PROJECT_STATUS_LABEL[status],
    count: statusCounts.value.get(status) ?? 0
  }))
])

const groups = computed(() =>
  store.companies
    .map((company) => ({
      company,
      projects: store.projects.filter((project) => project.companyId === company.id && matches(project))
    }))
    .filter((group) => group.projects.length > 0)
)

const totalMatching = computed(() => groups.value.reduce((sum, group) => sum + group.projects.length, 0))

const reviewingTotal = computed(() =>
  groups.value.reduce(
    (sum, group) => sum + group.projects.reduce((inGroup, project) => inGroup + reviewWaiting(rowsOf(project.id)), 0),
    0
  )
)

const targetCompany = computed(() => {
  if (companyFilter.value) return store.companies.find((c) => c.id === companyFilter.value) ?? null
  return store.companies[0] ?? null
})

function clearCompanyFilter(): void {
  companyFilter.value = null
  router.replace({ path: '/projects' })
}

function open(project: Project): void {
  router.push(`/projects/${project.id}`)
}

function newProject(): void {
  if (targetCompany.value) editing.value = projectDraft(targetCompany.value.id)
}

function blastOf(project: Project): string {
  const count = project.taskCount
  return `${count} task${count === 1 ? '' : 's'} · every note left on nothing`
}

async function confirmDelete(): Promise<void> {
  const project = deleting.value
  if (!project) return
  await run(() => store.deleteProject(project.id), `Deleted ${project.title}`)
  deleting.value = null
}
</script>

<template>
  <div class="flex-1 min-h-0 page-scroll bg-canvas">
    <AppCatalogHeader title="Projects">
      <template #actions>
        <AppButton
          variant="primary"
          size="sm"
          :disabled="!targetCompany"
          data-testid="new-project"
          @click="newProject"
        >
          + New project
        </AppButton>
      </template>
    </AppCatalogHeader>

    <ProjectsSubNav active="projects">
      <div class="w-70 max-w-full">
        <AppInput v-model="search" type="search" placeholder="Find a project" />
      </div>
    </ProjectsSubNav>

    <div class="page-column @container flex flex-col gap-3 py-4">
      <ProjectListToolbar
        v-model="statusChoice"
        :options="statusOptions"
        :company="targetCompany"
        :company-pinned="companyFilter !== null"
        :total="totalMatching"
        :reviewing="reviewingTotal"
        @clear-company="clearCompanyFilter"
      />

      <div v-if="store.isLoading" class="flex flex-col gap-2" aria-hidden="true">
        <div v-for="row in 6" :key="row" class="skeleton h-16 rounded-control" />
      </div>

      <AppEmptyState
        v-else-if="!groups.length && !store.companies.length"
        title="Nothing here yet"
        description="A project belongs to a company, and there isn't one yet. Start on the Companies tab."
      >
        <RouterLink to="/companies">
          <AppButton variant="secondary" size="sm">Go to companies</AppButton>
        </RouterLink>
      </AppEmptyState>

      <AppEmptyState
        v-else-if="!groups.length"
        title="No project matches"
        description="Try a different search, or clear the status filter."
      />

      <div v-else class="overflow-hidden rounded-card border border-border bg-surface">
        <ProjectLedgerHead />
        <section v-for="group in groups" :key="group.company.id" class="border-t border-border first:border-t-0">
          <h2 class="flex h-10 items-center gap-2.5 bg-canvas/50 px-4 text-body font-semibold text-text">
            <CompanyMark
              :company-id="group.company.id"
              :name="group.company.name"
              :image-version="group.company.logoImageVersion"
              :size="20"
            />
            {{ group.company.name }}
            <span class="font-mono text-caption font-normal text-text-subtle">
              {{ group.projects.length }} project{{ group.projects.length === 1 ? '' : 's' }}
            </span>
          </h2>
          <ProjectRow
            v-for="project in group.projects"
            :key="project.id"
            :project="project"
            :composition="projectComposition(rowsOf(project.id))"
            :reviewing="reviewWaiting(rowsOf(project.id))"
            :tokens="tokensOf(project.id)"
            @open="open(project)"
            @delete="deleting = project"
          />
        </section>
      </div>
    </div>

    <Transition name="dialog">
      <RecordDialog v-if="editing" :draft="editing" @close="editing = null" />
    </Transition>

    <Transition name="dialog">
      <AppConfirm
        v-if="deleting"
        :title="`Delete ${deleting.title}?`"
        body="Everything underneath goes with it. This is not recoverable."
        :blast="blastOf(deleting)"
        confirm-label="Delete project"
        @cancel="deleting = null"
        @confirm="confirmDelete"
      />
    </Transition>
  </div>
</template>
