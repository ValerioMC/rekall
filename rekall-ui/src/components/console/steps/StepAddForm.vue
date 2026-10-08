<script setup lang="ts">
import AppButton from '@/components/ui/AppButton.vue'
import AppInput from '@/components/ui/AppInput.vue'

/** The line at the foot of the checklist that names a new step, which starts as a draft. */
const title = defineModel<string>({ required: true })

defineEmits<{ submit: [] }>()
</script>

<template>
  <form
    class="dock-lane-safe flex shrink-0 items-center gap-2 border-t border-border bg-surface py-3 pl-5"
    @submit.prevent="$emit('submit')"
  >
    <span
      class="grid size-[22px] shrink-0 place-items-center rounded-full border border-dashed border-border-strong text-text-subtle"
      aria-hidden="true"
    >
      <svg class="size-3" viewBox="0 0 12 12" fill="none">
        <path d="M6 2.4v7.2M2.4 6h7.2" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
      </svg>
    </span>
    <AppInput
      id="new-step"
      v-model="title"
      placeholder="Name the next piece of this task. It starts as a draft."
      data-testid="new-step"
    />
    <AppButton
      variant="primary"
      type="submit"
      :disabled="!title.trim()"
      data-testid="add-step"
    >
      Add draft
    </AppButton>
  </form>
</template>
