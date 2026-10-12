<script setup lang="ts">
import { computed } from 'vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import ProjectMark from '@/components/ui/ProjectMark.vue'
import ResourceRow from './ResourceRow.vue'
import { groupName, groupParent } from '@/model/claude-resources'
import { memoriesBelow, relativeDirectory, type MemoryNode } from '@/model/memory-tree'

/** One project's memories; the folders inside it are the default slot, nested under its own. */
const props = defineProps<{
  node: MemoryNode
  expanded: boolean
  selectedPath: string | null
}>()

defineEmits<{ toggle: []; select: [path: string] }>()

const group = computed(() => props.node.group)
const orphaned = computed(() => !group.value.global && !group.value.folderExists)
const bodyId = computed(() => `memory-group-${group.value.id}`)
const nested = computed(() => props.node.parentDirectory !== null)
const subtitle = computed(() => relativeDirectory(props.node) ?? groupParent(group.value))
const total = computed(() => memoriesBelow(props.node))
const insideCount = computed(() => total.value - props.node.memories.length)
</script>

<template>
  <section class="mb-1" :aria-label="groupName(group)" data-testid="memory-group">
    <button
      class="focus-ring flex w-full items-center gap-2 rounded-control px-2 py-1.5 text-left transition-colors hover:bg-surface-raised"
      :aria-expanded="expanded"
      :aria-controls="bodyId"
      @click="$emit('toggle')"
    >
      <svg class="size-3 shrink-0 text-text-subtle transition-transform motion-reduce:transition-none" :class="expanded ? 'rotate-90' : ''" viewBox="0 0 12 12" fill="none" aria-hidden="true">
        <path d="m4.5 2.5 3.5 3.5-3.5 3.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
      </svg>
      <ProjectMark v-if="node.project" :project-id="node.project.id" :icon="node.project.icon" :image-version="node.project.iconImageVersion" :size="nested ? 16 : 20" />
      <span v-else class="grid shrink-0 place-items-center rounded-key bg-surface-raised text-text-subtle" :class="nested ? 'size-4' : 'size-5'" aria-hidden="true">
        <svg v-if="group.global" class="size-3" viewBox="0 0 16 16" fill="none">
          <circle cx="8" cy="8" r="5.5" stroke="currentColor" stroke-width="1.3" />
          <path d="M2.5 8h11M8 2.5c2 2 2 9 0 11M8 2.5c-2 2-2 9 0 11" stroke="currentColor" stroke-width="1.1" />
        </svg>
        <svg v-else class="size-3" viewBox="0 0 16 16" fill="none">
          <path d="M2 4.5A1.5 1.5 0 0 1 3.5 3h2.8l1.4 1.5h4.8A1.5 1.5 0 0 1 14 6v5.5a1.5 1.5 0 0 1-1.5 1.5h-9A1.5 1.5 0 0 1 2 11.5z" stroke="currentColor" stroke-width="1.3" stroke-linejoin="round" />
        </svg>
      </span>
      <span class="min-w-0 flex-1">
        <span class="block truncate font-semibold leading-[1.3] text-text" :class="nested ? 'text-small' : 'text-body'">{{ node.project?.title ?? groupName(group) }}</span>
        <span class="block truncate font-mono text-micro leading-[1.4] text-text-subtle" :title="group.workingDirectory ?? undefined">{{ subtitle }}</span>
      </span>
      <AppBadge v-if="orphaned" tone="warn" title="The folder these memories were written for is no longer on disk">gone</AppBadge>
      <span
        class="shrink-0 font-mono text-micro tabular-nums text-text-subtle"
        :title="insideCount > 0 ? `${node.memories.length} here, ${insideCount} in the folders inside` : undefined"
      >
        {{ node.memories.length }}<template v-if="insideCount > 0"> +{{ insideCount }}</template>
      </span>
    </button>

    <div v-show="expanded" :id="bodyId" class="ml-3 border-l border-border pl-1.5">
      <ResourceRow
        v-if="group.index"
        :summary="{ ...group.index, name: 'MEMORY.md', description: 'The index Claude loads first' }"
        :selected="selectedPath === group.index.path"
        @select="$emit('select', group.index.path)"
      />
      <ResourceRow v-for="memory in node.memories" :key="memory.path" :summary="memory" :selected="selectedPath === memory.path" @select="$emit('select', memory.path)" />
      <slot />
    </div>
  </section>
</template>
