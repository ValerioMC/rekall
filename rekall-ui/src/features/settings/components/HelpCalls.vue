<script setup lang="ts">
import { computed } from 'vue'
import { HELP_TOOLS, type HelpToolName } from '@/features/settings/model/help-context'

const props = defineProps<{ names: readonly HelpToolName[] }>()

const tools = computed(() => props.names.map((name) => HELP_TOOLS[name]))
</script>

<template>
  <p v-if="tools.length" class="flex flex-wrap items-baseline gap-x-2 text-micro text-text-subtle" data-testid="help-calls">
    <span>calls</span>
    <span v-for="tool in tools" :key="tool.name" class="font-mono" :title="`${tool.access}: ${tool.effect}`">{{ tool.name }}</span>
  </p>
</template>
