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
    class="focus-ring w-full resize-y rounded-[var(--radius-control)] border border-border bg-canvas p-3 font-mono text-[12.5px] leading-relaxed text-text transition-colors placeholder:text-text-subtle hover:border-border-strong focus-visible:border-accent-deep"
    @input="emit('update:modelValue', ($event.target as HTMLTextAreaElement).value)"
  />
</template>
