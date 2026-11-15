<script setup lang="ts">
import { computed, ref } from 'vue'
import { useElementWidth } from '@/composables/useElementWidth'

/**
 * The frame of the pane beside the step list: a header with a lead mark, a title, keys and a line
 * of facts; an optional drawer under it; a body; and a dock for a decision still to be made. A
 * step and the overview fill the same slots, so the pane keeps its shape whether a step is
 * selected or not. Everything is as wide as the pane up to a page width. With the list hidden on
 * a wide pane the body gains a rail beside the main column, and the slots are told so (`railed`).
 */
const props = defineProps<{
  /** The header and dock stay and the body scrolls; unset, the pane grows with its text inside a page that scrolls. */
  pinned?: boolean
  /** The step list is hidden: on a wide pane the record of the step moves to a rail beside the detail. */
  focused?: boolean
}>()

const RAIL_FROM_PX = 1100

const root = ref<HTMLElement | null>(null)
const width = useElementWidth(root)
const railed = computed(() => props.focused === true && width.value >= RAIL_FROM_PX)
</script>

<template>
  <section ref="root" class="flex flex-col" :class="pinned && 'h-full min-h-0'">
    <header :class="pinned && 'shrink-0 border-b border-border'">
      <div class="mx-auto w-full max-w-[76rem]" :class="pinned ? 'pb-3 pl-7 pr-5 pt-3.5' : 'pb-4'">
        <div class="flex items-start gap-3">
          <span class="mt-0.5 size-6.5 shrink-0" aria-hidden="true"><slot name="lead" /></span>
          <h2 class="min-w-0 flex-1 text-heading font-medium leading-snug tracking-[-0.015em] text-text [text-wrap:balance]">
            <slot name="title" />
          </h2>
          <div class="flex shrink-0 items-center gap-1"><slot name="controls" /></div>
        </div>
        <p class="mt-2 flex flex-wrap items-center gap-x-4 gap-y-1 pl-9.5 text-small text-text-muted" data-testid="decision-meta">
          <slot name="facts" :railed="railed" />
        </p>
      </div>
    </header>

    <slot name="drawer" :railed="railed" />

    <div :class="pinned && 'min-h-0 flex-1 overflow-y-auto'">
      <div
        class="mx-auto w-full max-w-[76rem]"
        :class="[pinned ? 'px-7 pb-8 pt-5' : 'pt-1', railed && 'grid grid-cols-[minmax(0,1fr)_22rem] items-start gap-x-10']"
      >
        <div class="flex min-w-0 flex-col gap-6"><slot name="main" :railed="railed" /></div>
        <aside v-if="railed" class="flex min-h-48 min-w-0 flex-col gap-6 border-l border-border pl-8" aria-label="Record">
          <slot name="rail" />
        </aside>
      </div>
    </div>

    <footer v-if="$slots.dock" class="shrink-0 border-t border-border bg-surface" data-testid="decision-actions">
      <div class="mx-auto flex w-full max-w-[76rem] items-center justify-end gap-2" :class="pinned ? 'px-7 py-3' : 'pt-3'">
        <slot name="dock" />
      </div>
    </footer>
  </section>
</template>
