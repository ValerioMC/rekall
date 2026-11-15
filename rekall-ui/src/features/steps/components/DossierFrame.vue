<script setup lang="ts">
/**
 * The frame of the pane beside the step list: a header with a lead mark, a title, keys and a line
 * of facts, a body of a reading column and a column of evidence, and a dock for the decision. A
 * step and the overview fill the same slots, so the pane keeps its shape whether a step is
 * selected or not. Pinned, the header and dock stay while the body scrolls; narrow, the columns
 * stack. The reading column holds a writing measure and the evidence column sits beside it, so on
 * a wide pane the pair stays together under the title and no width is left empty between them.
 */
defineProps<{
  /** The header and dock stay and the body scrolls; unset, the pane grows with its text inside a page that scrolls. */
  pinned?: boolean
}>()
</script>

<template>
  <section class="@container flex flex-col" :class="pinned && 'h-full min-h-0'">
    <header :class="pinned ? 'shrink-0 border-b border-border pb-3 pl-8 pr-5 pt-4' : 'pb-4'">
      <div class="flex items-start gap-3">
        <span class="mt-0.5 size-6.5 shrink-0" aria-hidden="true"><slot name="lead" /></span>
        <h2 class="min-w-0 flex-1 text-heading font-medium leading-snug tracking-[-0.015em] text-text [text-wrap:balance]">
          <slot name="title" />
        </h2>
        <div class="flex shrink-0 items-center gap-1"><slot name="controls" /></div>
      </div>
      <p class="mt-2 flex flex-wrap items-center gap-x-4 gap-y-1 pl-9.5 text-small text-text-muted" data-testid="decision-meta">
        <slot name="facts" />
      </p>
    </header>

    <div :class="pinned && 'min-h-0 flex-1 overflow-y-auto px-8 pb-8 pt-6'">
      <div class="flex flex-col gap-7 @min-[52rem]:flex-row @min-[52rem]:gap-x-9">
        <div class="min-w-0 flex-1 @min-[52rem]:max-w-[46rem]"><slot name="main" /></div>
        <div
          class="flex min-w-0 flex-col gap-6 border-t border-border pt-6 @min-[52rem]:w-[clamp(18.75rem,30%,32rem)] @min-[52rem]:shrink-0 @min-[52rem]:border-l @min-[52rem]:border-t-0 @min-[52rem]:pl-8 @min-[52rem]:pt-0"
        >
          <slot name="side" />
        </div>
      </div>
    </div>

    <footer
      v-if="$slots.dock"
      class="flex shrink-0 items-center gap-4 border-t border-border bg-surface"
      :class="pinned ? 'px-8 py-3' : 'mt-2 pt-3'"
      data-testid="decision-actions"
    >
      <slot name="dock" />
    </footer>
  </section>
</template>
