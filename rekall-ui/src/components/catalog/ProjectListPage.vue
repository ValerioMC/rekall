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
import { useAsyncAction } from '@/composables/useAsyncAction'
import { relativeTime } from '@/common/format/relative-time'
import { useConsoleStore } from '@/stores/console.store'
import { projectDraft } from '@/model/record-draft'
import { PROJECT_STATUSES, PROJECT_STATUS_LABEL } from '@/model/catalog'
import type { ProjectStatus } from '@/model/catalog'
import type { RecordDraft } from '@/model/record-draft'
import type { Project } from '@/model/catalog'
import type { CompanyId } from '@/model/branded'

const store = useConsoleStore()
const route = useRoute()
const router = useRouter()
const { run } = useAsyncAction()

const search = ref('')
const statusFilter = ref<ProjectStatus | null>(null)
const companyFilter = ref<CompanyId | null>((route.query.company as CompanyId) ?? null)

const editing = ref<RecordDraft | null>(null)
const deleting = ref<Project | null>(null)

function matches(project: Project): boolean {
  if (statusFilter.value && project.status !== statusFilter.value) return false
  if (companyFilter.value && project.companyId !== companyFilter.value) return false
  if (!search.value.trim()) return true
  const needle = search.value.toLowerCase().trim()
  const hay = `${project.title} ${project.label} ${project.companyName}`.toLowerCase()
  return hay.includes(needle)
}

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

async function confirmDelete(): Promise<void> {
  const project = deleting.value
  if (!project) return
  await run(() => store.deleteProject(project.id), `Deleted ${project.title}`)
  deleting.value = null
}
</script>

<template>
  <div class="min-h-full bg-canvas">
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
        <button
          class="focus-ring h-8 rounded-[var(--radius-control)] border px-3 text-[12.5px] transition-colors"
          :class="
            statusFilter === null
              ? 'border-accent bg-accent-soft text-accent'
              : 'border-border-strong bg-canvas text-text-muted hover:border-text-subtle hover:text-text'
          "
          @click="statusFilter = null"
        >
          All
        </button>
        <button
          v-for="option in PROJECT_STATUSES"
          :key="option"
          class="focus-ring h-8 rounded-[var(--radius-control)] border px-3 text-[12.5px] transition-colors"
          :class="
            statusFilter === option
              ? 'border-accent bg-accent-soft text-accent'
              : 'border-border-strong bg-canvas text-text-muted hover:border-text-subtle hover:text-text'
          "
          @click="statusFilter = option"
        >
          {{ PROJECT_STATUS_LABEL[option] }}
        </button>

        <button
          v-if="companyFilter"
          class="anchor-chip focus-ring ml-1 flex items-center gap-1.5 px-2.5 py-1 text-[11.5px]"
          @click="clearCompanyFilter"
        >
          <span>{{ targetCompany?.name ?? 'one company' }}</span>
          <span aria-hidden="true">&times;</span>
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
          <h2 class="mb-2 text-[11px] font-semibold uppercase tracking-[0.09em] text-text-subtle">
            {{ group.company.name }}
          </h2>
          <div class="overflow-hidden rounded-[var(--radius-card)] border border-border bg-surface">
            <button
              v-for="(project, index) in group.projects"
              :key="project.id"
              data-testid="project-card"
              class="focus-ring group/row relative flex w-full items-center gap-4 py-3.5 pl-4 pr-4 text-left transition-colors hover:bg-surface-raised"
              :class="index > 0 ? 'border-t border-border' : ''"
              @click="open(project)"
            >
              <span
                class="h-8 w-[3px] shrink-0 rounded-full"
                :class="
                  project.status === 'ACTIVE'
                    ? 'bg-accent'
                    : project.status === 'DONE'
                      ? 'bg-safe'
                      : 'bg-text-subtle'
                "
                aria-hidden="true"
              />

              <span class="min-w-0 flex-1">
                <span class="flex flex-wrap items-center gap-x-2 gap-y-1">
                  <span class="truncate text-[14px] font-semibold text-text">{{ project.title }}</span>
                  <span class="anchor-chip shrink-0 px-1.5 py-px text-[10px]">{{ project.anchor }}</span>
                </span>
              </span>

              <span class="hidden shrink-0 items-center gap-2 text-[11.5px] text-text-subtle sm:flex">
                <span>{{ project.taskCount }} task{{ project.taskCount === 1 ? '' : 's' }}</span>
                <span aria-hidden="true">&middot;</span>
                <span>{{ relativeTime(project.updatedAt) }}</span>
              </span>

              <AppBadge
                class="shrink-0"
                :tone="project.status === 'ACTIVE' ? 'accent' : project.status === 'DONE' ? 'safe' : 'neutral'"
              >
                {{ PROJECT_STATUS_LABEL[project.status] }}
              </AppBadge>

              <button
                class="focus-ring grid size-7 shrink-0 place-items-center rounded text-text-subtle opacity-0 transition hover:bg-surface-hover hover:text-danger focus-visible:opacity-100 group-hover/row:opacity-100"
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

              <span
                class="shrink-0 text-[13px] text-text-subtle transition-transform group-hover/row:translate-x-0.5 group-hover/row:text-accent"
                aria-hidden="true"
              >
                &#8594;
              </span>
            </button>
          </div>
        </section>
      </div>
    </div>

    <RecordDialog v-if="editing" :draft="editing" @close="editing = null" />

    <AppConfirm
      v-if="deleting"
      :title="`Delete ${deleting.title}?`"
      body="Everything underneath goes with it. This is not recoverable."
      :blast="blastOf(deleting)"
      confirm-label="Delete project"
      @cancel="deleting = null"
      @confirm="confirmDelete"
    />
  </div>
</template>
