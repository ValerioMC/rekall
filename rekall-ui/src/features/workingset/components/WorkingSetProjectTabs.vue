<script setup lang="ts">
import { computed } from 'vue'
import ProjectMark from '@/components/ui/ProjectMark.vue'
import type { CompanyGroup, ProjectGroup } from '@/features/workingset/model/working-set'
import type { ProjectId, TaskId } from '@/model/branded'

/**
 * Level 1 of the working-set bar: one tab per project with an open task, grouped under its
 * company. A tab counts the project's open tasks and, in amber, the ones waiting for review.
 */
const props = defineProps<{
  groups: readonly CompanyGroup[]
  currentProjectId: ProjectId | null
  reviewCounts: ReadonlyMap<TaskId, number>
}>()

const emit = defineEmits<{ select: [projectId: ProjectId] }>()

const projectOrder = computed(() => props.groups.flatMap((company) => company.projects.map((group) => group.project.id)))

function keyHintOf(group: ProjectGroup): string {
  const key = projectOrder.value.indexOf(group.project.id) + 1
  return key >= 1 && key <= 9 ? `${group.project.title} (key ${key})` : group.project.title
}

function reviewCountOf(group: ProjectGroup): number {
  return group.tasks.reduce((sum, task) => sum + (props.reviewCounts.get(task.id) ?? 0), 0)
}

function nameOf(group: ProjectGroup): string {
  const open = group.tasks.length
  const review = reviewCountOf(group)
  return `${group.project.title}, ${open} open${review > 0 ? `, ${review} to review` : ''}`
}
</script>

<template>
  <div
    class="flex h-10 shrink-0 items-center gap-1 overflow-x-auto border-b border-border bg-surface px-3"
    role="tablist"
    aria-label="Open projects"
    data-testid="working-set-projects"
  >
    <template v-for="company in groups" :key="company.company.id">
      <span class="whitespace-nowrap pl-2.5 pr-2 text-caption text-text-subtle first:pl-0" data-testid="working-set-company">
        {{ company.company.name }}
      </span>
      <button
        v-for="group in company.projects"
        :key="group.project.id"
        type="button"
        role="tab"
        class="focus-ring relative inline-flex h-8 shrink-0 cursor-pointer items-center gap-2 whitespace-nowrap rounded-control pl-[7px] pr-2.5 font-medium transition-colors duration-(--motion-quick) hover:bg-surface-raised hover:text-text"
        :class="
          currentProjectId === group.project.id
            ? 'bg-canvas text-text shadow-[inset_0_0_0_1px_var(--color-border-strong)] after:absolute after:inset-x-2 after:-bottom-[5px] after:h-0.5 after:rounded-full after:bg-accent'
            : 'text-text-muted'
        "
        :aria-selected="currentProjectId === group.project.id"
        :aria-label="nameOf(group)"
        :title="keyHintOf(group)"
        data-testid="working-set-project"
        @click="emit('select', group.project.id)"
      >
        <ProjectMark
          :project-id="group.project.id"
          :icon="group.project.icon"
          :image-version="group.project.iconImageVersion"
          :size="20"
        />
        <span class="max-w-40 truncate">{{ group.project.title }}</span>
        <span class="text-caption tabular-nums text-text-subtle" data-testid="working-set-open-count">{{ group.tasks.length }}</span>
        <span
          v-if="reviewCountOf(group) > 0"
          class="rounded-full bg-accent px-1.5 text-caption font-medium leading-4 text-accent-ink"
          data-testid="working-set-review-count"
        >
          {{ reviewCountOf(group) }}
        </span>
      </button>
    </template>
    <span
      class="ml-auto shrink-0 whitespace-nowrap pl-3 text-caption text-text-subtle"
      title="1-9 open a project, [ and ] step through the open tasks, ` goes back to the previous place"
      data-testid="working-set-keys"
    >
      <span class="font-mono">[ ]</span> step · <span class="font-mono">`</span> back
    </span>
  </div>
</template>
