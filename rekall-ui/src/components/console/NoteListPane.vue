<script setup lang="ts">
import { storeToRefs } from 'pinia'
import WrapupCard from '@/components/console/WrapupCard.vue'
import { useConsoleStore } from '@/stores/console.store'

/**
 * The notes on the selected task, with its wrapup pinned above them.
 *
 * The wrapup is first because it is the answer to the question you arrive with — what is this
 * now — and the notes are the background to that answer. It is one row, always present, even
 * when nothing has been written: the absence is the reason to write one.
 */
const store = useConsoleStore()
const { selectedTask, selectedDocId, taskDocuments, paneFocus, selectedWrapup, wrapupIsBehind } =
  storeToRefs(store)

/** A couple of lines of the body, with the markup stripped so the preview reads as prose. */
function excerpt(body: string): string {
  return body.replace(/[#`>*|_-]/g, ' ').replace(/\s+/g, ' ').trim().slice(0, 110)
}
</script>

<template>
  <section
    class="flex min-h-0 w-(--spacing-notelist) shrink-0 flex-col border-r border-border bg-surface"
    aria-label="Notes on the selected task"
  >
    <header class="flex h-(--spacing-header) shrink-0 items-center gap-2 border-b border-border px-3.5">
      <span class="min-w-0 flex-1">
        <span class="block truncate text-[13.5px] font-semibold text-text">
          {{ selectedTask?.title ?? 'Notes' }}
        </span>
        <span
          v-if="selectedTask"
          class="block truncate font-mono text-[10.5px] text-anchor/80"
          :title="selectedTask.anchor"
        >
          {{ selectedTask.anchor }}
        </span>
      </span>
      <span v-if="taskDocuments.length" class="font-mono text-[11px] text-text-subtle">
        {{ taskDocuments.length }}
      </span>
    </header>

    <div class="min-h-0 flex-1 overflow-y-auto p-2">
      <p v-if="!selectedTask" class="px-2 py-2 text-[12.5px] leading-relaxed text-text-subtle">
        Pick a task to see its notes.
      </p>

      <template v-else>
        <WrapupCard
          :wrapup="selectedWrapup"
          :selected="paneFocus === 'wrapup'"
          :behind="wrapupIsBehind"
          @open="store.openWrapup()"
        />

        <p
          v-if="!taskDocuments.length"
          class="px-2 py-2 text-[12.5px] leading-relaxed text-text-subtle"
        >
          No note on this task yet. Press
          <kbd class="rounded border border-border px-1 font-mono text-[10px]">N</kbd>
          to write the first one.
        </p>
      </template>

      <button
        v-for="document in taskDocuments"
        :key="document.id"
        data-testid="note-card"
        class="focus-ring mb-1 block w-full rounded-[var(--radius-control)] border p-2.5 text-left transition-all"
        :class="
          document.id === selectedDocId && paneFocus === 'note'
            ? 'border-accent bg-surface-raised shadow-lift'
            : 'border-transparent hover:border-border-strong hover:bg-surface-raised'
        "
        :aria-current="document.id === selectedDocId && paneFocus === 'note'"
        @click="store.selectDocument(document.id)"
      >
        <span class="flex items-center gap-2">
          <span class="min-w-0 flex-1 truncate text-[13px] font-medium text-text">
            {{ document.title }}
          </span>
          <span
            class="shrink-0 rounded border border-border bg-surface-raised px-1.5 py-px font-mono text-[9.5px] text-text-muted"
          >
            {{ document.kind }}
          </span>
          <span
            v-if="document.tasks.length > 1"
            class="anchor-chip shrink-0 px-1.5 py-px text-[9.5px]"
            :title="`On ${document.tasks.length} tasks`"
          >
            &#8942; {{ document.tasks.length }}
          </span>
        </span>
        <span class="mt-1 line-clamp-2 block text-[11.5px] leading-relaxed text-text-subtle">
          {{ excerpt(document.bodyMarkdown) }}
        </span>
      </button>
    </div>
  </section>
</template>
