<script setup lang="ts">
import { computed } from 'vue'
import { mcpArguments } from '@/features/chat/lib/mcp-arguments'

const props = defineProps<{ text: string }>()

const lines = computed(() => mcpArguments(props.text))
</script>

<template>
  <dl
    v-if="lines.length > 0"
    class="grid grid-cols-[max-content_minmax(0,1fr)] gap-x-3 gap-y-0.5 rounded-control border border-border bg-(--color-well-top) px-2 py-1.5 font-mono text-small"
    data-testid="tool-arguments"
  >
    <template v-for="line in lines" :key="line.key">
      <dt v-if="line.key" class="text-text-subtle">{{ line.key }}</dt>
      <dd class="min-w-0 break-words whitespace-pre-wrap text-text-muted" :class="!line.key && 'col-span-2'">{{ line.value }}</dd>
    </template>
  </dl>
  <p v-else class="text-caption text-text-subtle">No arguments</p>
</template>
