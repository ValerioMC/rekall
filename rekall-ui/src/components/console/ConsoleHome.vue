<script setup lang="ts">
/** The console with nothing open: the notes touched last, one click from where they were, and the keys that drive it. */
import { storeToRefs } from 'pinia'
import { useConsoleStore } from '@/stores/console.store'

const store = useConsoleStore()
const { recentDocuments, isLoading } = storeToRefs(store)

const SHORTCUTS = [
  { keys: '⌘K', does: 'search by anchor, from anywhere' },
  { keys: 'T', does: 'new task' },
  { keys: 'E', does: 'edit the task in view' },
  { keys: 'N', does: 'new note on it' },
  { keys: 'D', does: 'its description: what the work is' },
  { keys: 'W', does: 'its wrapup: what it currently is' },
  { keys: 'R', does: 'toggle read / write, on a description or a note' },
  { keys: 'B', does: 'switch between tasks and notes' },
  { keys: 'J K', does: 'walk the list' },
  { keys: '1-5', does: 'set the status' }
] as const
</script>

<template>
  <div class="min-h-0 flex-1 overflow-y-auto">
    <div class="max-w-[640px] px-9 py-12">
      <p class="eyebrow">
        Continue
      </p>
      <h2 class="mb-1.5 mt-1.5 text-display font-semibold leading-tight tracking-[-0.025em] text-text">
        Pick up where you left off
      </h2>
      <p class="mb-6 text-body leading-relaxed text-text-muted">
        Everything here is one anchor away from a Claude session.
      </p>

      <div v-if="isLoading" class="flex flex-col gap-1.5" aria-hidden="true">
        <div v-for="row in 3" :key="row" class="skeleton h-14 rounded-control" />
      </div>

      <div v-else-if="recentDocuments.length" class="flex flex-col gap-1.5">
        <button
          v-for="document in recentDocuments"
          :key="document.id"
          data-testid="resume-row"
          class="focus-ring group relative flex w-full items-center gap-3 overflow-hidden rounded-control border border-border bg-surface px-3.5 py-3 text-left transition-all hover:-translate-y-px hover:border-border-strong hover:bg-surface-raised hover:shadow-lift"
          @click="store.selectDocument(document.id)"
        >
          <span
            class="pointer-events-none absolute right-0 top-0 size-3.5 transition-colors"
            style="
              clip-path: polygon(100% 0, 0 0, 100% 100%);
              background-image: linear-gradient(135deg, var(--color-border-strong), var(--color-canvas));
            "
            aria-hidden="true"
          />
          <span class="min-w-0 flex-1">
            <span class="block truncate text-body font-medium text-text">
              {{ document.title }}
            </span>
            <span class="mt-0.5 block truncate font-mono text-caption text-anchor">
              {{ document.tasks[0]?.anchor }}
            </span>
          </span>
          <span
            class="shrink-0 text-body text-text-subtle transition-transform group-hover:translate-x-0.5 group-hover:text-accent"
            aria-hidden="true"
          >
            &#8594;
          </span>
        </button>
      </div>

      <p v-else class="text-body leading-relaxed text-text-subtle">
        Nothing written yet. Create a task on the left, then press
        <kbd class="rounded-key border border-border px-1 font-mono text-micro">N</kbd> to write its
        first note.
      </p>

      <ul class="mt-8 flex flex-col gap-2 text-small">
        <li v-for="hint in SHORTCUTS" :key="hint.keys" class="flex items-center gap-3">
          <kbd
            class="w-9 shrink-0 rounded-key border border-border bg-surface py-0.5 text-center font-mono text-micro text-text-muted"
          >
            {{ hint.keys }}
          </kbd>
          <span class="text-text-subtle">{{ hint.does }}</span>
        </li>
      </ul>
    </div>
  </div>
</template>
