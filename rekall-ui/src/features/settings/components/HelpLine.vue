<script setup lang="ts">
import { computed } from 'vue'
import { helpSegments, type HelpValues } from '@/features/settings/lib/help-template'

const props = defineProps<{ template: string; values: HelpValues }>()

const segments = computed(() => helpSegments(props.template, props.values))
const isRekallCommand = computed(() => props.template.startsWith('/rk'))
</script>

<template>
  <code class="block break-words font-mono text-small" :class="isRekallCommand ? 'text-anchor' : 'text-text'">
    <template v-for="(segment, index) in segments" :key="index">
      <span
        v-if="segment.replaceable"
        class="underline decoration-dashed decoration-1 underline-offset-4"
        title="Replace with your own"
        data-testid="help-placeholder"
      >{{ segment.text }}</span>
      <template v-else>{{ segment.text }}</template>
    </template>
  </code>
</template>
