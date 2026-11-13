<script setup lang="ts">
import ProjectMark from '@/components/ui/ProjectMark.vue'
import { optionId } from '@/features/search/lib/palette-entries'
import type { PaletteEntry, PaletteSection } from '@/features/search/model/palette'

defineProps<{
  sections: readonly PaletteSection[]
  activeKey: string | null
}>()
const emit = defineEmits<{ pick: [entry: PaletteEntry]; hover: [entry: PaletteEntry] }>()

</script>

<template>
  <div id="palette-list" role="listbox" aria-label="Results" class="max-h-[min(440px,60vh)] overflow-y-auto py-1">
    <div v-for="section in sections" :key="section.group" role="group" :aria-label="section.group">
      <p class="px-4 pb-1 pt-2.5 text-micro font-semibold uppercase tracking-[0.06em] text-text-subtle" aria-hidden="true">
        {{ section.group }}
      </p>
      <div
        v-for="entry in section.entries"
        :id="optionId(entry)"
        :key="entry.key"
        role="option"
        :aria-selected="entry.key === activeKey"
        class="flex cursor-pointer items-center gap-3 px-4 py-2 transition-colors"
        :class="entry.key === activeKey ? 'bg-accent-soft' : 'hover:bg-surface-raised'"
        data-testid="palette-entry"
        @mousedown.prevent="emit('pick', entry)"
        @mousemove="emit('hover', entry)"
      >
        <span class="grid size-5 shrink-0 place-items-center" aria-hidden="true">
          <ProjectMark
            v-if="entry.mark.kind === 'project'"
            :project-id="entry.mark.projectId"
            :icon="entry.mark.icon"
            :image-version="entry.mark.imageVersion"
            :size="20"
          />
          <span
            v-else-if="entry.mark.kind === 'task'"
            class="size-2 rounded-full"
            :class="entry.mark.needsReview ? 'bg-accent' : 'bg-border-strong'"
          />
          <span v-else-if="entry.mark.kind === 'note'" class="font-mono text-caption text-text-subtle">#</span>
          <span v-else-if="entry.mark.kind === 'screen'" class="text-small text-text-subtle">&#8250;</span>
        </span>
        <span
          v-if="entry.mark.kind === 'text'"
          class="shrink-0 rounded-full border border-border px-1.5 text-micro font-semibold uppercase tracking-[0.04em] text-text-subtle"
        >
          {{ entry.mark.label }}
        </span>
        <span class="min-w-0 truncate text-small font-medium text-text">{{ entry.title }}</span>
        <span
          class="ml-auto min-w-0 max-w-[55%] shrink truncate text-caption"
          :class="entry.group === 'In the text' ? 'text-text-muted' : 'font-mono text-anchor'"
        >
          {{ entry.detail }}
        </span>
      </div>
    </div>
  </div>
</template>
