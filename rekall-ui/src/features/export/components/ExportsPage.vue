<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { RouterLink } from 'vue-router'
import AppButton from '@/components/ui/AppButton.vue'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import AppEmptyState from '@/components/ui/AppEmptyState.vue'
import AppCatalogHeader from '@/features/catalog/components/AppCatalogHeader.vue'
import LibrarySubNav from '@/features/diagram/components/LibrarySubNav.vue'
import ExportCompanyGroup from '@/features/export/components/ExportCompanyGroup.vue'
import ExportsToolbar from '@/features/export/components/ExportsToolbar.vue'
import { filterExports, isNarrowing, NO_EXPORT_FILTER, type ExportFilter } from '@/features/export/lib/export-filter'
import { groupExports } from '@/features/export/lib/export-grouping'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useConsoleStore } from '@/stores/console.store'
import type { ExportProjectGroup } from '@/features/export/lib/export-grouping'
import type { ProjectExportClear } from '@/features/export/model/project-export'
import type { ProjectExportId } from '@/model/branded'

/**
 * Every export of every project, grouped by company and then by project, to download or to delete.
 * Filters and search work over the list already loaded; Refresh reads it again.
 */
const store = useConsoleStore()
const { run } = useAsyncAction()

const filter = ref<ExportFilter>(NO_EXPORT_FILTER)
const loading = ref(false)
const loadFailure = ref<string | null>(null)
const asking = ref<{ readonly title: string; readonly body: string; readonly confirmLabel: string; readonly clear: ProjectExportClear } | null>(null)

const entries = computed(() => store.exportEntries)
const shown = computed(() => filterExports(entries.value, filter.value))
const groups = computed(() => groupExports(shown.value))
const narrowing = computed(() => isNarrowing(filter.value))
const counts = computed(() => ({
  all: entries.value.length,
  done: entries.value.filter((entry) => entry.status === 'DONE').length,
  failed: entries.value.filter((entry) => entry.status === 'FAILED').length
}))
const firstLoad = computed(() => loading.value && !store.exportsLoaded)

async function load(): Promise<void> {
  loading.value = true
  loadFailure.value = null
  try {
    await store.loadExports()
  } catch (failure: unknown) {
    loadFailure.value = failure instanceof Error ? failure.message : String(failure)
  } finally {
    loading.value = false
  }
}

function remove(id: ProjectExportId): void {
  void run(() => store.removeExport(id), 'Export deleted.')
}

function askClearCompany(group: { companyId: ProjectExportClear['companyId']; companyName: string; count: number }): void {
  asking.value = {
    title: `Delete ${plural(group.count)}?`,
    body: `Every export of ${group.companyName}, with its file.`,
    confirmLabel: `Delete ${plural(group.count)}`,
    clear: { companyId: group.companyId }
  }
}

function askClearProject(group: ExportProjectGroup): void {
  asking.value = {
    title: `Delete ${plural(group.entries.length)}?`,
    body: `Every export of ${group.projectTitle}, with its file.`,
    confirmLabel: `Delete ${plural(group.entries.length)}`,
    clear: { projectId: group.projectId }
  }
}

function askClearFailed(): void {
  asking.value = {
    title: `Delete ${plural(counts.value.failed)}?`,
    body: 'Every export that failed. They have no file.',
    confirmLabel: `Delete ${plural(counts.value.failed)}`,
    clear: { status: 'FAILED' }
  }
}

async function confirmClear(): Promise<void> {
  const asked = asking.value
  asking.value = null
  if (asked) await run(() => store.clearExports(asked.clear), 'Exports deleted.')
}

function plural(count: number): string {
  return `${count} ${count === 1 ? 'export' : 'exports'}`
}

onMounted(load)
</script>

<template>
  <div class="relative flex min-h-0 flex-1 flex-col bg-canvas" data-testid="exports-page">
    <AppCatalogHeader title="Library">
      <template #actions>
        <AppButton size="sm" variant="secondary" :loading="loading" data-testid="exports-refresh" @click="load">Refresh</AppButton>
      </template>
    </AppCatalogHeader>
    <div class="page-column shrink-0 pt-4"><LibrarySubNav active="exports" :exports-count="store.exportsLoaded ? counts.all : undefined" /></div>

    <div class="min-h-0 flex-1 page-scroll">
      <div class="page-column flex flex-col gap-5 pb-16 pt-5">
        <ExportsToolbar v-if="counts.all > 0" v-model="filter" :counts="counts" :loaded-at="store.exportsLoadedAt" @clear-failed="askClearFailed" />

        <div v-if="loadFailure" class="flex items-start gap-3 rounded-control border border-danger/40 bg-danger-soft px-3 py-2" role="alert" data-testid="exports-load-failed">
          <p class="min-w-0 flex-1 text-small text-danger">
            <span class="font-semibold">The exports could not be loaded.</span>
            <span class="ml-1 font-mono text-caption">{{ loadFailure }}</span>
          </p>
          <AppButton size="sm" variant="secondary" @click="load">Try again</AppButton>
        </div>

        <div v-if="firstLoad" class="flex flex-col gap-3" role="status" aria-busy="true" data-testid="exports-loading">
          <span class="sr-only">Loading the exports</span>
          <span class="skeleton h-8 w-64 rounded-control" />
          <span v-for="index in 3" :key="index" class="skeleton h-12 rounded-control" />
        </div>

        <AppEmptyState
          v-else-if="store.exportsLoaded && counts.all === 0"
          class="mx-auto max-w-[540px]"
          title="Nothing exported yet"
          description="Open a blueprint, a page or a component and use Export. The PDF and Word files you make are kept here until you delete them."
        >
          <RouterLink to="/projects" class="focus-ring key-slate inline-flex h-8 items-center rounded-control px-3.5 text-body">Go to projects</RouterLink>
        </AppEmptyState>

        <template v-else-if="counts.all > 0">
          <p v-if="narrowing" class="text-caption text-text-subtle" data-testid="exports-showing">Showing {{ shown.length }} of {{ counts.all }}</p>
          <AppEmptyState v-if="shown.length === 0" class="mx-auto max-w-[540px]" title="No export matches" description="Nothing exported fits this search and these filters.">
            <AppButton size="sm" variant="secondary" data-testid="exports-clear-filters" @click="filter = NO_EXPORT_FILTER">Clear filters</AppButton>
          </AppEmptyState>
          <ExportCompanyGroup
            v-for="company in groups"
            :key="company.companyId"
            :group="company"
            :force-open="narrowing"
            @delete="remove"
            @clear-company="askClearCompany(company)"
            @clear-project="askClearProject"
          />
        </template>
      </div>
    </div>

    <AppConfirm
      v-if="asking"
      :title="asking.title"
      :body="asking.body"
      blast="The projects and their pages are not touched."
      :confirm-label="asking.confirmLabel"
      @cancel="asking = null"
      @confirm="confirmClear"
    />
  </div>
</template>
