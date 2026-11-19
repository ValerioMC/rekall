<script setup lang="ts">
interface Shortcut {
  readonly keys: string
  readonly does: string
}

const SHORTCUTS: readonly Shortcut[] = [
  { keys: '⌘K', does: 'Search tasks, projects, notes and screens, or find a phrase in the text' },
  { keys: 'T', does: 'New task' },
  { keys: 'Q', does: 'Open the run queue' },
  { keys: 'Esc', does: 'Close the sheet, dialog or palette in front' }
]
</script>

<template>
  <section class="flex flex-col gap-4" aria-label="Keyboard" data-testid="keyboard-section">
    <h2 class="text-heading font-medium text-text">Keyboard</h2>
    <p class="text-small text-text-muted">Shortcuts are bare keys, and ignored while a field has focus.</p>
    <dl class="divide-y divide-border overflow-hidden rounded-card border border-border bg-surface">
      <div v-for="shortcut in SHORTCUTS" :key="shortcut.keys" class="flex items-center gap-4 px-4 py-2.5">
        <dt>
          <kbd class="rounded-key border border-b-2 border-border-strong bg-surface-raised px-1.5 font-mono text-caption text-text-muted">{{ shortcut.keys }}</kbd>
        </dt>
        <dd class="text-body text-text">{{ shortcut.does }}</dd>
      </div>
    </dl>
  </section>
</template>
