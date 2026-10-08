<script setup lang="ts">
/** The notes written last, as tiles: kind, title, where they sit and how long ago. */
import { storeToRefs } from 'pinia'
import NoteKindGlyph from '@/components/ui/NoteKindGlyph.vue'
import { useConsoleStore } from '@/stores/console.store'
import { relativeTime } from '@/common/format/relative-time'

const store = useConsoleStore()
const { recentDocuments, isLoading } = storeToRefs(store)
</script>

<template>
  <section aria-labelledby="home-recent-title" data-testid="home-recent">
    <h2 id="home-recent-title" class="eyebrow mb-2.5 uppercase tracking-[0.12em]">Continue writing</h2>

    <div v-if="isLoading" class="grid grid-cols-1 gap-3 @xl:grid-cols-2 @4xl:grid-cols-4" aria-hidden="true">
      <div v-for="tile in 4" :key="tile" class="skeleton h-28 rounded-card" />
    </div>

    <div v-else-if="recentDocuments.length" class="grid grid-cols-1 gap-3 @xl:grid-cols-2 @4xl:grid-cols-4">
      <button
        v-for="document in recentDocuments"
        :key="document.id"
        type="button"
        class="focus-ring flex min-h-28 flex-col gap-2 rounded-card border border-border bg-surface p-3.5 text-left shadow-(--shadow-raised) transition-colors duration-(--motion-quick) hover:border-border-strong hover:bg-surface-raised"
        data-testid="resume-row"
        @click="store.selectDocument(document.id)"
      >
        <span class="flex items-center gap-1.5 text-caption text-text-muted">
          <NoteKindGlyph :kind="document.kind" />
          {{ document.kind }}
        </span>
        <span class="line-clamp-2 text-body font-medium leading-snug text-text">{{ document.title }}</span>
        <span class="mt-auto flex items-baseline justify-between gap-2 text-caption">
          <span class="min-w-0 truncate font-mono text-anchor">
            {{ document.tasks[0]?.anchor ?? document.anchor }}
          </span>
          <span class="shrink-0 text-text-subtle">{{ relativeTime(document.updatedAt) }}</span>
        </span>
      </button>
    </div>

    <p v-else class="text-body leading-relaxed text-text-muted">
      Nothing written yet. Create a task on the left, then press
      <kbd class="rounded-key border border-border px-1 font-mono text-micro">N</kbd> to write its
      first note.
    </p>
  </section>
</template>
