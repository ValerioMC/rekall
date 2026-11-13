<script setup lang="ts">
import { computed, ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import { useQuickTask } from '@/features/catalog/composables/useQuickTask'
import { QUICK_TASK_BUSY_REASON, QUICK_TASK_HELP } from '@/features/catalog/lib/quick-task'
import type { Project } from '@/features/catalog/model/catalog'

/**
 * One field to start a small change: the text becomes a task, a session opens on it, and the task
 * is accepted by itself once the wrapup is written. Held back while a quick task is still being
 * worked on the project, which the server also refuses.
 */
const props = defineProps<{ project: Project }>()

const projectId = computed(() => props.project.id)
const quick = useQuickTask(projectId)

const text = ref('')
const hasText = computed(() => text.value.trim() !== '')
const hasFolder = computed(() => Boolean(props.project.repoFolder))
const reason = computed(() => {
  if (quick.working.value) return QUICK_TASK_BUSY_REASON
  if (!hasFolder.value) return "Set this project's folder on its page to open a session from it."
  return null
})
const canStart = computed(() => hasText.value && reason.value === null && !quick.busy.value)

async function start(): Promise<void> {
  if (!canStart.value) return
  const sent = text.value
  if (await quick.start(sent)) text.value = ''
}

function onEnter(event: KeyboardEvent): void {
  if (event.shiftKey || event.isComposing) return
  event.preventDefault()
  void start()
}
</script>

<template>
  <section class="field flex flex-col rounded-card" aria-label="Quick task" data-testid="quick-task-bar">
    <textarea
      v-model="text"
      class="w-full resize-none rounded-t-card bg-transparent px-4 pt-3 pb-1 font-sans text-body leading-relaxed text-text outline-none placeholder:text-text-subtle disabled:cursor-not-allowed disabled:opacity-50"
      rows="2"
      :disabled="reason !== null"
      :aria-describedby="`quick-task-help-${project.id}`"
      aria-label="What should Claude do?"
      placeholder="Fix the typo in the README"
      data-testid="quick-task-input"
      @keydown.enter="onEnter"
    />
    <div class="flex flex-wrap items-center gap-x-4 gap-y-2 px-3 pb-3 pl-4">
      <p
        :id="`quick-task-help-${project.id}`"
        class="min-w-0 flex-1 basis-64 text-caption"
        :class="reason === null ? 'text-text-subtle' : 'text-warn'"
        data-testid="quick-task-help"
      >
        {{ reason ?? QUICK_TASK_HELP }}
      </p>
      <AppButton
        class="shrink-0"
        variant="primary"
        :disabled="!canStart"
        :loading="quick.busy.value"
        data-testid="quick-task-start"
        @click="start"
      >
        Start quick task
      </AppButton>
    </div>
  </section>
</template>
