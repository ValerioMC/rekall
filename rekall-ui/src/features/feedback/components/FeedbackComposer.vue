<script setup lang="ts">
import { ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppMarkdownEditor from '@/components/ui/AppMarkdownEditor.vue'
import { useFeedbackComposer } from '@/features/feedback/composables/useFeedbackComposer'
import { switchLabel, WRITTEN_KINDS } from '@/features/feedback/lib/feedback-labels'

/**
 * Where feedback is written: a markdown editor, what it is about, and one button. Once added, the
 * text lives only in the thread, so nothing stays in here.
 */
const { draft, kind, chooseKind, canSubmit, submit } = useFeedbackComposer()
const adding = ref(false)

async function add(): Promise<void> {
  if (!canSubmit.value || adding.value) return
  adding.value = true
  try {
    await submit()
  } finally {
    adding.value = false
  }
}
</script>

<template>
  <form
    class="rounded-card border border-border bg-surface p-3"
    data-testid="feedback-composer"
    @submit.prevent="add"
    @keydown.meta.enter.prevent="add"
    @keydown.ctrl.enter.prevent="add"
  >
    <div class="mb-2 flex items-center gap-2">
      <div class="flex gap-0.5 rounded-control bg-canvas p-0.5" role="group" aria-label="What the feedback is about">
        <button
          v-for="option in WRITTEN_KINDS"
          :key="option"
          type="button"
          class="focus-ring h-6 rounded-control px-2.5 text-caption transition-colors"
          :class="kind === option ? 'bg-surface-raised text-text' : 'text-text-subtle hover:text-text'"
          :aria-pressed="kind === option"
          :data-testid="`feedback-kind-${option}`"
          @click="chooseKind(option)"
        >
          {{ switchLabel(option) }}
        </button>
      </div>
      <span class="flex-1" />
      <span class="text-micro text-text-subtle">⌘↵ adds</span>
    </div>
    <AppMarkdownEditor v-model="draft" height="150px" placeholder="What should change?" compact :page-fullscreen="false" />
    <div class="mt-2 flex justify-end">
      <AppButton type="submit" variant="primary" size="sm" :disabled="!canSubmit" :loading="adding" data-testid="feedback-add">
        Add feedback
      </AppButton>
    </div>
  </form>
</template>
