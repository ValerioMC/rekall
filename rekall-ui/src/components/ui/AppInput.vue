<script setup lang="ts">
withDefaults(
  defineProps<{
    modelValue: string | number | null
    id?: string
    describedBy?: string
    type?: 'text' | 'number' | 'date' | 'search'
    placeholder?: string
    mono?: boolean
    disabled?: boolean
    /** Mobile keyboard hint. Postgres lengths and precisions are numeric fields. */
    inputmode?: 'text' | 'numeric' | 'search'
  }>(),
  {
    id: undefined,
    describedBy: undefined,
    inputmode: undefined,
    type: 'text',
    placeholder: '',
    mono: false,
    disabled: false
  }
)

const emit = defineEmits<{ 'update:modelValue': [value: string] }>()
</script>

<template>
  <input
    :id="id"
    :value="modelValue ?? ''"
    :type="type"
    :inputmode="inputmode ?? (type === 'number' ? 'numeric' : undefined)"
    :placeholder="placeholder"
    :disabled="disabled"
    :aria-describedby="describedBy"
    class="focus-ring h-(--spacing-control) w-full rounded-[var(--radius-control)] border border-border bg-canvas px-3 text-[13px] text-text transition-colors placeholder:text-text-subtle hover:border-border-strong focus-visible:border-accent-deep disabled:cursor-not-allowed disabled:opacity-50"
    :class="mono ? 'font-mono' : ''"
    @input="emit('update:modelValue', ($event.target as HTMLInputElement).value)"
  />
</template>
