<script setup lang="ts">
import { useSlots } from 'vue'

export type SelectOption = Readonly<{ value: string; label: string }>

withDefaults(
  defineProps<{
    modelValue: string | null
    options: readonly SelectOption[]
    id?: string
    describedBy?: string
    placeholder?: string
    disabled?: boolean
  }>(),
  { id: undefined, describedBy: undefined, placeholder: undefined, disabled: false }
)

const emit = defineEmits<{ 'update:modelValue': [value: string] }>()

// A `trailing` slot sits inside the field, right-aligned before the chevron, and describes the chosen option.
const slots = useSlots()
</script>

<template>
  <div class="relative">
    <select
      :id="id"
      :value="modelValue ?? ''"
      :disabled="disabled"
      :aria-describedby="describedBy"
      class="field text-text h-(--spacing-control) w-full cursor-pointer appearance-none rounded-[var(--radius-control)] pl-3 text-[13px]"
      :class="slots.trailing ? 'pr-28' : 'pr-9'"
      @change="emit('update:modelValue', ($event.target as HTMLSelectElement).value)"
    >
      <option v-if="placeholder" value="">{{ placeholder }}</option>
      <option v-for="option in options" :key="option.value" :value="option.value">
        {{ option.label }}
      </option>
    </select>
    <span
      v-if="slots.trailing"
      class="pointer-events-none absolute right-9 top-1/2 flex -translate-y-1/2 items-center"
    >
      <slot name="trailing" />
    </span>
    <svg
      class="pointer-events-none absolute right-3 top-1/2 size-3.5 -translate-y-1/2 text-text-subtle"
      viewBox="0 0 24 24"
      fill="none"
      aria-hidden="true"
    >
      <path d="M6 9l6 6 6-6" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
    </svg>
  </div>
</template>
