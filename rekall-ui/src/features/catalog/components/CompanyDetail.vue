<script setup lang="ts">
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import CompanyMark from '@/components/ui/CompanyMark.vue'
import ProjectMark from '@/components/ui/ProjectMark.vue'
import ProjectCompositionBar from '@/features/catalog/components/ProjectCompositionBar.vue'
import { relativeTime } from '@/common/format/relative-time'
import { PROJECT_STATUS_LABEL } from '@/features/catalog/model/project-status'
import type { ProjectComposition } from '@/features/catalog/lib/project-composition'
import type { Company, Project } from '@/features/catalog/model/catalog'

/** One company beside the list: what it holds, where its work stands, and the actions on the record. */
defineProps<{
  company: Company
  projects: readonly Project[]
  composition: ProjectComposition
  connectionsLabel: string
  connectionsEmpty: boolean
}>()

const emit = defineEmits<{ edit: []; delete: []; newProject: []; openConnections: [] }>()
</script>

<template>
  <section class="flex flex-col gap-5 p-6" :aria-label="company.name" data-testid="company-detail">
    <header class="flex items-start gap-4">
      <CompanyMark :company-id="company.id" :name="company.name" :image-version="company.logoImageVersion" :size="48" />
      <div class="min-w-0 flex-1">
        <h2 class="truncate text-heading font-medium text-text">{{ company.name }}</h2>
        <p v-if="company.description" class="mt-1 text-body text-text-muted">{{ company.description }}</p>
        <p class="mt-1 text-caption text-text-subtle">Updated {{ relativeTime(company.updatedAt) }}</p>
      </div>
      <span class="flex shrink-0 gap-2">
        <AppButton variant="secondary" size="sm" data-testid="edit-company" @click="emit('edit')">Edit</AppButton>
        <AppButton variant="secondary" size="sm" data-testid="delete-company" @click="emit('delete')">Delete</AppButton>
      </span>
    </header>

    <ProjectCompositionBar :composition="composition" class="max-w-[360px]" />

    <div class="flex items-center gap-3 text-small">
      <span class="text-text-muted">Connections shared with this company</span>
      <button
        type="button"
        class="focus-ring rounded-key px-1.5 py-1 underline underline-offset-2 hover:text-text"
        :class="connectionsEmpty ? 'text-text-subtle' : 'text-accent'"
        data-testid="company-connections"
        @click="emit('openConnections')"
      >
        {{ connectionsLabel }}
      </button>
    </div>

    <div>
      <div class="mb-2 flex items-center gap-3">
        <h3 class="eyebrow text-caption">Projects</h3>
        <span class="font-mono text-caption text-text-subtle">{{ projects.length }}</span>
        <span class="flex-1" />
        <AppButton variant="secondary" size="sm" data-testid="new-company-project" @click="emit('newProject')">+ New project</AppButton>
      </div>
      <p v-if="!projects.length" class="rounded-card border border-dashed border-border px-4 py-6 text-center text-small text-text-subtle">
        No project yet. A project holds this company's work.
      </p>
      <div v-else class="overflow-hidden rounded-card border border-border bg-surface">
        <RouterLink
          v-for="(project, index) in projects"
          :key="project.id"
          :to="`/projects/${project.id}`"
          class="focus-ring flex items-center gap-3 px-4 py-2.5 transition-colors hover:bg-surface-raised"
          :class="index > 0 ? 'border-t border-border' : ''"
          data-testid="company-project"
        >
          <ProjectMark :project-id="project.id" :icon="project.icon" :image-version="project.iconImageVersion" :size="24" />
          <span class="min-w-0 flex-1 truncate text-body font-medium text-text">{{ project.title }}</span>
          <span class="anchor-chip shrink-0 px-1.5 text-micro">{{ project.anchor }}</span>
          <AppBadge :tone="project.status === 'ACTIVE' ? 'accent' : project.status === 'DONE' ? 'safe' : 'neutral'">
            {{ PROJECT_STATUS_LABEL[project.status] }}
          </AppBadge>
        </RouterLink>
      </div>
    </div>
  </section>
</template>
