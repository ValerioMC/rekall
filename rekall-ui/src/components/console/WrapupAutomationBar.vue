<script setup lang="ts">
import { onUnmounted, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import AppCheckbox from '@/components/ui/AppCheckbox.vue'
import AppInput from '@/components/ui/AppInput.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import type { TaskId } from '@/model/branded'

/**
 * The standing form of a `/rk … wrapup` directive, set once on a task. It sits under the header
 * of both the description pane and the steps pane, because a task is worked from one or the
 * other and the setting has to be reachable from wherever the session is being driven.
 *
 * The toggle saves the moment it changes. The directive it reveals autosaves on a pause, the
 * way the description does. A blank directive is stored as none, and the server drops any
 * directive once the toggle is off, so a stale instruction never outlives the intent.
 */
const store = useConsoleStore()
const { selectedTask } = storeToRefs(store)
const { run } = useAsyncAction()

const enabled = ref(false)
const directive = ref('')
let saveTimer: ReturnType<typeof setTimeout> | null = null

function flush(taskId: TaskId): void {
  if (!saveTimer) return
  clearTimeout(saveTimer)
  saveTimer = null
  void run(() => store.saveTaskWrapup(taskId, enabled.value, directive.value))
}

function toggle(value: boolean): void {
  const task = selectedTask.value
  if (!task) return
  if (saveTimer) {
    clearTimeout(saveTimer)
    saveTimer = null
  }
  enabled.value = value
  void run(() => store.saveTaskWrapup(task.id, value, directive.value))
}

function scheduleDirectiveSave(): void {
  const task = selectedTask.value
  if (!task) return
  store.saveState = 'unsaved'
  if (saveTimer) clearTimeout(saveTimer)
  saveTimer = setTimeout(() => {
    saveTimer = null
    void run(() => store.saveTaskWrapup(task.id, enabled.value, directive.value))
  }, 700)
}

watch(
  () => selectedTask.value?.id ?? null,
  (_id, previousId) => {
    if (previousId) flush(previousId)
    enabled.value = selectedTask.value?.autoWrapup ?? false
    directive.value = selectedTask.value?.wrapupDirective ?? ''
  },
  { immediate: true }
)

watch(
  () => [selectedTask.value?.autoWrapup ?? false, selectedTask.value?.wrapupDirective ?? ''] as const,
  ([auto, stored]) => {
    if (saveTimer) return
    enabled.value = auto
    directive.value = stored
  }
)

onUnmounted(() => {
  if (selectedTask.value) flush(selectedTask.value.id)
})
</script>

<template>
  <div
    v-if="selectedTask"
    class="shrink-0 border-b border-border bg-surface px-5 py-2.5"
    data-testid="wrapup-automation"
  >
    <AppCheckbox
      :model-value="enabled"
      label="Generate the wrapup every session"
      data-testid="wrapup-auto-toggle"
      @update:model-value="toggle"
    />

    <div
      v-if="enabled"
      class="mt-2 flex flex-wrap items-center gap-x-2 gap-y-1.5"
      data-testid="wrapup-directive-field"
    >
      <span class="eyebrow shrink-0 text-[9.5px]">Directive</span>
      <AppInput
        v-model="directive"
        class="min-w-0 flex-1 basis-[240px]"
        placeholder="Optional: the words the wrapup should follow, e.g. export module only"
        aria-label="Standing wrapup directive"
        data-testid="wrapup-directive-input"
        @update:model-value="scheduleDirectiveSave"
      />
    </div>
  </div>
</template>
