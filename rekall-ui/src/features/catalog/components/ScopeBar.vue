<script setup lang="ts">
defineSlots<{ switcher(): unknown; meta(): unknown }>()
</script>

<template>
  <div class="flex min-h-10 items-center gap-3 pt-5" data-testid="scope-bar">
    <div class="min-w-0"><slot name="switcher" /></div>
    <div class="flex min-w-0 items-center gap-2.5 text-small text-text-subtle"><slot name="meta" /></div>
  </div>
</template>
