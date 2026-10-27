<script setup lang="ts">
import { computed } from 'vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppMarkdownEditor from '@/components/ui/AppMarkdownEditor.vue'
import AppModeToggle from '@/components/ui/AppModeToggle.vue'
import MemoryTypeChip from './MemoryTypeChip.vue'
import { useEditorMode } from '@/composables/useEditorMode'
import { resourceKindLabel, type ResourceFile } from '@/features/claude-resources/model/claude-resources'

const props = defineProps<{
  file: ResourceFile
  draft: string
  dirty: boolean
  saving: boolean
  conflict: boolean
}>()

const emit = defineEmits<{
  'update:draft': [body: string]
  save: []
  revert: []
  remove: []
  reload: []
  overwrite: []
}>()

const { mode } = useEditorMode('write')
const summary = computed(() => props.file.summary)
const readOnly = computed(() => !summary.value.editable)
/** Who owns a read-only file, so the lock says more than "no". */
const sourceLabel = computed(() => {
  const source = summary.value.source
  if (!source || source.kind === 'own') return null
  return `Provided by ${source.name}${source.detail ? ` · ${source.detail}` : ''}`
})
const deletable = computed(() => summary.value.editable && summary.value.kind !== 'instructions' && summary.value.kind !== 'memoryIndex')
</script>

<template>
  <section class="flex min-h-0 min-w-0 flex-1 flex-col bg-canvas" aria-label="Open file">
    <header class="texture-dots flex shrink-0 items-start gap-3.5 border-b border-border px-5 py-3.5">
      <div class="min-w-0 flex-1">
        <div class="flex min-w-0 items-center gap-2">
          <h2 class="truncate text-heading font-semibold tracking-[-0.015em] text-text" data-testid="resource-title">{{ summary.name }}</h2>
          <AppBadge tone="neutral">{{ resourceKindLabel(summary.kind) }}</AppBadge>
          <MemoryTypeChip :type="summary.memoryType" />
          <AppBadge v-if="readOnly" tone="filed" :title="sourceLabel ?? undefined">read only</AppBadge>
        </div>
        <p class="mt-1 truncate font-mono text-caption text-text-subtle" :title="summary.path">~/.claude/{{ summary.path }}</p>
        <p v-if="sourceLabel" class="mt-0.5 truncate text-caption text-text-subtle" data-testid="resource-source">{{ sourceLabel }}. It is rewritten when the provider updates, so it cannot be edited here.</p>
      </div>

      <div class="flex shrink-0 items-center gap-2">
        <span v-if="dirty" class="flex items-center gap-1.5 text-caption text-warn" role="status" data-testid="resource-dirty">
          <span class="size-1.5 rounded-full bg-warn" aria-hidden="true" />
          Unsaved
        </span>
        <AppButton v-if="dirty" size="md" variant="ghost" data-testid="resource-revert" @click="emit('revert')">Revert</AppButton>
        <AppButton
          size="md"
          variant="primary"
          :disabled="!dirty || readOnly"
          :loading="saving"
          data-testid="resource-save"
          title="Save (⌘S)"
          @click="emit('save')"
        >
          Save
        </AppButton>
        <AppButton v-if="deletable" size="md" variant="danger-quiet" data-testid="resource-delete" aria-label="Delete this file" @click="emit('remove')">
          <svg class="size-3.5" viewBox="0 0 16 16" fill="none" aria-hidden="true">
            <path d="M2.5 4.5h11M6 4.5V3h4v1.5M4 4.5l.7 8.6a1 1 0 0 0 1 .9h4.6a1 1 0 0 0 1-.9l.7-8.6" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
          Delete
        </AppButton>
      </div>
    </header>

    <div
      v-if="conflict"
      class="flex shrink-0 items-center gap-3 border-b border-warn/40 bg-warn-soft px-5 py-2.5 text-small text-warn"
      role="alert"
      data-testid="resource-conflict"
    >
      <span class="min-w-0 flex-1">This file changed on disk while you were editing it, most likely a session. Your text is still here.</span>
      <AppButton size="sm" variant="secondary" data-testid="resource-conflict-reload" @click="emit('reload')">Load the disk version</AppButton>
      <AppButton size="sm" variant="danger" data-testid="resource-conflict-overwrite" @click="emit('overwrite')">Overwrite it</AppButton>
    </div>

    <div class="flex shrink-0 items-center gap-2 border-b border-border px-5 py-1.5">
      <p v-if="summary.description" class="min-w-0 flex-1 truncate text-small text-text-subtle">{{ summary.description }}</p>
      <AppModeToggle v-model="mode" class="ml-auto" />
    </div>

    <div class="min-h-0 flex-1 overflow-y-auto p-4">
      <AppMarkdownEditor
        v-if="mode === 'write' && !readOnly"
        :key="`write-${summary.path}`"
        :model-value="draft"
        height="100%"
        @update:model-value="emit('update:draft', $event)"
      />
      <AppMarkdownEditor v-else :key="`read-${summary.path}`" :model-value="draft" readonly />
    </div>
  </section>
</template>
