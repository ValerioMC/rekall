<script setup lang="ts">
import { computed } from 'vue'
import KindGlyph from '@/components/diagram/canvas/KindGlyph.vue'
import { KIND_INFO } from '@/common/diagram/kinds'
import { formatDuration } from '@/common/format/duration'
import { useNow } from '@/composables/useNow'
import type { PendingGeneration } from '@/stores/diagram.store'

/**
 * A diagram a session is drawing now: what was asked, as which types, for how long, and the
 * session itself one click away. Once the session ends without delivering, it says so.
 */
const props = defineProps<{ generation: PendingGeneration; anchors: string; ended: boolean }>()

const emit = defineEmits<{ watch: []; dismiss: [] }>()

const now = useNow(1000)
const elapsed = computed(() => formatDuration(Math.max(0, (now.value - props.generation.startedAt) / 1000)))
</script>

<template>
  <article
    class="flex min-w-0 flex-col gap-2 rounded-card border border-dashed px-3.5 py-3"
    :class="ended ? 'border-border bg-surface' : 'border-accent/40 bg-accent-soft/40'"
    :aria-busy="!ended"
    data-testid="generation-card"
  >
    <header class="flex items-center gap-2">
      <span v-if="!ended" class="drawing-progress relative h-1 w-10 overflow-hidden rounded-full bg-accent/20" aria-hidden="true" />
      <span class="text-caption font-semibold" :class="ended ? 'text-text-muted' : 'text-accent'" role="status">
        {{ ended ? 'Session ended without a diagram' : 'Being drawn' }}
      </span>
      <span class="ml-auto font-mono text-micro tabular-nums text-text-subtle">{{ elapsed }}</span>
    </header>
    <p class="line-clamp-3 text-small leading-snug text-text">{{ generation.request }}</p>
    <div class="flex flex-wrap items-center gap-1.5">
      <span v-for="kind in generation.kinds" :key="kind" class="inline-flex items-center gap-1 rounded-full border border-border bg-canvas px-1.5 py-0.5 text-micro text-text-muted">
        <KindGlyph :kind="kind" :size="12" />
        {{ KIND_INFO[kind].label }}
      </span>
      <span class="anchor-chip truncate px-1.5 py-0.5 text-micro">{{ anchors }}</span>
    </div>
    <footer class="mt-1 flex justify-end gap-1.5">
      <button class="key-quiet focus-ring inline-flex h-7 items-center rounded-key px-2.5 text-caption" data-testid="generation-dismiss" @click="emit('dismiss')">Dismiss</button>
      <button v-if="!ended" class="key-gold focus-ring inline-flex h-7 items-center rounded-key px-2.5 text-caption font-medium" data-testid="generation-watch" @click="emit('watch')">Watch session</button>
    </footer>
  </article>
</template>

<style scoped>
.drawing-progress::after {
  content: '';
  position: absolute;
  inset: 0;
  width: 40%;
  border-radius: inherit;
  background: var(--color-accent);
  animation: drawing-progress 1.4s ease-in-out infinite;
}

@keyframes drawing-progress {
  from {
    transform: translateX(-100%);
  }
  to {
    transform: translateX(250%);
  }
}

@media (prefers-reduced-motion: reduce) {
  .drawing-progress::after {
    animation: none;
    width: 100%;
    opacity: 0.6;
  }
}
</style>
