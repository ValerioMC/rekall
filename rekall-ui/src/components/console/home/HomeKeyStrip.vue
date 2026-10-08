<script setup lang="ts">
/** The keys that drive the console, as one wrapping strip of keycaps along the foot of the home screen. */
const SHORTCUTS = [
  { keys: '⌘K', does: 'search by anchor' },
  { keys: 'T', does: 'new task' },
  { keys: 'E', does: 'edit task' },
  { keys: 'N', does: 'new note' },
  { keys: 'D', does: 'description' },
  { keys: 'W', does: 'wrapup' },
  { keys: 'R', does: 'read / write' },
  { keys: 'B', does: 'tasks / notes' },
  { keys: 'J K', does: 'walk the list' },
  { keys: '1-5', does: 'set status' }
] as const
</script>

<template>
  <footer class="border-t border-border pt-4" aria-label="Keyboard shortcuts">
    <ul class="flex flex-wrap gap-x-5 gap-y-2.5 text-small text-text-muted">
      <li v-for="hint in SHORTCUTS" :key="hint.keys" class="flex items-center gap-2">
        <kbd
          class="min-w-5 rounded-key border border-b-2 border-border-strong bg-surface-raised px-1.5 text-center font-mono text-micro leading-[18px] text-text-muted"
        >
          {{ hint.keys }}
        </kbd>
        {{ hint.does }}
      </li>
    </ul>
  </footer>
</template>
