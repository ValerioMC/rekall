<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import AppCatalogHeader from '@/components/catalog/AppCatalogHeader.vue'
import RecordDialog from '@/components/console/RecordDialog.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import AppEmptyState from '@/components/ui/AppEmptyState.vue'
import AppInput from '@/components/ui/AppInput.vue'
import ProjectTrace from '@/components/ui/ProjectTrace.vue'
import StatusMixBar from '@/components/ui/StatusMixBar.vue'
import CloseGlyph from '@/components/ui/CloseGlyph.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { relativeTime } from '@/common/format/relative-time'
import { useConsoleStore } from '@/stores/console.store'
import { hasActivity, projectActivitySeries } from '@/common/trace/activity-series'
import { projectDraft } from '@/model/record-draft'
import { identityHue } from '@/common/identity'
import { PROJECT_STATUSES, PROJECT_STATUS_LABEL } from '@/model/catalog'
import type { ProjectStatus, Task } from '@/model/catalog'
import type { RecordDraft } from '@/model/record-draft'
import type { Project } from '@/model/catalog'
import type { CompanyId, ProjectId } from '@/model/branded'

const store = useConsoleStore()
const route = useRoute()
const router = useRouter()
const { run } = useAsyncAction()

const search = ref('')
const statusFilter = ref<ProjectStatus | null>(null)
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

function matches(project: Project): boolean {
  if (statusFilter.value && project.status !== statusFilter.value) return false
  return matchesOutsideStatus(project)
}

/**
 * How many projects each status option would show, given the search and the company filter.
 * Counted without the status filter itself, so every option says what pressing it gives.
 */
const statusCounts = computed(() => {
  const inScope = store.projects.filter(matchesOutsideStatus)
  const counts = new Map<ProjectStatus | null, number>([[null, inScope.length]])
  for (const status of PROJECT_STATUSES) {
    counts.set(status, inScope.filter((project) => project.status === status).length)
  }
  return counts
})

const STATUS_OPTIONS: readonly { value: ProjectStatus | null; label: string }[] = [
  { value: null, label: 'All' },
  ...PROJECT_STATUSES.map((status) => ({ value: status, label: PROJECT_STATUS_LABEL[status] }))
]

const groups = computed(() =>
  store.companies
    .map((company) => ({
      company,
      projects: store.projects.filter((project) => project.companyId === company.id && matches(project))
    }))
    .filter((group) => group.projects.length > 0)
)

const totalMatching = computed(() => groups.value.reduce((sum, group) => sum + group.projects.length, 0))

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

function tasksOf(projectId: ProjectId): Task[] {
  return store.tasks.filter((t) => t.projectId === projectId)
}

function activitySeries(projectId: ProjectId): number[] | undefined {
  const series = projectActivitySeries(projectId, store.tasks, store.timeEntries, Date.now())
  return hasActivity(series) ? series : undefined
}

async function confirmDelete(): Promise<void> {
  const project = deleting.value
  if (!project) return
  await run(() => store.deleteProject(project.id), `Deleted ${project.title}`)
  deleting.value = null
}
</script>

