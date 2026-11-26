<script setup lang="ts">
import { computed, onUnmounted, ref } from 'vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import CopyGlyph from '@/components/ui/CopyGlyph.vue'
import ToolArguments from './ToolArguments.vue'
import ToolFileDiffs from './ToolFileDiffs.vue'
import ToolTodoList from './ToolTodoList.vue'
import { fileDiffs } from '@/features/chat/lib/tool-diff'
import type { ToolInput } from '@/features/chat/model/chat-event'

/**
 * What a tool was asked to do: the command with a copy key, the lines an edit changes, the file a
 * write creates. `lineLimit` caps the lines of a diff before the rest waits behind a key.
 */
const props = withDefaults(defineProps<{ input: ToolInput; lineLimit?: number }>(), { lineLimit: 40 })

const COPIED_FLASH_MS = 1400

const files = computed(() => fileDiffs(props.input))
const copied = ref(false)
let copiedTimer: ReturnType<typeof setTimeout> | null = null

async function copyCommand(): Promise<void> {
  if (props.input.tool !== 'bash') return
  await navigator.clipboard?.writeText(props.input.command)
  copied.value = true
  if (copiedTimer) clearTimeout(copiedTimer)
  copiedTimer = setTimeout(() => (copied.value = false), COPIED_FLASH_MS)
}

onUnmounted(() => {
  if (copiedTimer) clearTimeout(copiedTimer)
})
</script>

<template>
  <template v-if="input.tool === 'bash'">
    <div class="flex items-start gap-2 rounded-control border border-border bg-(--color-well-top) px-2 py-1.5">
      <pre class="min-w-0 flex-1 font-mono text-small whitespace-pre-wrap break-words text-text"><span class="select-none text-text-subtle">$ </span>{{ input.command }}</pre>
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
    <p v-if="input.description" class="text-caption text-text-muted">{{ input.description }}</p>
  </template>

  <template v-else-if="files.length > 0">
    <span v-if="files.length === 1" class="min-w-0 truncate font-mono text-small text-text">{{ files[0]!.path }}</span>
    <ToolFileDiffs
      class="overflow-hidden rounded-control border border-border"
      :files="files"
      :named="files.length > 1"
      :limit="lineLimit"
    />
  </template>

  <ToolTodoList v-else-if="input.tool === 'todos'" :items="input.items" />

  <template v-else-if="input.tool === 'agent'">
    <div class="flex items-center gap-2">
      <span class="min-w-0 truncate text-small text-text">{{ input.description }}</span>
      <AppBadge v-if="input.subagentType" tone="neutral">{{ input.subagentType }}</AppBadge>
    </div>
    <pre
      class="rounded-control border border-border bg-(--color-well-top) px-2 py-1.5 font-mono text-small whitespace-pre-wrap break-words text-text-muted"
      >{{ input.prompt }}</pre
    >
  </template>

  <p v-else-if="input.tool === 'web'" class="min-w-0 break-all font-mono text-small text-text" data-testid="tool-web-target">
    {{ input.target }}
  </p>

  <ToolArguments v-else-if="input.tool === 'mcp'" :text="input.text" />

  <pre
    v-else-if="input.tool === 'other'"
    class="rounded-control border border-border bg-(--color-well-top) px-2 py-1.5 font-mono text-small whitespace-pre-wrap break-words text-text-muted"
    >{{ input.text }}</pre
  >
</template>
