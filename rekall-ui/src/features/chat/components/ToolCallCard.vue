<script setup lang="ts">
import { computed, onUnmounted, ref, useId } from 'vue'
import CopyGlyph from '@/components/ui/CopyGlyph.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import ToolDiff from './ToolDiff.vue'
import ToolGlyph from './ToolGlyph.vue'
import ToolOutput from './ToolOutput.vue'
import ToolStateGlyph from './ToolStateGlyph.vue'
import { describeTool } from '@/features/chat/lib/describe-tool'
import { diffLines, writeLines } from '@/features/chat/lib/tool-diff'
import type { ToolItem } from '@/features/chat/model/timeline'

/**
 * One tool call as a single row, opened to its input and output. An edit shows as the lines it
 * changes, a command as the command and the end of its output.
 */
const props = defineProps<{ call: ToolItem }>()

const COPIED_FLASH_MS = 1400

const open = ref(false)
const copied = ref(false)
const bodyId = `tool-${useId()}`
let copiedTimer: ReturnType<typeof setTimeout> | null = null

const description = computed(() => describeTool(props.call.input))
const showsOutput = computed(() => {
  const { input, result } = props.call
  if (!result) return false
  return (input.tool !== 'edit' && input.tool !== 'write') || result.isError
})

async function copyCommand(): Promise<void> {
  if (props.call.input.tool !== 'bash') return
  await navigator.clipboard?.writeText(props.call.input.command)
  copied.value = true
  if (copiedTimer) clearTimeout(copiedTimer)
  copiedTimer = setTimeout(() => (copied.value = false), COPIED_FLASH_MS)
}

onUnmounted(() => {
  if (copiedTimer) clearTimeout(copiedTimer)
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
      <template v-if="call.input.tool === 'bash'">
        <div class="flex items-start gap-2 rounded-control border border-border bg-(--color-well-top) px-2 py-1.5">
          <pre class="min-w-0 flex-1 font-mono text-small whitespace-pre-wrap break-words text-text"><span class="select-none text-text-subtle">$ </span>{{ call.input.command }}</pre>
          <button
            type="button"
            class="focus-ring relative grid size-6 shrink-0 cursor-pointer place-items-center rounded-control text-text-subtle hover:text-text hit-target"
            aria-label="Copy the command"
            title="Copy the command"
            @click="copyCommand"
          >
            <CopyGlyph :copied="copied" />
          </button>
        </div>
        <p v-if="call.input.description" class="text-caption text-text-muted">{{ call.input.description }}</p>
      </template>

      <template v-else-if="call.input.tool === 'edit'">
        <div class="flex items-center gap-2">
          <span class="min-w-0 truncate font-mono text-small text-text">{{ call.input.filePath }}</span>
          <AppBadge v-if="call.input.replaceAll" tone="neutral">all occurrences</AppBadge>
        </div>
        <ToolDiff :lines="diffLines(call.input.oldString, call.input.newString)" />
      </template>

      <template v-else-if="call.input.tool === 'write'">
        <span class="min-w-0 truncate font-mono text-small text-text">{{ call.input.filePath }}</span>
        <ToolDiff :lines="writeLines(call.input.content)" />
      </template>

      <pre
        v-else-if="call.input.tool === 'other'"
        class="rounded-control border border-border bg-(--color-well-top) px-2 py-1.5 font-mono text-small whitespace-pre-wrap break-words text-text-muted"
        >{{ call.input.text }}</pre
      >

      <ToolOutput
        v-if="showsOutput && call.result"
        :text="call.result.excerpt"
        :truncated="call.result.truncated"
        :is-error="call.result.isError"
      />
    </div>
  </div>
</template>
