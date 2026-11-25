<script setup lang="ts">
import StatusPill from '@/components/ui/StatusPill.vue'
import { formatFileSize } from '@/common/format/file-size'
import { DIAGRAM_ERROR_TEXT } from '@/features/export/model/diagram-error'
import type { ProjectExportEntry } from '@/features/export/model/project-export'

/** What the dialog shows once an export has finished: the file that was made, or the failure that was kept. */
defineProps<{ entry: ProjectExportEntry | null; failure: { readonly message: string; readonly recorded: boolean } | null }>()
</script>

<template>
  <div v-if="entry" class="flex flex-col gap-3" data-testid="export-done">
    <div class="flex items-center gap-2.5">
      <StatusPill tone="done">Done</StatusPill>
      <span class="min-w-0 truncate font-mono text-small text-text" :title="entry.fileName">{{ entry.fileName }}</span>
    </div>
    <p class="text-small text-text-muted">{{ entry.summary }} · {{ entry.format === 'PDF' ? 'PDF' : 'Word' }} · {{ formatFileSize(entry.sizeBytes) }}</p>
    <p v-if="entry.diagramsMissing > 0" class="flex items-start gap-2 text-small text-warn" data-testid="export-diagrams-missing">
      <svg class="mt-0.5 size-3.5 shrink-0" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <path d="M8 2.2 14.2 13H1.8L8 2.2ZM8 6.5v3M8 11.3v.1" />
      </svg>
      <span>
        {{ entry.diagramsMissing === 1 ? '1 diagram' : `${entry.diagramsMissing} diagrams` }} could not be drawn. {{ entry.diagramsMissing === 1 ? 'It appears' : 'They appear' }} as “{{ DIAGRAM_ERROR_TEXT }}” in the file.
      </span>
    </p>
  </div>
  <div v-else-if="failure" class="flex flex-col gap-1.5" role="alert" data-testid="export-failed">
    <p class="whitespace-pre-line rounded-control border border-danger/40 bg-danger-soft px-3 py-2 font-mono text-caption leading-relaxed text-danger select-text">{{ failure.message }}</p>
    <p class="text-caption text-text-subtle">{{ failure.recorded ? 'Recorded on the Exports page.' : 'Not recorded.' }}</p>
  </div>
</template>
