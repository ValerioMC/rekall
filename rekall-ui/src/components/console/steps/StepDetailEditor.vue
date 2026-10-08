<script setup lang="ts">
import AppInput from '@/components/ui/AppInput.vue'
import AppMarkdownEditor from '@/components/ui/AppMarkdownEditor.vue'
import AppModeToggle from '@/components/ui/AppModeToggle.vue'
import { useStepEditor } from '@/composables/useStepEditor'

/**
 * The title and markdown detail of the step the checklist has open, to write or to read, saved
 * as it is typed. A checklist step and a draft share it, each with its own wording.
 */
defineProps<{
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
        <div class="mt-2 min-w-0 overflow-hidden" :style="{ height: editorHeight }">
          <AppMarkdownEditor
            v-model="body"
            height="100%"
            :path-root="pathRoot"
            :placeholder="placeholder"
            :data-testid="`${testidPrefix}-detail-field`"
            @update:model-value="scheduleSave"
          />
        </div>
      </div>

      <div v-else-if="body.trim()" key="read" class="step-detail min-w-0 overflow-x-auto">
        <AppMarkdownEditor :model-value="body" readonly compact :data-testid="`${testidPrefix}-detail-read`" />
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
