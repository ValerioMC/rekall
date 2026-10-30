<script setup lang="ts">
import { onUnmounted, ref } from 'vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import CopyGlyph from '@/components/ui/CopyGlyph.vue'
import ToolDiff from './ToolDiff.vue'
import { diffLines, writeLines } from '@/features/chat/lib/tool-diff'
import type { ToolInput } from '@/features/chat/model/chat-event'

/**
 * What a tool was asked to do: the command with a copy key, the lines an edit changes, the file a
 * write creates. `lineLimit` caps the lines of a diff before the rest waits behind a key.
 */
const props = withDefaults(defineProps<{ input: ToolInput; lineLimit?: number }>(), { lineLimit: 40 })

const COPIED_FLASH_MS = 1400

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

  <template v-else-if="input.tool === 'edit'">
    <div class="flex items-center gap-2">
      <span class="min-w-0 truncate font-mono text-small text-text">{{ input.filePath }}</span>
      <AppBadge v-if="input.replaceAll" tone="neutral">all occurrences</AppBadge>
    </div>
    <ToolDiff :lines="diffLines(input.oldString, input.newString)" :limit="lineLimit" />
  </template>

  <template v-else-if="input.tool === 'write'">
    <span class="min-w-0 truncate font-mono text-small text-text">{{ input.filePath }}</span>
    <ToolDiff :lines="writeLines(input.content)" :limit="lineLimit" />
  </template>

  <pre
    v-else-if="input.tool === 'other'"
    class="rounded-control border border-border bg-(--color-well-top) px-2 py-1.5 font-mono text-small whitespace-pre-wrap break-words text-text-muted"
    >{{ input.text }}</pre
  >
</template>
