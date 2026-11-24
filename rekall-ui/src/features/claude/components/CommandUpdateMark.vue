<script setup lang="ts">
import StepSeal from '@/features/steps/components/StepSeal.vue'
</script>

<template>
  <span class="inline-flex items-center gap-2" data-testid="command-update-mark">
    <span class="size-4 shrink-0" aria-hidden="true"><StepSeal state="RUNNING" /></span>
    <span class="whitespace-nowrap">rk commands updating</span>
  </span>
</template>
