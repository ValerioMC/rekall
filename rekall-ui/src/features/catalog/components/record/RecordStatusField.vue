<script setup lang="ts" generic="Status extends string">
defineProps<{ options: readonly { value: Status; label: string }[] }>()

const status = defineModel<Status>({ required: true })
</script>

<template>
  <p class="mb-1.5 mt-5 eyebrow text-caption">Status</p>
  <div class="flex flex-wrap gap-1.5">
    <button
      v-for="option in options"
      :key="option.value"
      type="button"
      class="focus-ring h-8 rounded-control border px-3 text-small transition-colors"
      :class="
        status === option.value
          ? 'border-accent bg-accent-soft text-accent'
          : 'border-border-strong bg-canvas text-text-muted hover:border-text-subtle hover:text-text'
      "
      :aria-pressed="status === option.value"
      @click="status = option.value"
    >
      {{ option.label }}
    </button>
  </div>
</template>
