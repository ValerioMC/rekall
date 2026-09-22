<script setup lang="ts">
import { computed } from 'vue'
import { excerpt as previewOf } from '@/common/format/excerpt'
import { relativeTime } from '@/common/format/relative-time'
import { WRAPUP_AUTHOR_LABEL } from '@/model/catalog'
import type { Wrapup } from '@/model/catalog'

const props = defineProps<{
  wrapup: Wrapup | null
  selected: boolean
  behind: number
  missesSteps: number
}>()

const emit = defineEmits<{ open: [] }>()

const excerpt = computed(() => previewOf(props.wrapup?.bodyMarkdown ?? ''))

const writtenBy = computed(() =>
  props.wrapup ? WRAPUP_AUTHOR_LABEL[props.wrapup.writtenBy] : ''
)
</script>

<template>
  <button
    data-testid="wrapup-card"
    class="dossier-section"
    :class="selected && 'dossier-section-selected'"
    :aria-current="selected"
    @click="emit('open')"
  >
    <span
      class="dossier-node"
      :class="selected ? 'dossier-node-lit' : !wrapup && 'dossier-node-empty'"
      aria-hidden="true"
    >
      <svg class="size-[9px]" viewBox="0 0 12 12">
        <path
          d="M6 1.2 10.8 6 6 10.8 1.2 6z"
          :fill="wrapup ? 'currentColor' : 'none'"
          stroke="currentColor"
          stroke-width="1.4"
          stroke-linejoin="round"
        />
      </svg>
    </span>
    <span class="flex min-h-[17px] items-center gap-2">
      <span
        class="text-[11.5px] font-semibold tracking-[0.005em] transition-colors"
        :class="selected ? 'text-text' : 'text-text-muted'"
      >
        Wrapup
      </span>
      <span
        v-if="wrapup"
        class="ml-auto flex shrink-0 items-center gap-1 truncate text-[10.5px] text-text-muted"
        :title="`Written by ${writtenBy} on ${new Date(wrapup.updatedAt).toLocaleString()}`"
      >
        <svg v-if="wrapup.writtenBy === 'CLAUDE'" class="size-2.5 shrink-0" viewBox="0 0 12 12" aria-hidden="true">
          <path
            d="M6 0.5v3M6 8.5v3M0.5 6h3M8.5 6h3M2.3 2.3l2.1 2.1M7.6 7.6l2.1 2.1M9.7 2.3 7.6 4.4M4.4 7.6l-2.1 2.1"
            stroke="currentColor"
            stroke-width="1.1"
            stroke-linecap="round"
          />
        </svg>
        <svg v-else class="size-2.5 shrink-0" viewBox="0 0 12 12" aria-hidden="true">
          <path
            d="M2 10 3 6.6l4.4-4.4a1.2 1.2 0 0 1 1.7 1.7L4.7 8.3 2 10Z"
            fill="none"
            stroke="currentColor"
            stroke-width="1.1"
            stroke-linejoin="round"
          />
        </svg>
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

    <span
      v-if="wrapup && missesSteps > 0"
      class="mt-1.5 flex items-center gap-1.5 text-[10.5px] text-warn"
      data-testid="wrapup-misses-steps"
    >
      <span class="size-[5px] shrink-0 rounded-full bg-warn" aria-hidden="true" />
      {{ missesSteps }} step{{ missesSteps === 1 ? '' : 's' }} done since this was written
    </span>
    <span
      v-if="wrapup && behind > 0"
      class="mt-1.5 flex items-center gap-1.5 text-[10.5px] text-warn"
    >
      <span class="size-[5px] shrink-0 rounded-full bg-warn" aria-hidden="true" />
      {{ behind }} note{{ behind === 1 ? '' : 's' }} newer than this
    </span>
  </button>
</template>
