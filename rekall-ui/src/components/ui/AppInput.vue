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
    class="field text-text h-(--spacing-control) w-full rounded-[var(--radius-control)] px-3 text-[13px]"
    :class="mono ? 'font-mono' : ''"
    @input="emit('update:modelValue', ($event.target as HTMLInputElement).value)"
  />
</template>
