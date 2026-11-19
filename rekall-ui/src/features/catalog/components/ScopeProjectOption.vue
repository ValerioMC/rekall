<script setup lang="ts">
import ProjectMark from '@/components/ui/ProjectMark.vue'
import { PROJECT_STATUS_LABEL } from '@/features/catalog/model/project-status'
import type { Project } from '@/features/catalog/model/catalog'

/** One project in the switcher: a row to pick it, a star to pin it, and on Work a key that opens its page instead. */
defineProps<{ project: Project; active: boolean; current: boolean; pinned: boolean; canOpenPage: boolean }>()

defineEmits<{ pick: []; 'toggle-pin': []; 'open-page': [] }>()
</script>

<template>
  <div
    class="group/option flex items-center gap-1 rounded-control"
    :class="active && 'bg-surface-hover'"
    :data-active="active"
    data-testid="scope-project-row"
  >
    <button
      type="button"
      role="option"
      class="focus-ring flex h-10 min-w-0 flex-1 cursor-pointer items-center gap-3 rounded-control px-3 text-left"
      :aria-selected="current"
      data-testid="scope-project"
      @click="$emit('pick')"
    >
      <ProjectMark :project-id="project.id" :icon="project.icon" :image-version="project.iconImageVersion" :size="20" />
      <span class="min-w-0 flex-1 truncate text-body" :class="current ? 'text-accent-strong' : 'text-text'">{{ project.title }}</span>
      <span v-if="project.status !== 'ACTIVE'" class="shrink-0 text-caption text-text-subtle">{{ PROJECT_STATUS_LABEL[project.status] }}</span>
      <span class="shrink-0 text-small tabular-nums text-text-subtle">{{ project.taskCount }}</span>
      <svg v-if="current" class="size-3.5 shrink-0 text-accent" viewBox="0 0 16 16" fill="none" aria-hidden="true">
        <path d="m3 8.5 3.2 3L13 4.5" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
      </svg>
    </button>
    <button
      type="button"
      class="focus-ring mr-1 grid size-6 shrink-0 cursor-pointer place-items-center rounded-key transition-opacity duration-(--motion-quick) hover:bg-surface hover:text-text focus-visible:opacity-100 group-hover/option:opacity-100"
      :class="pinned ? 'text-text-muted opacity-100' : 'text-text-subtle opacity-0'"
      :aria-label="`${pinned ? 'Unpin' : 'Pin'} ${project.title}`"
      :aria-pressed="pinned"
      data-testid="scope-pin"
      @click="$emit('toggle-pin')"
    >
      <svg class="size-3.5" viewBox="0 0 16 16" :fill="pinned ? 'currentColor' : 'none'" aria-hidden="true">
        <path d="m8 2 1.8 3.8 4.2.5-3.1 2.9.8 4.1L8 11.2 4.3 13.3l.8-4.1L2 6.3l4.2-.5L8 2Z" stroke="currentColor" stroke-width="1.4" stroke-linejoin="round" />
      </svg>
    </button>
    <button
      v-if="canOpenPage"
      type="button"
      class="focus-ring mr-1 hidden h-6 shrink-0 cursor-pointer items-center gap-1 rounded-key px-2 text-caption text-text-muted hover:bg-surface hover:text-text focus-visible:flex group-hover/option:flex"
      :aria-label="`Open the ${project.title} page`"
      data-testid="scope-open-page"
      @click="$emit('open-page')"
    >
      Open page
      <svg class="size-3" viewBox="0 0 16 16" fill="none" aria-hidden="true">
        <path d="M6 3.5h6.5V10M12.5 3.5 4 12" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" />
      </svg>
    </button>
  </div>
</template>
