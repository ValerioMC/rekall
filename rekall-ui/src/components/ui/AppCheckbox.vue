<script setup lang="ts">
import { useId } from 'vue'

const props = withDefaults(
  defineProps<{ modelValue: boolean; label: string; id?: string; describedBy?: string }>(),
  { id: undefined, describedBy: undefined }
)

const emit = defineEmits<{ 'update:modelValue': [value: boolean] }>()

// Falls back to its own id so the control is still labelled when used outside an AppField.
const fallbackId = useId()
const inputId = () => props.id ?? fallbackId
</script>

<template>
  <label
    :for="inputId()"
    class="inline-flex cursor-pointer select-none items-center gap-2.5 text-[13px] text-text"
  >
    <span class="relative flex size-4 items-center justify-center">
      <input
        :id="inputId()"
        type="checkbox"
        :checked="modelValue"
        :aria-describedby="describedBy"
        class="focus-ring peer size-4 cursor-pointer appearance-none rounded border border-border-strong bg-canvas transition-colors checked:border-accent checked:bg-accent"
        @change="emit('update:modelValue', ($event.target as HTMLInputElement).checked)"
      />
      <svg
        class="pointer-events-none absolute size-3 text-accent-ink opacity-0 transition-opacity peer-checked:opacity-100"
        viewBox="0 0 24 24"
        fill="none"
        aria-hidden="true"
      >
        <path d="M5 12.5l4.5 4.5L19 7.5" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" />
      </svg>
    </span>
    {{ label }}
  </label>
</template>
