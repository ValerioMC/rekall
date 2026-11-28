<script setup lang="ts">
defineProps<{ label: string; active: boolean; descending: boolean; alignEnd?: boolean }>()

defineEmits<{ sort: [] }>()
</script>

<template>
  <button
    type="button"
    class="group/sort focus-ring flex h-7 min-w-0 cursor-pointer items-center gap-1 whitespace-nowrap rounded-key hover:text-text"
    :class="[active ? 'text-text' : 'text-text-subtle', alignEnd && 'flex-row-reverse']"
    :aria-pressed="active"
    :data-direction="active ? (descending ? 'descending' : 'ascending') : undefined"
    @click="$emit('sort')"
  >
    {{ label }}
    <svg
      class="size-3 shrink-0 transition-[opacity,transform] duration-(--motion-quick)"
      :class="[active ? 'opacity-100' : 'opacity-0 group-hover/sort:opacity-50', active && !descending && 'rotate-180']"
      viewBox="0 0 12 12"
      fill="none"
      aria-hidden="true"
    >
      <path d="M6 2.5v7M3 6.5l3 3 3-3" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
    </svg>
  </button>
</template>
