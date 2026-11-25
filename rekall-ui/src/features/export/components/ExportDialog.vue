<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { RouterLink } from 'vue-router'
import AppButton from '@/components/ui/AppButton.vue'
import AppDialog from '@/components/ui/AppDialog.vue'
import CloseGlyph from '@/components/ui/CloseGlyph.vue'
import SegmentedControl from '@/components/ui/SegmentedControl.vue'
import ExportItemPicker from '@/features/export/components/ExportItemPicker.vue'
import ExportOutcome from '@/features/export/components/ExportOutcome.vue'
import { useProjectExport } from '@/features/export/composables/useProjectExport'
import { exportFileUrl } from '@/features/export/api/project-export.api'
import { pageGroupsOf } from '@/features/export/lib/export-items'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useProjectComponentsStore } from '@/features/project/store/projectComponents.store'
import { useProjectSectionsStore } from '@/features/project/store/projectSections.store'
import type { Project } from '@/features/catalog/model/catalog'
import type { ExportSelection } from '@/features/export/model/project-export'

/** Export of one project: tick what goes in, choose PDF or Word, and the file downloads when it is written. */
const props = defineProps<{ project: Project; initial: ExportSelection }>()
const emit = defineEmits<{ close: [] }>()

const FORMATS = [
  { value: 'PDF', label: 'PDF' },
  { value: 'DOCX', label: 'Word' }
] as const

const sections = useProjectSectionsStore()
const components = useProjectComponentsStore()
const { run } = useAsyncAction()
const exporting = useProjectExport(props.project, props.initial)
const { format, selection, ticked, busy, phase } = exporting

const pageGroups = computed(() => pageGroupsOf(sections.contextsOf(props.project.id), sections.sectionsOf(props.project.id)))
const projectComponents = computed(() => components.componentsOf(props.project.id))
const blueprintEmpty = computed(() => (props.project.blueprintMarkdown ?? '').trim() === '')
const finished = computed(() => phase.value.kind === 'done')
const formatName = computed(() => (format.value === 'PDF' ? 'PDF' : 'Word'))
const failure = computed(() => (phase.value.kind === 'failed' ? phase.value : null))
const doneEntry = computed(() => (phase.value.kind === 'done' ? phase.value.entry : null))
const writing = computed(() => phase.value.kind === 'writing')

const statusLine = computed(() => {
  const current = phase.value
  if (current.kind === 'drawing') return `Drawing diagrams ${Math.min(current.finished + 1, current.total)} of ${current.total}`
  if (current.kind === 'writing') return `Writing the ${formatName.value}`
  if (current.kind === 'done') return 'Download started'
  if (ticked.value === 0) return 'Tick at least one item.'
  return `${ticked.value} ${ticked.value === 1 ? 'item' : 'items'}, ${formatName.value}`
})

function dismiss(): void {
  if (phase.value.kind === 'drawing') {
    exporting.cancel()
    return
  }
  if (!writing.value) emit('close')
}

onMounted(() => {
  if (!sections.isLoaded(props.project.id)) void run(() => sections.load(props.project.id))
  void run(() => components.ensureLoaded(props.project.id))
})
</script>

<template>
  <AppDialog :label="`Export ${project.title}`" panel-class="max-w-[560px]" panel-testid="export-dialog" :dismissible="!writing" @dismiss="dismiss">
    <header class="flex items-center gap-3 border-b border-border px-5 py-4">
      <span class="min-w-0 flex-1">
        <span class="block text-title font-semibold tracking-[-0.01em] text-text">Export</span>
        <span class="mt-0.5 block truncate text-caption text-text-subtle">{{ project.title }} · {{ project.companyName }}</span>
      </span>
      <button
        class="focus-ring grid size-7 shrink-0 place-items-center rounded-control text-text-subtle transition-colors hover:bg-surface-raised hover:text-text"
        aria-label="Close"
        :disabled="writing"
        @click="dismiss"
      >
        <CloseGlyph />
      </button>
    </header>

    <template v-if="!finished">
      <div class="flex items-center justify-between gap-3 border-b border-border px-5 py-3">
        <SegmentedControl v-model="format" :options="FORMATS" label="File format" :class="busy && 'pointer-events-none opacity-60'" />
        <span class="text-small tabular-nums text-text-muted" data-testid="export-count">{{ ticked }} ticked</span>
      </div>
      <div class="max-h-[60vh] overflow-y-auto px-5 py-4">
        <ExportItemPicker
          :selection="selection"
          :blueprint-empty="blueprintEmpty"
          :page-groups="pageGroups"
          :components="projectComponents"
          :disabled="busy"
          @blueprint="exporting.setBlueprint($event)"
          @page="exporting.toggleSection($event)"
          @component="exporting.toggleComponent($event)"
          @pages="(ids, on) => exporting.setSections(ids, on)"
          @components="(ids, on) => exporting.setComponents(ids, on)"
        />
      </div>
      <div v-if="failure" class="px-5 pb-4">
        <ExportOutcome :entry="null" :failure="failure" />
      </div>
    </template>
    <div v-else class="px-5 py-5">
      <ExportOutcome :entry="doneEntry" :failure="null" />
    </div>

    <footer class="flex items-center gap-3 border-t border-border bg-canvas px-5 py-3">
      <p class="min-w-0 flex-1 truncate text-small text-text-subtle" role="status" data-testid="export-status">{{ statusLine }}</p>
      <template v-if="finished">
        <RouterLink to="/exports" class="focus-ring rounded-control px-2 py-1 text-small text-text-muted hover:text-text" data-testid="export-open-page" @click="emit('close')">
          Go to Exports
        </RouterLink>
        <a
          v-if="doneEntry"
          :href="exportFileUrl(doneEntry.id)"
          :download="doneEntry.fileName"
          class="focus-ring key-slate inline-flex h-7 items-center rounded-control px-3 text-caption font-medium"
          data-testid="export-again"
        >
          Download again
        </a>
        <AppButton variant="primary" size="sm" data-testid="export-close" @click="emit('close')">Done</AppButton>
      </template>
      <template v-else>
        <AppButton variant="ghost" size="sm" :disabled="writing" data-testid="export-cancel" @click="dismiss">Cancel</AppButton>
        <AppButton variant="primary" size="sm" :disabled="ticked === 0" :loading="busy" data-testid="export-submit" @click="exporting.start()">
          {{ failure ? 'Try again' : 'Export' }}
        </AppButton>
      </template>
    </footer>
  </AppDialog>
</template>
