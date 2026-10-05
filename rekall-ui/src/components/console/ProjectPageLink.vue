<script setup lang="ts">
import { computed } from 'vue'
import { identityHue } from '@/common/identity'
import type { ProjectId } from '@/model/branded'

/**
 * The end cap of a navigator project header: one click from the console to that project's page.
 * Always visible, so the way there never depends on finding a hover target.
 */
const props = defineProps<{
  projectId: ProjectId
  projectTitle: string
}>()

const hue = computed(() => identityHue(props.projectId))
const label = computed(() => `Open ${props.projectTitle}`)
</script>

<template>
  <RouterLink
    data-testid="open-project-page"
    class="project-page-link focus-ring grid w-9 shrink-0 place-items-center rounded-r-[var(--radius-card)] border-l border-border text-text-subtle transition-colors"
    :style="{ '--project-tint': hue.base, '--project-tint-soft': hue.soft }"
    :to="{ name: 'project-detail', params: { id: projectId } }"
    :aria-label="label"
    :title="label"
  >
    <svg class="size-3.5" viewBox="0 0 24 24" fill="none" aria-hidden="true">
      <path
        d="M14 4h6v6M20 4l-8.5 8.5M18 14v4.5a1.5 1.5 0 0 1-1.5 1.5h-11A1.5 1.5 0 0 1 4 18.5v-11A1.5 1.5 0 0 1 5.5 6H10"
        stroke="currentColor"
        stroke-width="1.8"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
    </svg>
  </RouterLink>
</template>

<style scoped>
.project-page-link:hover,
.project-page-link:focus-visible {
  color: var(--project-tint);
  background: var(--project-tint-soft);
}
</style>
