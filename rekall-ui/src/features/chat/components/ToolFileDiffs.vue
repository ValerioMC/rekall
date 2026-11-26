<script setup lang="ts">
import ToolDiff from './ToolDiff.vue'
import type { FileDiff } from '@/features/chat/lib/tool-diff'

/** The changed files one under the other. `named` heads each with its path, for a set of several. */
withDefaults(defineProps<{ files: readonly FileDiff[]; named?: boolean; limit?: number }>(), {
  named: false,
  limit: 40
})
</script>

<template>
  <div class="divide-y divide-border bg-(--color-well-top)">
    <section v-for="(file, index) in files" :key="index" data-testid="tool-file">
      <p v-if="named || file.note" class="flex flex-wrap gap-x-2 border-b border-border px-3 py-1 text-caption">
        <span v-if="named" class="min-w-0 font-mono break-all text-text">{{ file.path }}</span>
        <span v-if="file.note" class="min-w-0 break-all text-text-muted">{{ file.note }}</span>
      </p>
      <ToolDiff :lines="file.lines" :limit="limit" />
    </section>
  </div>
</template>
