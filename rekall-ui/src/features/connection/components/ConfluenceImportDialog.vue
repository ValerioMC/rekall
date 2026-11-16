<script setup lang="ts">
import { computed, ref } from 'vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppCheckbox from '@/components/ui/AppCheckbox.vue'
import AppDialog from '@/components/ui/AppDialog.vue'
import CloseGlyph from '@/components/ui/CloseGlyph.vue'
import ConnectionKindIcon from '@/features/connection/components/ConnectionKindIcon.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useConsoleStore } from '@/stores/console.store'
import type { ConfluenceImport, ProjectConnection } from '@/features/connection/model/connection'

/**
 * Fills the project's description, blueprint and sections from the Confluence space a binding names.
 * What the project already has is kept unless the person asks for it to be rewritten.
 */
const props = defineProps<{ binding: ProjectConnection }>()
const emit = defineEmits<{ close: [] }>()

const consoleStore = useConsoleStore()
const { run, isRunning } = useAsyncAction()
const replace = ref(false)
const imported = ref<ConfluenceImport | null>(null)

const heading = computed(() => `Fill project from ${props.binding.title}`)

const OUTCOME_LABEL = { created: 'Added', replaced: 'Rewritten', kept: 'Kept' } as const

async function fill(): Promise<void> {
  if (isRunning.value) return
  const result = await run(() => consoleStore.importConfluenceSpace(props.binding.projectId, props.binding.id, replace.value))
  if (result) imported.value = result
}
</script>

<template>
  <AppDialog :label="heading" panel-class="max-w-[560px]" panel-testid="confluence-import-dialog" @dismiss="emit('close')">
    <header class="flex items-center gap-3 border-b border-border px-6 py-4">
      <ConnectionKindIcon kind="CONFLUENCE" :size="18" />
      <span class="min-w-0 flex-1 text-title font-semibold tracking-[-0.01em] text-text">{{ heading }}</span>
      <span class="font-mono text-caption text-text-subtle">{{ binding.target }}</span>
      <button
        class="focus-ring grid size-7 place-items-center rounded-control text-text-subtle transition-colors hover:bg-surface-raised hover:text-text"
        aria-label="Close"
        @click="emit('close')"
      >
        <CloseGlyph />
      </button>
    </header>

    <div class="max-h-[60vh] overflow-y-auto px-6 py-4">
      <div v-if="!imported" class="flex flex-col gap-3">
        <p class="text-small text-text">
          The space's home page becomes the project's blueprint and its description the project's. Every other page
          goes to the section of the concept it is about: architecture, decisions, API, deployment, operations,
          conventions, requirements or a glossary.
        </p>
        <AppCheckbox
          v-model="replace"
          label="Rewrite what the project already has"
          data-testid="confluence-replace"
        />
        <p class="text-caption text-text-subtle">
          Left off, a description or blueprint that holds text and a section with the same title stay as they are.
        </p>
      </div>
      <div v-else class="flex flex-col gap-3" data-testid="confluence-import-result">
        <p class="text-small text-text">
          Read {{ imported.pagesRead }} {{ imported.pagesRead === 1 ? 'page' : 'pages' }} of {{ imported.spaceName }}.
        </p>
        <ul class="flex flex-col gap-1 text-small text-text">
          <li>Description: {{ imported.descriptionWritten ? 'written' : 'kept' }}</li>
          <li>Blueprint: {{ imported.blueprintWritten ? 'written' : 'kept' }}</li>
        </ul>
        <ul v-if="imported.sections.length" class="divide-y divide-border">
          <li v-for="section in imported.sections" :key="section.title" class="flex items-center gap-3 py-2" data-testid="confluence-section">
            <span class="min-w-0 flex-1 truncate text-small text-text">{{ section.title }}</span>
            <span class="text-caption text-text-subtle">{{ section.pages }} {{ section.pages === 1 ? 'page' : 'pages' }}</span>
            <AppBadge :tone="section.outcome === 'kept' ? 'neutral' : 'safe'">{{ OUTCOME_LABEL[section.outcome] }}</AppBadge>
          </li>
        </ul>
        <p v-else class="text-small text-text-subtle">The space has no page beyond its home page.</p>
      </div>
    </div>

    <footer class="flex items-center justify-end gap-2 border-t border-border bg-canvas px-6 py-3.5">
      <AppButton size="sm" variant="ghost" @click="emit('close')">{{ imported ? 'Done' : 'Cancel' }}</AppButton>
      <AppButton v-if="!imported" size="sm" variant="primary" :loading="isRunning" data-testid="fill-project" @click="fill">
        Fill project
      </AppButton>
    </footer>
  </AppDialog>
</template>
