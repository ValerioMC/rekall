<script setup lang="ts">
import ToolStateGlyph from './ToolStateGlyph.vue'
import type { ToolState } from '@/features/chat/model/timeline'

/** A command the session ran, told by what it is for: neither its text nor its output is shown. */
defineProps<{ description: string; state: ToolState }>()
</script>

<template>
  <p class="flex items-start gap-2 px-1 py-0.5 text-small text-text-muted" data-testid="chat-command-line" :data-state="state">
    <ToolStateGlyph class="h-[1lh]" :state="state" />
    <span class="min-w-0 break-words">{{ description }}</span>
  </p>
</template>
