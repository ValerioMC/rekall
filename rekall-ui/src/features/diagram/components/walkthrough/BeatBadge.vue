<script setup lang="ts">
import { MOTION_LABEL, type BeatMotion } from '@/features/diagram/lib/walkthrough/walkthrough-beats'

defineProps<{ motion: BeatMotion }>()
</script>

<template>
  <span class="beat-badge rounded-key px-2 py-0.5 text-caption font-semibold" :class="`motion-${motion}`">{{ MOTION_LABEL[motion] }}</span>
</template>

<style scoped>
.beat-badge {
  color: var(--beat-tint);
  background: color-mix(in srgb, var(--beat-tint) 14%, transparent);
}

.motion-call,
.motion-self {
  --beat-tint: var(--color-accent);
}

.motion-reply {
  --beat-tint: var(--color-kind-data);
}
</style>
