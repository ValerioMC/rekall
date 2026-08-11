<script setup lang="ts">
import { useId } from 'vue'

withDefaults(
  defineProps<{ label: string; hint?: string; required?: boolean; error?: string | null }>(),
  { hint: undefined, required: false, error: null }
)

/**
 * The field owns the id and hands it to whatever control fills its slot.
 *
 * A label that is merely next to an input is not associated with it: clicking it does nothing
 * and a screen reader announces the control as unlabelled. Generating the id here is what
 * makes `for` and `id` agree without every call site having to invent a unique string.
 */
const id = useId()
const describedBy = `${id}-description`
</script>

<template>
  <div class="mb-4">
    <label :for="id" class="mb-1.5 flex items-center gap-1 text-xs font-medium text-text-muted">
      {{ label }}
      <span v-if="required" class="text-accent" aria-hidden="true">*</span>
      <span v-if="required" class="sr-only">required</span>
    </label>

    <slot :field-id="id" :described-by="hint || error ? describedBy : undefined" />

    <p v-if="error" :id="describedBy" class="mt-1.5 text-[11.5px] text-danger" role="alert">
      {{ error }}
    </p>
    <p v-else-if="hint" :id="describedBy" class="mt-1.5 text-[11.5px] text-text-subtle">{{ hint }}</p>
  </div>
</template>
