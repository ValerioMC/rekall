<script setup lang="ts">
import { computed } from 'vue'
import { relativeTime } from '@/common/format/relative-time'
import { WRAPUP_AUTHOR_LABEL } from '@/model/catalog'
import type { Wrapup } from '@/model/catalog'

/**
 * The wrapup, pinned above the notes it summarises.
 *
 * It is not a note and must not read as the first one in the list. What separates it is form
 * rather than colour: a raised surface, a rule under it, a glyph nothing else uses. The palette
 * already spends amber on where you are and cyan on anchors, and a third hue for "this row is
 * special" is how a two-colour system becomes a five-colour one.
 *
 * The absent state is a card too, in outline. A task with no wrapup is a fact worth showing —
 * it is the reason to write one — and hiding the row would leave nothing to click.
 */
const props = defineProps<{
  wrapup: Wrapup | null
  selected: boolean
  /** Notes written since the wrapup was. Not proof it is wrong, only that it is older. */
  behind: number
}>()

const emit = defineEmits<{ open: [] }>()

/** A couple of lines of the body, with the markup stripped so the preview reads as prose. */
const excerpt = computed(() =>
  (props.wrapup?.bodyMarkdown ?? '')
    .replace(/[#`>*|_-]/g, ' ')
    .replace(/\s+/g, ' ')
    .trim()
    .slice(0, 110)
)

const writtenBy = computed(() =>
  props.wrapup ? WRAPUP_AUTHOR_LABEL[props.wrapup.writtenBy] : ''
)
</script>

<template>
  <div class="mb-2">
    <button
      data-testid="wrapup-card"
      class="focus-ring block w-full rounded-[var(--radius-control)] border p-2.5 text-left transition-all"
      :class="[
        wrapup
          ? 'border-border-strong bg-surface-raised hover:border-text-subtle'
          : 'border-dashed border-border-strong bg-transparent hover:border-accent hover:bg-accent-soft',
        selected && 'border-accent bg-surface-raised shadow-lift'
      ]"
      :aria-current="selected"
      @click="emit('open')"
    >
      <span class="flex items-center gap-2">
        <svg
          class="size-3 shrink-0 transition-colors"
          :class="selected ? 'text-accent' : wrapup ? 'text-text-muted' : 'text-text-subtle'"
          viewBox="0 0 12 12"
          aria-hidden="true"
        >
          <path
            d="M6 1.2 10.8 6 6 10.8 1.2 6z"
            :fill="wrapup ? 'currentColor' : 'none'"
            stroke="currentColor"
            stroke-width="1.4"
            stroke-linejoin="round"
          />
        </svg>
        <span class="text-[10px] font-semibold uppercase tracking-[0.09em] text-text-subtle">
          Wrapup
        </span>
        <span
          v-if="wrapup"
          class="ml-auto shrink-0 truncate text-[10.5px] text-text-muted"
          :title="`Written by ${writtenBy} on ${new Date(wrapup.updatedAt).toLocaleString()}`"
        >
          {{ writtenBy }} &middot; {{ relativeTime(wrapup.updatedAt) }}
        </span>
      </span>

      <span
        v-if="wrapup"
        class="mt-1 line-clamp-2 block text-[12px] leading-relaxed text-text-muted"
      >
        {{ excerpt }}
      </span>
      <span v-else class="mt-1 block text-[11.5px] leading-relaxed text-text-muted">
        No wrapup yet. Open it to write what this task currently is.
      </span>

      <!-- Said as a fact, not as an alarm: newer notes make a wrapup older, not wrong. -->
      <span
        v-if="wrapup && behind > 0"
        class="mt-1.5 flex items-center gap-1.5 text-[10.5px] text-warn"
      >
        <span class="size-[5px] shrink-0 rounded-full bg-warn" aria-hidden="true" />
        {{ behind }} note{{ behind === 1 ? '' : 's' }} newer than this
      </span>
    </button>

    <!-- The rule is what says the list below is a different kind of thing. -->
    <div class="mt-2 flex items-center gap-2 px-1">
      <span class="h-px flex-1 bg-border" aria-hidden="true" />
      <span class="text-[9.5px] font-semibold uppercase tracking-[0.09em] text-text-subtle">
        Notes
      </span>
      <span class="h-px flex-1 bg-border" aria-hidden="true" />
    </div>
  </div>
</template>
