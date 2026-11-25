<script setup lang="ts">
import { computed, ref, useId, watch } from 'vue'
import ToolCallRow from './ToolCallRow.vue'
import ToolInputView from './ToolInputView.vue'
import ToolOutput from './ToolOutput.vue'
import { foldsOnResult, opensByDefault, showsOutput } from '@/features/chat/lib/tool-card'
import type { ToolItem } from '@/features/chat/model/timeline'

/**
 * One tool call: a row that says what it was and what it did, opened to its input and output.
 * A todo list starts open, being the progress of the work; an Agent is open while it runs, showing
 * what its subagent does, and folds when its result arrives; every other call starts closed.
 */
const props = withDefaults(defineProps<{ call: ToolItem; innerCalls?: number }>(), { innerCalls: 0 })

const open = ref(opensByDefault(props.call))

watch(
  () => props.call.state,
  () => {
    if (foldsOnResult(props.call)) open.value = false
  }
)
const bodyId = `tool-${useId()}`

const output = computed(() => (showsOutput(props.call) ? props.call.result : null))
</script>

<template>
  <div class="rounded-card border border-border bg-surface" data-testid="tool-call-card" :data-state="call.state">
    <ToolCallRow :call="call" :open="open" :controls="bodyId" :inner-calls="innerCalls" @toggle="open = !open" />

    <div v-if="open" :id="bodyId" class="flex flex-col gap-2 border-t border-border px-3 py-2" data-testid="tool-call-body">
      <ToolInputView :input="call.input" />

      <slot name="inner" />

      <ToolOutput v-if="output" :text="output.excerpt" :truncated="output.truncated" :is-error="output.isError" />
    </div>
  </div>
</template>
