<script setup lang="ts">
/** What a local command such as /context printed, on the well, labelled with the command. */
defineProps<{ command: string; text: string }>()
</script>

<template>
  <section class="overflow-hidden rounded-control border border-border bg-(--color-well-top)" data-testid="chat-command-output">
    <h3 class="border-b border-border bg-surface px-2 py-1 font-mono text-caption text-text-muted">{{ command }}</h3>
    <pre class="max-h-96 overflow-auto px-2 py-1.5 font-mono text-small whitespace-pre text-text-muted" tabindex="0">{{ text }}</pre>
  </section>
</template>
