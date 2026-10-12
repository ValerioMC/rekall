<script setup lang="ts">
import { computed } from 'vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import ProjectMark from '@/components/ui/ProjectMark.vue'
import ResourceRow from './ResourceRow.vue'
import { groupName, groupParent, type MemoryGroup, type ResourceSummary } from '@/model/claude-resources'
import type { Project } from '@/model/catalog'

const props = defineProps<{
  group: MemoryGroup
  /** The Rekall project whose folder this is, when there is one. */
  project: Project | null
  memories: readonly ResourceSummary[]
  expanded: boolean
  selectedPath: string | null
}>()

defineEmits<{ toggle: []; select: [path: string] }>()

const orphaned = computed(() => !props.group.global && !props.group.folderExists)
const bodyId = computed(() => `memory-group-${props.group.id}`)
</script>

<template>
  <section class="mb-1" :aria-label="groupName(group)" data-testid="memory-group">
    <button
      class="focus-ring flex w-full items-center gap-2 rounded-control px-2 py-1.5 text-left transition-colors hover:bg-surface-raised"
      :aria-expanded="expanded"
      :aria-controls="bodyId"
      @click="$emit('toggle')"
    >
      <svg
        class="size-3 shrink-0 text-text-subtle transition-transform"
        :class="expanded ? 'rotate-90' : ''"
        viewBox="0 0 12 12"
        fill="none"
        aria-hidden="true"
      >
        <path d="m4.5 2.5 3.5 3.5-3.5 3.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
      </svg>
      <ProjectMark v-if="project" :project-id="project.id" :icon="project.icon" :image-version="project.iconImageVersion" :size="20" />
      <span v-else class="grid size-5 shrink-0 place-items-center rounded-key bg-surface-raised text-text-subtle" aria-hidden="true">
        <svg v-if="group.global" class="size-3" viewBox="0 0 16 16" fill="none">
          <circle cx="8" cy="8" r="5.5" stroke="currentColor" stroke-width="1.3" />
          <path d="M2.5 8h11M8 2.5c2 2 2 9 0 11M8 2.5c-2 2-2 9 0 11" stroke="currentColor" stroke-width="1.1" />
        </svg>
        <svg v-else class="size-3" viewBox="0 0 16 16" fill="none">
          <path d="M2 4.5A1.5 1.5 0 0 1 3.5 3h2.8l1.4 1.5h4.8A1.5 1.5 0 0 1 14 6v5.5a1.5 1.5 0 0 1-1.5 1.5h-9A1.5 1.5 0 0 1 2 11.5z" stroke="currentColor" stroke-width="1.3" stroke-linejoin="round" />
        </svg>
      </span>
      <span class="min-w-0 flex-1">
        <span class="block truncate text-body font-semibold leading-[1.3] text-text">{{ project?.title ?? groupName(group) }}</span>
        <span class="block truncate font-mono text-micro leading-[1.4] text-text-subtle" :title="group.workingDirectory ?? undefined">
          {{ groupParent(group) }}
        </span>
      </span>
      <AppBadge v-if="orphaned" tone="warn" title="The folder these memories were written for is no longer on disk">gone</AppBadge>
      <span class="shrink-0 font-mono text-micro tabular-nums text-text-subtle">{{ memories.length }}</span>
    </button>

    <div v-show="expanded" :id="bodyId" class="ml-3 border-l border-border pl-1.5">
      <ResourceRow
        v-if="group.index"
        :summary="{ ...group.index, name: 'MEMORY.md', description: 'The index Claude loads first' }"
        :selected="selectedPath === group.index.path"
        @select="$emit('select', group.index.path)"
      />
      <ResourceRow
        v-for="memory in memories"
        :key="memory.path"
        :summary="memory"
        :selected="selectedPath === memory.path"
        @select="$emit('select', memory.path)"
      />
    </div>
  </section>
</template>
