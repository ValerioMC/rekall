<script setup lang="ts">
defineProps<{ label: string; active: boolean; alignEnd?: boolean }>()

defineEmits<{ sort: [] }>()
</script>

<template>
  <button
    type="button"
    class="focus-ring cursor-pointer hover:text-text"
    :class="[active ? 'text-text' : 'text-text-subtle', alignEnd ? 'text-right' : 'text-left']"
    :aria-pressed="active"
    @click="$emit('sort')"
  >
    {{ label }}
  </button>
</template>
