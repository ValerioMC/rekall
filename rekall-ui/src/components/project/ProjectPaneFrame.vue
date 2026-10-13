<script setup lang="ts">
import { ref } from 'vue'
import { useContentBoxHeight } from '@/composables/useContentBoxHeight'

/**
 * The document column of the project page: a fixed header over a body that fills what is left
 * and scrolls on its own, so the page never scrolls to reach the text. The body hands its own
 * height to the editor, which then fits the pane exactly.
 */
defineProps<{ label: string; testid: string }>()

const EDITOR_FLOOR_PX = 320

const body = ref<HTMLElement | null>(null)
const editorHeight = useContentBoxHeight(body, EDITOR_FLOOR_PX)
</script>

<template>
  <section
    class="flex h-full min-h-0 flex-col overflow-hidden rounded-card border border-border bg-surface shadow-[var(--shadow-raised)]"
    :aria-label="label"
    :data-testid="testid"
  >
    <header class="relative shrink-0 border-b border-border">
      <slot name="header" />
    </header>
    <div ref="body" class="min-h-0 flex-1 overflow-y-auto p-4">
      <slot :editor-height="editorHeight" />
    </div>
  </section>
</template>
