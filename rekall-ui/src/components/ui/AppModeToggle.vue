<script setup lang="ts">
withDefaults(defineProps<{ modelValue: 'write' | 'read'; testidPrefix?: string | null }>(), {
  testidPrefix: null
})

const emit = defineEmits<{ 'update:modelValue': ['write' | 'read'] }>()

const OPTIONS = ['write', 'read'] as const
</script>

<template>
  <div class="relative flex shrink-0 gap-0.5 rounded-control bg-surface p-0.5" role="group" aria-label="Mode">
    <span
      class="pointer-events-none absolute inset-y-0.5 left-0.5 w-[52px] rounded-control bg-surface-raised shadow-[0_1px_2px_rgb(0_0_0/0.4)] transition-transform duration-(--motion-move) ease-out"
      :style="{ transform: modelValue === 'read' ? 'translateX(52px)' : 'translateX(0)' }"
      aria-hidden="true"
    />
    <button
      v-for="option in OPTIONS"
      :key="option"
      type="button"
      class="focus-ring relative z-10 h-6 w-[52px] rounded-control text-caption capitalize transition-colors"
      :class="modelValue === option ? 'text-text' : 'text-text-subtle hover:text-text'"
      :aria-pressed="modelValue === option"
      :data-testid="testidPrefix ? `${testidPrefix}-${option}` : undefined"
      @click="emit('update:modelValue', option)"
    >
      {{ option }}
    </button>
  </div>
</template>
