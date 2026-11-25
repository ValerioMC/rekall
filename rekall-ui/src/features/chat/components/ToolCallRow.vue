<script setup lang="ts">
import { computed } from 'vue'
import ToolGlyph from './ToolGlyph.vue'
import ToolStateGlyph from './ToolStateGlyph.vue'
import { describeTool } from '@/features/chat/lib/describe-tool'
import type { ToolItem } from '@/features/chat/model/timeline'

/** The one-line face of a tool call: what it is, what it was pointed at, and what it did. */
const props = defineProps<{ call: ToolItem; open: boolean; controls: string }>()

defineEmits<{ toggle: [] }>()

const description = computed(() => describeTool(props.call.input))
const summary = computed(() => props.call.result?.summary ?? null)
</script>

<template>
  <button
    type="button"
    class="focus-ring flex w-full cursor-pointer items-center gap-2 rounded-card px-3 py-2 text-left transition-colors duration-(--motion-quick) hover:bg-surface-raised"
    :aria-expanded="open"
    :aria-controls="controls"
    @click="$emit('toggle')"
  >
    <ToolGlyph class="text-text-muted" :icon="description.icon" />
    <span class="shrink-0 text-small font-semibold text-text">{{ description.name }}</span>
    <span class="min-w-0 flex-1 truncate font-mono text-small text-text-muted">{{ description.target }}</span>
    <span
      v-if="summary"
      class="max-w-[40%] shrink-0 truncate text-caption"
      :class="call.state === 'failed' ? 'text-danger' : 'text-text-subtle'"
      data-testid="tool-summary"
      >{{ summary }}</span
    >
    <ToolStateGlyph :state="call.state" />
  </button>
</template>
