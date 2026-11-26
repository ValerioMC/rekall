<script setup lang="ts">
import { computed, ref, useId, watch } from 'vue'
import ToolCallRow from './ToolCallRow.vue'
import ToolFileDiffs from './ToolFileDiffs.vue'
import ToolInputView from './ToolInputView.vue'
import ToolOutput from './ToolOutput.vue'
import { foldsOnResult, opensByDefault, showsOutput } from '@/features/chat/lib/tool-card'
import { fileDiffs } from '@/features/chat/lib/tool-diff'
import type { ToolItem } from '@/features/chat/model/timeline'

/**
 * One tool call: a row that says what it was and what it did, opened to its input and output.
 * A change to files starts open on its diff, edge to edge. A todo list starts open too; an Agent is
 * open while it runs and folds when its result arrives; every other call starts closed.
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

const files = computed(() => fileDiffs(props.call.input))
const output = computed(() => (showsOutput(props.call) ? props.call.result : null))
</script>

<template>
  <div class="rounded-card border border-border bg-surface" data-testid="tool-call-card" :data-state="call.state">
    <ToolCallRow :call="call" :open="open" :controls="bodyId" :inner-calls="innerCalls" @toggle="open = !open" />

    <div
      v-if="open"
      :id="bodyId"
      class="border-t border-border"
      :class="files.length > 0 ? 'overflow-hidden rounded-b-card' : 'flex flex-col gap-2 px-3 py-2'"
      data-testid="tool-call-body"
    >
      <ToolFileDiffs v-if="files.length > 0" :files="files" :named="files.length > 1" />

      <template v-else>
        <ToolInputView :input="call.input" />
        <slot name="inner" />
      </template>

      <ToolOutput
        v-if="output"
        :class="files.length > 0 && 'm-2'"
        :text="output.excerpt"
        :truncated="output.truncated"
        :is-error="output.isError"
      />
    </div>
  </div>
</template>
