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
  <section class="flex flex-col gap-1.5" aria-label="Quick task" data-testid="quick-task-bar">
    <div class="flex items-end gap-2">
      <textarea
        v-model="text"
        class="field min-w-0 flex-1 resize-none rounded-control px-3 py-2 font-sans text-body leading-relaxed"
        rows="2"
        :disabled="reason !== null"
        :aria-describedby="`quick-task-help-${project.id}`"
        aria-label="What should Claude do?"
        placeholder="Fix the typo in the README"
        data-testid="quick-task-input"
        @keydown.enter="onEnter"
      />
      <AppButton
        variant="primary"
        :disabled="!canStart"
        :loading="quick.busy.value"
        data-testid="quick-task-start"
        @click="start"
      >
        Start quick task
      </AppButton>
    </div>
    <p
      :id="`quick-task-help-${project.id}`"
      class="text-micro text-text-subtle"
      :class="{ 'text-warn': reason !== null }"
      data-testid="quick-task-help"
    >
      {{ reason ?? QUICK_TASK_HELP }}
    </p>
  </section>
</template>
