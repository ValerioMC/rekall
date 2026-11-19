<script setup lang="ts">
import { computed } from 'vue'
import BeatBadge from '@/features/diagram/components/walkthrough/BeatBadge.vue'
import WalkthroughEnds from '@/features/diagram/components/walkthrough/WalkthroughEnds.vue'
import { baseOf, folderOf, spanLabel } from '@/features/diagram/lib/source-label'
import type { WalkthroughBeat } from '@/features/diagram/lib/walkthrough/walkthrough-beats'
import type { SourceLocation } from '@/features/diagram/model/diagram'

/**
 * What happens at the current beat: who calls whom with which method, why, what goes in and
 * what comes back. A call that has a sheet of its own offers to step into it, and one linked to
 * code offers to open it.
 */
const props = defineProps<{
  beat: WalkthroughBeat
  position: number
  count: number
  labels: ReadonlyMap<string, string>
  external: ReadonlySet<string>
  sources: readonly SourceLocation[]
  /** The title of the sheet this call opens, when it opens one. */
  opensTitle: string | null
}>()

const emit = defineEmits<{ stepInto: []; openCode: [] }>()

const receivesExternal = computed(() => props.beat.motion === 'call' && props.external.has(props.beat.message.to))
const firstSource = computed(() => props.sources[0] ?? null)
/** A reply shows only what it carries back; a call what it receives and what it gives back. */
const values = computed(() => {
  const { motion, input, output } = props.beat
  const shown = motion === 'reply' ? [{ name: 'Returned', text: output }] : [{ name: 'Input', text: input }, { name: 'Output', text: output }]
  return shown.filter((value): value is { name: string; text: string } => value.text !== null)
})
</script>

<template>
  <aside
    class="glass pointer-events-auto flex max-h-full w-[min(420px,calc(100vw-2rem))] flex-col overflow-hidden rounded-card border border-border shadow-lift"
    aria-labelledby="walkthrough-card-title"
    aria-live="polite"
    data-testid="walkthrough-card"
  >
    <header class="shrink-0 border-b border-border px-4 py-3">
      <p class="flex items-center gap-2 text-caption">
        <BeatBadge :motion="beat.motion" />
        <span class="font-mono tabular-nums text-text-subtle">Step {{ position + 1 }} of {{ count }}</span>
      </p>
      <h3 id="walkthrough-card-title" class="mt-2 break-words font-mono text-body font-semibold leading-snug text-text">{{ beat.message.text || '(no text)' }}</h3>
      <WalkthroughEnds class="mt-2" :message="beat.message" :labels="labels" :external="external" />
    </header>

    <div class="flex min-h-0 flex-col gap-4 overflow-y-auto px-4 pb-5 pt-4">
      <p v-if="beat.explain" class="text-body leading-relaxed text-text" data-testid="walkthrough-explain">{{ beat.explain }}</p>
      <p v-if="receivesExternal" class="rounded-control border border-dashed border-border px-3 py-2 text-small leading-relaxed text-text-muted">
        This call leaves the project's code: what happens inside {{ labels.get(beat.message.to) ?? beat.message.to }} is not shown.
      </p>

      <dl v-if="values.length > 0" class="grid gap-2" data-testid="walkthrough-values">
        <div v-for="value in values" :key="value.name">
          <dt class="section-label mb-1">{{ value.name }}</dt>
          <dd class="max-h-[140px] overflow-auto whitespace-pre-wrap break-words rounded-control border border-border bg-canvas px-3 py-2 font-mono text-caption leading-relaxed text-text">{{ value.text }}</dd>
        </div>
      </dl>

      <button
        v-if="beat.opens !== null"
        class="step-into focus-ring flex w-full items-center gap-3 rounded-control border px-3 py-2.5 text-left transition-colors"
        data-testid="walkthrough-step-into"
        @click="emit('stepInto')"
      >
        <svg viewBox="0 0 16 16" class="size-4 shrink-0" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M4 3v6.5h8M9 6.5l3 3-3 3" /></svg>
        <span class="min-w-0 flex-1">
          <span class="block text-small font-semibold">Step into this call</span>
          <span v-if="opensTitle" class="block truncate text-caption opacity-80">{{ opensTitle }}</span>
        </span>
        <kbd>↵</kbd>
      </button>

      <section aria-labelledby="walkthrough-code">
        <h4 id="walkthrough-code" class="section-label mb-2">Code</h4>
        <p v-if="firstSource === null" class="rounded-control border border-dashed border-border px-3 py-2.5 text-small leading-relaxed text-text-subtle">Not linked to a place in the code.</p>
        <button
          v-else
          class="focus-ring block w-full overflow-hidden rounded-control border border-border bg-canvas text-left transition-colors hover:border-border-field"
          aria-haspopup="dialog"
          title="Open the implementation (C)"
          data-testid="walkthrough-open-code"
          @click="emit('openCode')"
        >
          <span class="flex items-center gap-2 border-b border-border bg-surface-raised px-2.5 py-1.5 font-mono text-caption">
            <span class="flex min-w-0 flex-1" :title="firstSource.file">
              <span class="truncate text-text-subtle">{{ folderOf(firstSource.file) }}</span><span class="shrink-0 text-text">{{ baseOf(firstSource.file) }}</span>
            </span>
            <span class="shrink-0 tabular-nums text-accent-strong">{{ spanLabel(firstSource) }}</span>
            <kbd>C</kbd>
          </span>
          <span v-if="firstSource.symbol" class="block truncate px-2.5 py-2 font-mono text-caption text-text-muted">{{ firstSource.symbol }}</span>
        </button>
      </section>
    </div>
  </aside>
</template>

<style scoped>
.step-into {
  color: var(--color-accent-strong);
  border-color: color-mix(in srgb, var(--color-accent) 45%, transparent);
  background: color-mix(in srgb, var(--color-accent) 8%, transparent);
}

.step-into:hover {
  background: color-mix(in srgb, var(--color-accent) 14%, transparent);
}
</style>
