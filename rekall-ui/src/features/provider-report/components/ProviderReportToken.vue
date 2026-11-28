<script setup lang="ts">
import { APP_LOCALE } from '@/common/format/app-locale'

const props = defineProps<{ count: number }>()
const compact = new Intl.NumberFormat(APP_LOCALE, { maximumFractionDigits: 2 })
const exact = new Intl.NumberFormat(APP_LOCALE)

function format(count: number): string {
  if (count >= 1_000_000) return `${compact.format(count / 1_000_000)}M`
  if (count >= 1_000) return `${compact.format(count / 1_000)}k`
  return exact.format(count)
}
</script>

<template>
  <span class="whitespace-nowrap" :class="{ 'text-text-subtle': props.count === 0 }" :title="`${exact.format(props.count)} tokens`" :data-token-count="props.count">{{ format(props.count) }}</span>
</template>