<template>
  <div class="flex-1 min-h-0 bg-canvas">
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

    <div class="mx-auto max-w-[1240px] px-8 py-6">
      <div class="mb-5 flex flex-wrap items-center gap-2.5">
        <div class="w-full max-w-[320px]">
          <AppInput v-model="search" type="search" placeholder="Find a project" />
        </div>
        <!-- One segmented control rather than four loose pills: the options are exclusive, so
             they read as one choice, sit at the search field's height, and each carries the
             count it would show. -->
        <div
          class="flex h-(--spacing-control) items-center gap-0.5 rounded-[var(--radius-control)] border border-border bg-surface p-0.5"
          role="radiogroup"
          aria-label="Status"
        >
          <button
            v-for="option in STATUS_OPTIONS"
            :key="option.label"
            type="button"
            role="radio"
            :aria-checked="statusFilter === option.value"
            class="focus-ring flex h-full items-center gap-1.5 rounded-[calc(var(--radius-control)-3px)] px-2.5 text-[12.5px] transition-colors"
            :class="
              statusFilter === option.value
                ? 'bg-surface-raised text-text shadow-[0_1px_2px_rgb(0_0_0/0.4)]'
                : 'text-text-subtle hover:text-text'
            "
            @click="statusFilter = option.value"
          >
            {{ option.label }}
            <span
              class="font-mono text-[10.5px] tabular-nums"
              :class="statusFilter === option.value ? 'text-accent' : 'text-text-subtle/70'"
            >
              {{ statusCounts.get(option.value) ?? 0 }}
            </span>
          </button>
        </div>

        <button
          v-if="companyFilter"
          class="anchor-chip focus-ring ml-1 flex items-center gap-1.5 px-2.5 py-1 text-[11.5px]"
          @click="clearCompanyFilter"
        >
          <span>{{ targetCompany?.name ?? 'one company' }}</span>
          <CloseGlyph small />
        </button>

        <span class="ml-auto font-mono text-[11.5px] text-text-subtle">
          {{ totalMatching }} project{{ totalMatching === 1 ? '' : 's' }}
        </span>
      </div>

      <div v-if="store.isLoading" class="flex flex-col gap-6" aria-hidden="true">
        <div v-for="group in 2" :key="group" class="flex flex-col gap-2">
          <div class="skeleton h-3 w-32" />
          <div v-for="row in 3" :key="row" class="skeleton h-[60px] rounded-[var(--radius-control)]" />
        </div>
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

      <div v-else class="flex flex-col gap-7">
        <section v-for="group in groups" :key="group.company.id">
          <h2 class="mb-2 eyebrow text-[11px]">
            {{ group.company.name }}
          </h2>
          <div class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
            <div
              v-for="project in group.projects"
              :key="project.id"
              class="group/card relative flex flex-col gap-3 rounded-[var(--radius-card)] border border-border bg-surface p-4 transition-all duration-150 hover:-translate-y-0.5 hover:border-border-strong hover:bg-surface-raised hover:shadow-lift"
            >
              <!-- The project's own hue along the top edge, the same line its pane header
                   carries in the console, so a card is recognisable before it is read. -->
              <span
                class="pointer-events-none absolute inset-x-4 top-0 h-px opacity-70 transition-opacity group-hover/card:opacity-100"
                :style="{ background: `linear-gradient(90deg, transparent, ${identityHue(project.id).base}, transparent)` }"
                aria-hidden="true"
              />
              <button
                data-testid="project-card"
                class="focus-ring absolute inset-0 rounded-[var(--radius-card)]"
                :aria-label="`Open ${project.title}`"
                @click="open(project)"
              />

              <div class="pointer-events-none flex items-center justify-between gap-2">
                <ProjectTrace :id="project.id" size="md" :series="activitySeries(project.id)" />
                <AppBadge
                  class="shrink-0"
                  :tone="project.status === 'ACTIVE' ? 'accent' : project.status === 'DONE' ? 'safe' : 'neutral'"
                >
                  {{ PROJECT_STATUS_LABEL[project.status] }}
                </AppBadge>
              </div>

              <div class="pointer-events-none min-w-0">
                <p class="line-clamp-2 text-[14px] font-semibold leading-snug text-text">{{ project.title }}</p>
                <span class="anchor-chip mt-1.5 inline-block px-1.5 py-px text-[10px]">{{ project.anchor }}</span>
              </div>

              <StatusMixBar class="pointer-events-none" :tasks="tasksOf(project.id)" />

              <div class="pointer-events-none mt-auto flex items-center justify-between text-[11px] text-text-subtle">
                <span>{{ project.taskCount }} task{{ project.taskCount === 1 ? '' : 's' }}</span>
                <!-- Steps aside for the delete control under the pointer, rather than the card
                     keeping an empty row at its foot for a control that is hidden at rest. -->
                <span class="transition-opacity duration-100 group-hover/card:opacity-0 group-focus-within/card:opacity-0">
                  {{ relativeTime(project.updatedAt) }}
                </span>
              </div>

              <button
                class="focus-ring pointer-events-auto absolute bottom-2.5 right-2.5 z-10 grid size-7 place-items-center rounded text-text-subtle opacity-0 transition hover:bg-surface-hover hover:text-danger focus-visible:opacity-100 group-hover/card:opacity-100"
                :aria-label="`Delete ${project.title}`"
                data-testid="delete-project"
                @click.stop="deleting = project"
              >
                <svg class="size-3.5" viewBox="0 0 24 24" fill="none" aria-hidden="true">
                  <path
                    d="M5 7h14M9 7V5a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2m1 0-.6 12.4a2 2 0 0 1-2 1.6H9.6a2 2 0 0 1-2-1.6L7 7"
                    stroke="currentColor"
                    stroke-width="1.6"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  />
                </svg>
              </button>
            </div>
          </div>
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
