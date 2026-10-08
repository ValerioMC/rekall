<script setup lang="ts">
/**
 * What the list says when it has no rows to show: the folder could not be listed, may not be
 * read, holds nothing the filter matches, holds only hidden files, or is empty.
 */
defineProps<{
  failure: string | null
  unreadable: boolean
  needle: string
  showHidden: boolean
  hiddenCount: number
}>()

defineEmits<{ 'show-hidden': [] }>()
</script>

<template>
  <div v-if="failure" class="path-message" data-testid="path-picker-failure">
    <p class="text-text-muted">{{ failure }}</p>
    <p class="text-text-subtle">Keep typing, or <kbd>esc</kbd> to clear the filter.</p>
  </div>

  <div v-else-if="unreadable" class="path-message" data-testid="path-picker-unreadable">
    <p class="text-text-muted">Rekall is not allowed to read this folder.</p>
    <p class="text-text-subtle">
      macOS keeps some folders private to the apps that own them. <kbd>⌘↵</kbd> still inserts its path.
    </p>
  </div>

  <div v-else class="path-message" data-testid="path-picker-empty">
    <template v-if="needle">
      <p class="text-text-muted">
        Nothing here matches <span class="font-mono text-text">{{ needle }}</span>.
      </p>
      <p v-if="!showHidden && hiddenCount" class="text-text-subtle">
        Hidden files are left out.
        <button type="button" class="focus-ring text-accent hover:underline" @click="$emit('show-hidden')">Show them</button>
      </p>
    </template>
    <template v-else-if="hiddenCount">
      <p class="text-text-muted">Only hidden files in here.</p>
      <p class="text-text-subtle">
        <button type="button" class="focus-ring text-accent hover:underline" @click="$emit('show-hidden')">Show them</button>
        <kbd class="ml-1">⌘⇧.</kbd>, or <kbd>↵</kbd> to insert the folder itself.
      </p>
    </template>
    <template v-else>
      <p class="text-text-muted">This folder is empty.</p>
      <p class="text-text-subtle"><kbd>↵</kbd> inserts the folder itself.</p>
    </template>
  </div>
</template>

<style scoped>
.path-message {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 14px 14px 10px;
  font-size: var(--text-small);
  line-height: 1.5;
}
</style>
