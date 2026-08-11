<script setup lang="ts">
defineProps<{ title: string }>()
</script>

<template>
  <header class="glass sticky top-0 z-(--z-sticky) border-b border-border px-8 py-4">
    <div class="mx-auto flex max-w-[1240px] items-center gap-3">
      <slot name="back" />
      <h1 class="text-[20px] font-semibold tracking-[-0.01em] text-text">{{ title }}</h1>
      <slot name="title-suffix" />
      <div class="flex-1" />
      <slot name="actions" />
    </div>
  </header>
</template>
