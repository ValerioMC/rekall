<script setup lang="ts">
import ProjectMark from '@/components/ui/ProjectMark.vue'
import StatusPill from '@/components/ui/StatusPill.vue'
import { highlightSegments } from '@/features/search/lib/highlight'
import { optionId } from '@/features/search/lib/palette-entries'
import type { PaletteEntry } from '@/features/search/model/palette'

/** One result: what it is, where it lives (project, anchor) and its state. A text hit adds the words around the match. */
const props = defineProps<{ entry: PaletteEntry; terms: readonly string[]; active: boolean }>()
const emit = defineEmits<{ pick: [entry: PaletteEntry]; hover: [entry: PaletteEntry] }>()

const isText = props.entry.group === 'In the text'
</script>

<template>
  <div
    :id="optionId(entry)"
    role="option"
    :aria-selected="active"
    class="flex cursor-pointer gap-3 px-4 py-2 transition-colors"
    :class="[active ? 'bg-accent-soft' : 'hover:bg-surface-raised', isText ? 'items-start' : 'items-center']"
    data-testid="palette-entry"
    @mousedown.prevent="emit('pick', entry)"
    @mousemove="emit('hover', entry)"
  >
    <span v-if="entry.mark.kind === 'text'" class="mt-px shrink-0 rounded-key border border-border-strong px-1.5 text-micro font-semibold uppercase leading-4 tracking-[0.04em] text-text-subtle">
      {{ entry.mark.label }}
    </span>
    <span v-else class="grid size-5 shrink-0 place-items-center" aria-hidden="true">
      <ProjectMark
        v-if="entry.mark.kind === 'project'"
        :project-id="entry.mark.projectId"
        :icon="entry.mark.icon"
        :image-version="entry.mark.imageVersion"
        :size="20"
      />
      <span v-else-if="entry.mark.kind === 'task'" class="size-2 rounded-full bg-border-strong" />
      <span v-else-if="entry.mark.kind === 'note'" class="font-mono text-caption text-text-subtle">#</span>
      <span v-else-if="entry.mark.kind === 'screen'" class="text-small text-text-subtle">&#8250;</span>
    </span>

    <span class="flex min-w-0 flex-1 flex-col">
      <span class="flex min-w-0 items-baseline gap-2.5">
        <span class="truncate text-small font-medium text-text" data-testid="palette-title">
          <template v-for="(part, index) in highlightSegments(entry.title, terms)" :key="index">
            <mark v-if="part.matched" class="rounded-key bg-text/10 px-px font-semibold text-text">{{ part.text }}</mark>
            <template v-else>{{ part.text }}</template>
          </template>
        </span>
        <span v-if="entry.subtitle" class="max-w-[45%] shrink-0 truncate text-small text-text-subtle">{{ entry.subtitle }}</span>
      </span>
      <span v-if="entry.excerpt" class="line-clamp-2 text-small text-text-muted" data-testid="palette-excerpt">
        <template v-for="(part, index) in highlightSegments(entry.excerpt, terms)" :key="index">
          <mark v-if="part.matched" class="rounded-key bg-text/10 px-px font-semibold text-text">{{ part.text }}</mark>
          <template v-else>{{ part.text }}</template>
        </template>
      </span>
    </span>

    <span class="flex shrink-0 items-center gap-2.5" :class="isText && 'mt-px'">
      <span v-if="entry.owner" class="flex items-center gap-1.5 text-small text-text-subtle" data-testid="palette-owner">
        <ProjectMark :project-id="entry.owner.projectId" :icon="entry.owner.icon" :image-version="entry.owner.imageVersion" :size="14" />
        {{ entry.owner.label }}
      </span>
      <span v-if="entry.anchor" class="font-mono text-caption text-anchor" data-testid="palette-anchor">{{ entry.anchor }}</span>
      <StatusPill v-if="entry.status" :tone="entry.status.tone">{{ entry.status.label }}</StatusPill>
    </span>
  </div>
</template>
