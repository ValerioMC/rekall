<script setup lang="ts">
import AppInput from '@/components/ui/AppInput.vue'
import AppMarkdownEditor from '@/components/ui/AppMarkdownEditor.vue'
import AppModeToggle from '@/components/ui/AppModeToggle.vue'
import { useSelectedProjectComponents } from '@/features/project/composables/useSelectedProjectComponents'
import { useSelectedProjectConnections } from '@/features/connection/composables/useSelectedProjectConnections'
import { useSelectedProjectTasks } from '@/features/catalog/composables/useSelectedProjectTasks'
import { useStepEditor } from '@/features/steps/composables/useStepEditor'

/**
 * The title and markdown detail of the step the checklist has open, to write or to read, saved
 * as it is typed. A checklist step and a draft share it, each with its own wording.
 */
defineProps<{
  /** Read flush inside a row, in a fixed-height writing surface; unset it reads and writes as a page that grows with its text. */
  compact?: boolean
  stepTitle: string
  /** `step` or `draft`: the prefix of each field's test id. */
  testidPrefix: string
  label: string
  editorHeight: string
  placeholder: string
  emptyText: string
  writePrompt: string
  pathRoot: string | null | undefined
}>()

const { mode, title, body, scheduleSave } = useStepEditor()
const projectComponents = useSelectedProjectComponents()
const projectConnections = useSelectedProjectConnections()
const projectTasks = useSelectedProjectTasks()
</script>

<template>
  <div>
    <div class="mb-2 flex items-center gap-2">
      <span class="eyebrow" :data-testid="`${testidPrefix}-detail-label`">{{ label }}</span>
      <span class="h-px flex-1 bg-border" aria-hidden="true" />
      <AppModeToggle v-model="mode" :testid-prefix="`${testidPrefix}-detail`" />
    </div>

    <Transition name="fade-quick" mode="out-in">
      <div v-if="mode === 'write'" key="write">
        <AppInput
          v-model="title"
          :aria-label="`Title of ${stepTitle}`"
          :data-testid="`${testidPrefix}-title-field`"
          @update:model-value="scheduleSave"
        />
        <div class="mt-2 min-w-0" :class="compact && 'overflow-hidden'" :style="compact ? { height: editorHeight } : undefined">
          <AppMarkdownEditor
            v-model="body"
            :height="compact ? '100%' : editorHeight"
            :grow="!compact"
            :path-root="pathRoot"
            :components="projectComponents"
            :connections="projectConnections"
            :tasks="projectTasks"
            :placeholder="placeholder"
            :data-testid="`${testidPrefix}-detail-field`"
            @update:model-value="scheduleSave"
          />
        </div>
      </div>

      <div v-else-if="body.trim()" key="read" class="min-w-0 overflow-x-auto" :class="compact ? 'step-detail' : 'step-reading'">
        <AppMarkdownEditor :model-value="body" :components="projectComponents" :connections="projectConnections" :tasks="projectTasks" readonly :compact="compact" :data-testid="`${testidPrefix}-detail-read`" />
      </div>

      <p v-else key="empty" class="py-1 text-small text-text-subtle">
        {{ emptyText }}
        <button class="focus-ring text-accent underline-offset-2 hover:underline" @click="mode = 'write'">
          {{ writePrompt }}
        </button>
      </p>
    </Transition>
  </div>
</template>

<style scoped>
/* The page holds a reading measure for its prose; a table or a code block takes the width of the pane. */
.step-reading :deep(.md-editor-preview) {
  max-width: none;
  margin-inline: 0 !important;
  padding: 0 0 8px !important;
}

.step-reading :deep(.md-editor-preview) > :where(p, ul, ol, blockquote, h1, h2, h3, h4) {
  max-width: 76ch;
}

.step-detail :deep(.md-editor-preview) h1,
.step-detail :deep(.md-editor-preview) h2 {
  font-size: var(--text-body);
  margin-top: 0.6em;
}

.step-detail :deep(.md-editor-preview) h3,
.step-detail :deep(.md-editor-preview) h4 {
  font-size: var(--text-body);
  margin-top: 0.6em;
}

.step-detail :deep(.md-editor-preview) > :first-child {
  margin-top: 0;
}

.step-detail :deep(.md-editor-preview) p,
.step-detail :deep(.md-editor-preview) ul,
.step-detail :deep(.md-editor-preview) ol {
  margin-block: 0.5em;
}
</style>
