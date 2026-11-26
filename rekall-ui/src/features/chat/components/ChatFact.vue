<script setup lang="ts">
/** One fact about a session: a labelled row in the column, the value alone in the narrow strip. */
defineProps<{ label: string }>()
</script>

<template>
  <div class="min-w-0 max-w-full @min-[1060px]/chat:border-b @min-[1060px]/chat:border-border @min-[1060px]/chat:py-3 @min-[1060px]/chat:last:border-b-0">
    <dt class="sr-only @min-[1060px]/chat:not-sr-only @min-[1060px]/chat:mb-1.5 @min-[1060px]/chat:text-caption @min-[1060px]/chat:text-text-subtle">
      {{ label }}
    </dt>
    <dd class="flex min-w-0 flex-wrap items-center gap-1.5">
      <slot />
    </dd>
  </div>
</template>
