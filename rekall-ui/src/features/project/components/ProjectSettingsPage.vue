<script setup lang="ts">
import ProjectAboutGroup from '@/features/project/components/ProjectAboutGroup.vue'
import ProjectRepositoryGroup from '@/features/project/components/ProjectRepositoryGroup.vue'
import type { Project } from '@/features/catalog/model/catalog'

/** What the project says about itself and where its sessions run, one column to read down. */
defineProps<{ project: Project }>()
</script>

<template>
  <section
    class="h-full min-h-0 overflow-y-auto rounded-card border border-border bg-surface shadow-[var(--shadow-raised)]"
    aria-label="Project settings"
    data-testid="project-settings"
  >
    <div class="mx-auto flex w-full max-w-[680px] flex-col gap-8 px-6 py-6">
      <ProjectAboutGroup :project="project" />
      <ProjectRepositoryGroup :project="project" />
    </div>
  </section>
</template>
