<script setup lang="ts">
import { useId } from 'vue'

/**
 * The per-project pull request flow control. Whether the project can run it (a folder, a main
 * branch, a GitHub or GitLab task manager) is decided by the server; a refusal comes back as the
 * save's error.
 */
const props = withDefaults(defineProps<{ modelValue: boolean; saving?: boolean }>(), { saving: false })

const emit = defineEmits<{ 'update:modelValue': [value: boolean] }>()

const id = useId()
const descriptionId = `${id}-description`

function toggle(): void {
  if (props.saving) return
  emit('update:modelValue', !props.modelValue)
}
</script>

<template>
  <div class="flex items-start gap-3.5" data-testid="pull-request-flow">
    <button
      :id="id"
      type="button"
      role="switch"
      :aria-checked="modelValue"
      :aria-disabled="saving"
      :aria-describedby="descriptionId"
      class="focus-ring mt-0.5 h-5 w-9 shrink-0 rounded-full border transition-colors"
      :class="modelValue ? 'border-accent bg-accent-soft' : 'border-border bg-surface-raised'"
      data-testid="pull-request-flow-switch"
      @click="toggle"
    >
      <span
        class="block size-3.5 rounded-full transition-transform"
        :class="modelValue ? 'translate-x-[18px] bg-accent' : 'translate-x-[2px] bg-text-subtle'"
      />
    </button>
    <div class="min-w-0 flex-1">
      <label :for="id" class="block cursor-pointer text-body font-medium text-text">Merge through a pull request</label>
      <p :id="descriptionId" class="mt-1 max-w-[62ch] text-small leading-relaxed text-text-subtle">
        When a task's work is claimed its branch is pushed and a pull request (a merge request on GitLab) is opened
        against the main branch. Accepting the task merges it there. Needs the folder, a main branch and a GitHub
        or GitLab task manager.
      </p>
    </div>
  </div>
</template>
