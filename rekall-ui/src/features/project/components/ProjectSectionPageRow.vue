<script setup lang="ts">
import { relativeTime } from '@/common/format/relative-time'
import { compactCharacters } from '@/features/project/lib/section-size'
import type { ProjectSection } from '@/features/project/model/project-section'

/** One page in the rail: its title, how big it is and when it changed. Draggable, so a person can move it into another context. */
defineProps<{ section: ProjectSection; selected: boolean; dragged: boolean }>()

const emit = defineEmits<{ select: []; dragStart: [event: DragEvent]; dragEnd: [] }>()
</script>

<template>
  <button
    type="button"
    draggable="true"
    class="focus-ring group relative flex w-full cursor-pointer items-center rounded-control py-1.5 pl-2.5 pr-7 text-left transition-colors duration-(--motion-quick)"
    :class="[selected ? 'selected-row' : 'hover:bg-surface-raised', dragged && 'opacity-40']"
    :aria-current="selected ? 'page' : undefined"
    :data-testid="`rail-section-${section.id}`"
    @click="emit('select')"
    @dragstart="emit('dragStart', $event)"
    @dragend="emit('dragEnd')"
  >
    <span class="min-w-0 flex-1">
      <span class="block truncate text-body text-text">{{ section.title }}</span>
      <span class="block truncate text-caption text-text-subtle">
        {{ section.bodyMarkdown.trim() === '' ? 'Empty' : `${compactCharacters(section.bodyMarkdown.length)} · Edited ${relativeTime(section.updatedAt)}` }}
      </span>
    </span>
    <svg
      class="pointer-events-none absolute right-2 top-1/2 size-3 -translate-y-1/2 text-text-subtle opacity-0 transition-opacity duration-(--motion-quick) group-hover:opacity-100"
      viewBox="0 0 12 14"
      fill="currentColor"
      aria-hidden="true"
    >
      <circle cx="4" cy="3" r="1.1" /><circle cx="8" cy="3" r="1.1" /><circle cx="4" cy="7" r="1.1" /><circle cx="8" cy="7" r="1.1" /><circle cx="4" cy="11" r="1.1" /><circle cx="8" cy="11" r="1.1" />
    </svg>
  </button>
</template>
