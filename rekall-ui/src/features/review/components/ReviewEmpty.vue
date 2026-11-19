<script setup lang="ts">
import { RouterLink } from 'vue-router'
import StepSeal from '@/features/steps/components/StepSeal.vue'

/** The queue is clear: the large done seal, stamped as it arrives. */
</script>

<template>
  <div class="grid min-h-0 flex-1 place-items-center px-6 text-center" data-testid="review-empty">
    <div class="flex flex-col items-center gap-4">
      <div class="size-21" aria-hidden="true"><StepSeal state="DONE" complete arriving /></div>
      <p class="text-display font-semibold text-text">Nothing waits for you.</p>
      <p class="text-body text-text-muted">Every claimed step is accepted. Resume a session to get more.</p>
      <RouterLink :to="{ name: 'work' }" class="focus-ring rounded-control border border-border-strong px-3 py-1.5 text-small text-text hover:bg-surface-raised">
        Back to work
      </RouterLink>
    </div>
  </div>
</template>
