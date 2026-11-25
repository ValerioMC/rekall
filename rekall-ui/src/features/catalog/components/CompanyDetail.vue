<script setup lang="ts">
import AppButton from '@/components/ui/AppButton.vue'
import CompanyMark from '@/components/ui/CompanyMark.vue'
import ProjectCompositionBar from '@/features/catalog/components/ProjectCompositionBar.vue'
import ProjectCompositionLegend from '@/features/catalog/components/ProjectCompositionLegend.vue'
import ProjectRow from '@/features/catalog/components/ProjectRow.vue'
import { relativeTime } from '@/common/format/relative-time'
import type { ProjectComposition } from '@/features/catalog/lib/project-composition'
import type { Company, Project } from '@/features/catalog/model/catalog'
import type { ProjectLedgerEntry } from '@/features/catalog/model/project-ledger-entry'

/** One company beside the list: where its work stands, its connections, its projects as ledger rows, and the actions on the record. */
defineProps<{
  company: Company
  entries: readonly ProjectLedgerEntry[]
  composition: ProjectComposition
  connectionsLabel: string
  connectionsEmpty: boolean
}>()

const emit = defineEmits<{
  edit: []
  delete: []
  newProject: []
  openConnections: []
  openProject: [project: Project]
}>()
</script>

<template>
  <section
    class="@container overflow-hidden rounded-card border border-border bg-surface"
    :aria-label="company.name"
    data-testid="company-detail"
  >
    <header class="flex items-start gap-4 px-6 pb-5 pt-6 @max-[52rem]:px-4 @max-[52rem]:pb-4 @max-[52rem]:pt-5">
      <CompanyMark :company-id="company.id" :name="company.name" :image-version="company.logoImageVersion" :size="48" />
      <div class="min-w-0 flex-1">
        <h2 class="truncate text-heading font-medium text-text">{{ company.name }}</h2>
        <p v-if="company.description" class="mt-1 max-w-[70ch] text-body text-text-muted">{{ company.description }}</p>
        <p class="mt-1.5 text-caption text-text-subtle @max-[52rem]:hidden">Updated {{ relativeTime(company.updatedAt) }}</p>
      </div>
      <span class="flex shrink-0 gap-1">
        <AppButton variant="quiet" size="sm" data-testid="edit-company" @click="emit('edit')">Edit</AppButton>
        <AppButton variant="danger-quiet" size="sm" data-testid="delete-company" @click="emit('delete')">Delete</AppButton>
      </span>
    </header>

    <div class="grid gap-2.5 px-6 pb-5 @max-[52rem]:px-4 @max-[52rem]:pb-4">
      <ProjectCompositionBar :composition="composition" />
      <ProjectCompositionLegend :composition="composition" />
    </div>

    <div
      class="mx-6 mb-6 flex flex-wrap items-center justify-between gap-3 rounded-control border border-border bg-canvas px-3.5 py-2.5 text-small text-text-muted @max-[52rem]:mx-4 @max-[52rem]:mb-5"
    >
      <span>Connections shared with this company</span>
      <button
        type="button"
        class="focus-ring inline-flex items-center gap-1 rounded-key px-1 hover:underline hover:underline-offset-3"
        :class="connectionsEmpty ? 'text-text-subtle' : 'font-medium text-text'"
        data-testid="company-connections"
        @click="emit('openConnections')"
      >
        {{ connectionsLabel }}
        <svg class="size-3" viewBox="0 0 24 24" fill="none" aria-hidden="true">
          <path d="M9 6l6 6-6 6" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
      </button>
    </div>

    <div class="flex h-11 items-center gap-2.5 border-t border-border bg-canvas/50 pl-6 pr-4 @max-[52rem]:px-4">
      <h3 class="text-body font-semibold text-text">Projects</h3>
      <span class="font-mono text-caption text-text-subtle">{{ entries.length }}</span>
      <AppButton class="ml-auto" variant="secondary" size="sm" data-testid="new-company-project" @click="emit('newProject')">
        + New project
      </AppButton>
    </div>
    <p v-if="!entries.length" class="border-t border-border px-6 py-7 text-center text-small text-text-subtle">
      No project yet. A project holds this company's work.
    </p>
    <ProjectRow
      v-for="entry in entries"
      :key="entry.project.id"
      :project="entry.project"
      :composition="entry.composition"
      :reviewing="entry.reviewing"
      :tokens="entry.tokens"
      :deletable="false"
      @open="emit('openProject', entry.project)"
    />
  </section>
</template>
