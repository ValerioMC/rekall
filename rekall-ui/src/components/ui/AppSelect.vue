<script setup lang="ts">
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
</script>

<template>
  <div class="relative">
    <select
      :id="id"
      :value="modelValue ?? ''"
      :disabled="disabled"
      :aria-describedby="describedBy"
      class="focus-ring h-(--spacing-control) w-full cursor-pointer appearance-none rounded-[var(--radius-control)] border border-border bg-canvas pl-3 pr-9 text-[13px] text-text transition-colors hover:border-border-strong focus-visible:border-accent-deep disabled:cursor-not-allowed disabled:opacity-50"
      @change="emit('update:modelValue', ($event.target as HTMLSelectElement).value)"
    >
      <option v-if="placeholder" value="">{{ placeholder }}</option>
      <option v-for="option in options" :key="option.value" :value="option.value">
        {{ option.label }}
      </option>
    </select>
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
