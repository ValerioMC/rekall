<script setup lang="ts">
import { computed, nextTick, ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import StatusPill from '@/components/ui/StatusPill.vue'
import { formatDateTime } from '@/common/format/app-locale'
import { formatFileSize } from '@/common/format/file-size'
import { relativeTime } from '@/common/format/relative-time'
import { exportFileUrl } from '@/features/export/api/project-export.api'
import { DIAGRAM_ERROR_TEXT } from '@/features/export/model/diagram-error'
import type { ProjectExportEntry } from '@/features/export/model/project-export'

/**
 * One export: its format, what it holds, when and how big, with Download and Delete. A failed one
 * stands apart with its kind and message and is deleted at once; a done one asks first, in place.
 */
const props = defineProps<{ entry: ProjectExportEntry }>()
const emit = defineEmits<{ delete: [] }>()

const asking = ref(false)
const keepButton = ref<InstanceType<typeof AppButton> | null>(null)

const failed = computed(() => props.entry.status === 'FAILED')

async function askOrDelete(): Promise<void> {
  if (failed.value) {
    emit('delete')
    return
  }
  asking.value = true
  await nextTick()
  ;(keepButton.value?.$el as HTMLElement | undefined)?.focus()
}
</script>

<template>
  <li
    class="grid grid-cols-[56px_minmax(0,1fr)_110px_72px_264px] items-start gap-x-3 border-b border-border px-3 py-2 transition-colors duration-(--motion-quick) last:border-b-0"
    :class="failed ? 'bg-danger-soft' : 'hover:bg-surface-hover'"
    :data-testid="`export-row-${entry.id}`"
    :data-status="entry.status"
    @keydown.esc.stop="asking = false"
  >
    <span class="mt-0.5 inline-flex h-5 w-fit items-center rounded-key bg-surface-raised px-1.5 text-caption text-text-muted">
      {{ entry.format === 'PDF' ? 'PDF' : 'Word' }}
    </span>

    <div class="min-w-0">
      <p class="flex min-w-0 items-center gap-2">
        <span class="min-w-0 truncate text-body text-text" :title="entry.summary">{{ entry.summary }}</span>
        <template v-if="failed">
          <StatusPill tone="danger">Failed</StatusPill>
          <span class="font-mono text-caption text-danger" data-testid="export-failure-kind">{{ entry.failureKind }}</span>
        </template>
      </p>
      <p v-if="failed" class="mt-1 select-text whitespace-pre-line font-mono text-caption leading-relaxed text-danger" data-testid="export-failure-message">
        {{ entry.failureMessage }}
      </p>
      <p v-else class="truncate font-mono text-caption text-text-subtle" :title="entry.fileName">{{ entry.fileName }}</p>
      <p v-if="!failed && entry.diagramsMissing > 0" class="mt-1 flex items-start gap-1.5 text-caption text-warn" data-testid="export-row-diagrams-missing">
        <svg class="mt-px size-3.5 shrink-0" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <path d="M8 2.2 14.2 13H1.8L8 2.2ZM8 6.5v3M8 11.3v.1" />
        </svg>
        {{ entry.diagramsMissing === 1 ? '1 diagram' : `${entry.diagramsMissing} diagrams` }} could not be drawn. {{ entry.diagramsMissing === 1 ? 'It appears' : 'They appear' }} as “{{ DIAGRAM_ERROR_TEXT }}” in the file.
      </p>
    </div>

    <span class="mt-0.5 text-small text-text-muted" :title="formatDateTime(new Date(entry.createdAt))">{{ relativeTime(entry.createdAt) }}</span>
    <span class="mt-0.5 text-right text-small tabular-nums text-text-muted">{{ failed ? '—' : formatFileSize(entry.sizeBytes) }}</span>

    <div class="flex items-center justify-end gap-1">
      <template v-if="asking">
        <span class="mr-1 text-small text-text-muted">Delete this file?</span>
        <AppButton ref="keepButton" variant="ghost" size="sm" data-testid="export-keep" @click="asking = false">Keep</AppButton>
        <AppButton variant="danger" size="sm" data-testid="export-delete-confirm" @click="emit('delete')">Delete</AppButton>
      </template>
      <template v-else>
        <a
          v-if="!failed"
          :href="exportFileUrl(entry.id)"
          :download="entry.fileName"
          class="focus-ring key-slate inline-flex h-7 items-center rounded-control px-3 text-caption font-medium"
          data-testid="export-download"
        >
          Download
        </a>
        <AppButton variant="danger-quiet" size="sm" data-testid="export-delete" @click="askOrDelete">Delete</AppButton>
      </template>
    </div>
  </li>
</template>
