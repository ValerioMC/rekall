<script setup lang="ts">
import { ref } from 'vue'
import KindGlyph from '@/components/diagram/canvas/KindGlyph.vue'
import type { Diagram, DiagramSummary } from '@/model/diagram'
import type { DiagramId } from '@/model/branded'

/**
 * What is open and how to leave it: back to the gallery, a switch to another diagram of the
 * same project, the question it answers and where it belongs.
 */
defineProps<{ diagram: Diagram; siblings: readonly DiagramSummary[]; projectAnchor: string | null; taskAnchor: string | null }>()

const emit = defineEmits<{ back: []; open: [id: DiagramId]; remove: [] }>()

const switching = ref(false)

function pick(id: DiagramId): void {
  switching.value = false
  emit('open', id)
}

function onKeydown(event: KeyboardEvent): void {
  if (event.key !== 'Escape' || !switching.value) return
  event.stopPropagation()
  switching.value = false
}
</script>

<template>
  <div class="glass pointer-events-auto relative min-w-0 max-w-[520px] rounded-card border border-border px-3 py-2.5 shadow-lift" data-testid="diagram-title-card" @keydown="onKeydown">
    <div class="flex min-w-0 items-center gap-1.5">
      <button
        class="focus-ring grid size-7 shrink-0 place-items-center rounded-control text-text-muted transition-colors hover:bg-surface-raised hover:text-text"
        aria-label="Back to all diagrams"
        title="All diagrams (Backspace)"
        data-testid="diagram-back"
        @click="emit('back')"
      >
        <svg viewBox="0 0 16 16" class="size-4" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M10 3.5 5.5 8l4.5 4.5" /></svg>
      </button>
      <button
        class="focus-ring flex min-w-0 items-center gap-1.5 rounded-control px-1.5 py-0.5 text-left transition-colors hover:bg-surface-raised disabled:cursor-default disabled:hover:bg-transparent"
        :disabled="siblings.length === 0"
        aria-haspopup="listbox"
        :aria-expanded="switching"
        data-testid="diagram-switcher"
        @click="switching = !switching"
      >
        <h2 class="truncate text-title font-semibold tracking-[-0.01em] text-text">{{ diagram.title }}</h2>
        <svg v-if="siblings.length > 0" viewBox="0 0 12 12" class="size-3 shrink-0 text-text-subtle" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="m3 4.5 3 3 3-3" /></svg>
      </button>
      <button
        class="focus-ring ml-auto grid size-7 shrink-0 place-items-center rounded-control text-text-subtle transition-colors hover:bg-danger-soft hover:text-danger"
        aria-label="Delete this diagram"
        title="Delete this diagram"
        data-testid="diagram-delete"
        @click="emit('remove')"
      >
        <svg viewBox="0 0 16 16" class="size-3.5" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M3 4.5h10M6.5 4.5V3h3v1.5M4.5 4.5l.6 8.5h5.8l.6-8.5" /></svg>
      </button>
    </div>
    <p v-if="diagram.question" class="mt-1 line-clamp-2 pl-[38px] text-small leading-snug text-text-muted" :title="diagram.question">{{ diagram.question }}</p>
    <div v-if="projectAnchor || taskAnchor" class="mt-1.5 flex flex-wrap items-center gap-1.5 pl-[38px] text-caption">
      <span v-if="projectAnchor" class="anchor-chip px-1.5 py-0.5">{{ projectAnchor }}</span>
      <span v-if="taskAnchor" class="anchor-chip px-1.5 py-0.5">{{ taskAnchor }}</span>
    </div>

    <template v-if="switching">
      <ul
        class="rise absolute left-0 top-[calc(100%+6px)] z-(--z-overlay) max-h-[320px] w-[340px] overflow-y-auto rounded-card border border-border-strong bg-surface-raised p-1.5 shadow-modal"
        role="listbox"
        aria-label="Other diagrams of this project"
        data-testid="diagram-switcher-list"
      >
        <li v-for="sibling in siblings" :key="sibling.id" role="option" :aria-selected="false">
          <button class="focus-ring flex w-full min-w-0 items-center gap-2.5 rounded-control px-2 py-2 text-left transition-colors hover:bg-surface" @click="pick(sibling.id)">
            <span class="flex shrink-0 gap-0.5 text-text-subtle">
              <KindGlyph v-for="kind in sibling.kinds.slice(0, 3)" :key="kind" :kind="kind" :size="14" />
            </span>
            <span class="min-w-0 flex-1 truncate text-small text-text">{{ sibling.title }}</span>
          </button>
        </li>
      </ul>
      <button class="fixed inset-0 z-(--z-sticky) cursor-default" tabindex="-1" aria-hidden="true" @click="switching = false" />
    </template>
  </div>
</template>
