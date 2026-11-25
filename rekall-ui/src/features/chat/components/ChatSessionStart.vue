<script setup lang="ts">
/** The start of a session in one quiet row: the model, where it runs and how it asks for leave. */
defineProps<{ model: string; cwd: string; permissionMode: string }>()
</script>

<template>
  <p class="flex flex-wrap items-baseline gap-x-2 text-caption text-text-subtle" data-testid="chat-session-start">
    <span class="font-semibold text-text-muted">{{ model }}</span>
    <span aria-hidden="true">·</span>
    <span class="min-w-0 break-all font-mono">{{ cwd }}</span>
    <span aria-hidden="true">·</span>
    <span>{{ permissionMode }} permissions</span>
  </p>
</template>
