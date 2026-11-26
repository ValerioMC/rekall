<script setup lang="ts">
defineProps<{ label: string; fieldId: string }>()
</script>

<template>
  <div class="border-t border-border px-5 py-3.5">
    <label :for="fieldId" class="mb-2 block text-body font-medium text-text">{{ label }}</label>
    <slot name="control" :described-by="`${fieldId}-hint`" />
    <div :id="`${fieldId}-hint`" class="mt-1.5 text-small leading-relaxed text-text-subtle">
      <slot />
    </div>
  </div>
</template>
