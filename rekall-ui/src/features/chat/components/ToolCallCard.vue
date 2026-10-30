<script setup lang="ts">
import { computed, ref, useId } from 'vue'
import ToolGlyph from './ToolGlyph.vue'
import ToolInputView from './ToolInputView.vue'
import ToolOutput from './ToolOutput.vue'
import ToolStateGlyph from './ToolStateGlyph.vue'
import { describeTool } from '@/features/chat/lib/describe-tool'
import type { ToolItem } from '@/features/chat/model/timeline'

/**
 * One tool call as a single row, opened to its input and output. An edit shows as the lines it
 * changes, a command as the command and the end of its output.
 */
const props = defineProps<{ call: ToolItem }>()

const open = ref(false)
const bodyId = `tool-${useId()}`

const description = computed(() => describeTool(props.call.input))
const showsOutput = computed(() => {
  const { input, result } = props.call
  if (!result) return false
  return (input.tool !== 'edit' && input.tool !== 'write') || result.isError
})
</script>

<template>
  <div class="rounded-card border border-border bg-surface" data-testid="tool-call-card" :data-state="call.state">
    <button
      type="button"
      class="focus-ring flex w-full cursor-pointer items-center gap-2 rounded-card px-3 py-2 text-left transition-colors duration-(--motion-quick) hover:bg-surface-raised"
      :aria-expanded="open"
      :aria-controls="bodyId"
      @click="open = !open"
    >
      <ToolGlyph class="text-text-muted" :icon="description.icon" />
      <span class="shrink-0 text-small font-semibold text-text">{{ description.name }}</span>
      <span class="min-w-0 flex-1 truncate font-mono text-small text-text-muted">{{ description.target }}</span>
      <ToolStateGlyph :state="call.state" />
    </button>

    <div v-if="open" :id="bodyId" class="flex flex-col gap-2 border-t border-border px-3 py-2" data-testid="tool-call-body">
      <ToolInputView :input="call.input" />

      <ToolOutput
        v-if="showsOutput && call.result"
        :text="call.result.excerpt"
        :truncated="call.result.truncated"
        :is-error="call.result.isError"
      />
    </div>
  </div>
</template>
