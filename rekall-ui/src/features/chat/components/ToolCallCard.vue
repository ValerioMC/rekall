<script setup lang="ts">
import { computed, ref, useId } from 'vue'
import ToolCallRow from './ToolCallRow.vue'
import ToolInputView from './ToolInputView.vue'
import ToolOutput from './ToolOutput.vue'
import { opensByDefault, showsOutput } from '@/features/chat/lib/tool-card'
import type { ToolItem } from '@/features/chat/model/timeline'

/**
 * One tool call: a row that says what it was and what it did, opened to its input and output.
 * A todo list starts open, being the progress of the work; every other call starts closed.
 */
const props = defineProps<{ call: ToolItem }>()

const open = ref(opensByDefault(props.call.input))
const bodyId = `tool-${useId()}`

const output = computed(() => (showsOutput(props.call) ? props.call.result : null))
</script>

<template>
  <div class="rounded-card border border-border bg-surface" data-testid="tool-call-card" :data-state="call.state">
    <ToolCallRow :call="call" :open="open" :controls="bodyId" @toggle="open = !open" />

    <div v-if="open" :id="bodyId" class="flex flex-col gap-2 border-t border-border px-3 py-2" data-testid="tool-call-body">
      <ToolInputView :input="call.input" />

      <ToolOutput v-if="output" :text="output.excerpt" :truncated="output.truncated" :is-error="output.isError" />
    </div>
  </div>
</template>
