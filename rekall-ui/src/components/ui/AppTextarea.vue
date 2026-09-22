<script setup lang="ts">
withDefaults(
  defineProps<{
    modelValue: string | null
    id?: string
    describedBy?: string
    placeholder?: string
    rows?: number
  }>(),
  { id: undefined, describedBy: undefined, placeholder: '', rows: 6 }
)

const emit = defineEmits<{ 'update:modelValue': [value: string] }>()
</script>

<template>
  <textarea
    :id="id"
    :value="modelValue ?? ''"
    :placeholder="placeholder"
    :rows="rows"
    :aria-describedby="describedBy"
    spellcheck="false"
    class="field text-text w-full resize-y rounded-[var(--radius-control)] p-3 font-mono text-[12.5px] leading-relaxed"
    @input="emit('update:modelValue', ($event.target as HTMLTextAreaElement).value)"
  />
</template>
