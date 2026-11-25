<script setup lang="ts">
import { computed, ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import ExportDialog from '@/features/export/components/ExportDialog.vue'
import { useConsoleStore } from '@/stores/console.store'
import type { ExportSelection } from '@/features/export/model/project-export'
import type { ProjectId } from '@/model/branded'

/**
 * The Export key of a project pane. It opens the export dialog with the pane's own item ticked and,
 * when there is nothing to export, stays disabled and says why. `settle` lets the pane save a
 * pending draft first, so the file holds what the pane shows.
 */
const props = withDefaults(
  defineProps<{ projectId: ProjectId; selection: ExportSelection; emptyReason?: string | null; settle?: () => Promise<void> }>(),
  { emptyReason: null, settle: undefined }
)

const console_ = useConsoleStore()
const open = ref(false)
const opened = ref<ExportSelection | null>(null)

const project = computed(() => console_.projects.find((candidate) => candidate.id === props.projectId) ?? null)
const reason = computed(() => props.emptyReason ?? (project.value ? null : 'The project is not loaded yet.'))

async function show(): Promise<void> {
  await props.settle?.()
  opened.value = props.selection
  open.value = true
}
</script>

<template>
  <span class="inline-flex" :title="reason ?? undefined" data-testid="export-action-frame">
    <AppButton variant="quiet" size="sm" :disabled="reason !== null" :aria-describedby="reason ? 'export-action-reason' : undefined" data-testid="export-action" @click="show">
      <svg class="size-3.5" viewBox="0 0 16 16" fill="none" aria-hidden="true">
        <path d="M4 1.8h5l3 3v8.4a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1V2.8a1 1 0 0 1 1-1Z" stroke="currentColor" stroke-width="1.3" stroke-linejoin="round" />
        <path d="M8 6.6v4.3m0 0L6.3 9.2M8 10.9l1.7-1.7" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round" />
      </svg>
      Export
    </AppButton>
    <span v-if="reason" id="export-action-reason" class="sr-only">{{ reason }}</span>
  </span>
  <ExportDialog v-if="open && project && opened" :project="project" :initial="opened" @close="open = false" />
</template>
