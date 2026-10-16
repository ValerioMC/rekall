<script setup lang="ts">
import { computed } from 'vue'
import { identityHue } from '@/common/identity'
import { describeTokens, formatTokens } from '@/common/format/token-count'
import type { TokenTotals } from '@/features/tokens/model/token-usage'

/**
 * The tokens generated for a step, a task or a project, with what was read to generate them in
 * the tooltip. `hueOf` tints the glyph with a project's identity colour.
 */
const props = withDefaults(defineProps<{ totals: TokenTotals; hueOf?: string | null }>(), { hueOf: null })

const label = computed(() => formatTokens(props.totals.generated))
const description = computed(() => describeTokens(props.totals))
const glyphColor = computed(() => (props.hueOf ? identityHue(props.hueOf).base : undefined))
</script>

<template>
  <span
    class="inline-flex shrink-0 items-center gap-[3px] font-mono text-micro tabular-nums"
    :class="totals.generated > 0 ? 'text-text-muted' : 'text-text-subtle'"
    :title="description"
    data-testid="token-count"
  >
    <svg class="size-2" viewBox="0 0 12 12" fill="none" aria-hidden="true" :style="{ color: glyphColor }">
      <path d="M6 1 10.5 3.5v5L6 11 1.5 8.5v-5z" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round" />
    </svg>
    {{ label }}
    <span class="sr-only">tokens generated</span>
  </span>
</template>
