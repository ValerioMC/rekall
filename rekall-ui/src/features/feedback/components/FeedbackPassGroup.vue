<script setup lang="ts">
import { ref } from 'vue'
import RevisionCard from '@/features/feedback/components/RevisionCard.vue'
import { kindLabel } from '@/features/feedback/lib/feedback-labels'
import type { Feedback } from '@/features/feedback/model/feedback'

/**
 * The revisions one Run revision took, under a header that says whether a session is still on them.
 * Open it for every revision in full; closed it keeps one line per revision with the first line of
 * the session's reply.
 */
const props = defineProps<{
  title: string
  detail: string
  entries: readonly Feedback[]
  running: boolean
  answered: number
  open: boolean
}>()
const emit = defineEmits<{ discard: [entry: Feedback] }>()

const expanded = ref(props.open)

function replyLine(entry: Feedback): string {
  return entry.replyMarkdown?.split('\n').find((line) => line.trim() !== '') ?? 'No reply yet'
}
</script>

<template>
  <section
    class="pass border-l pl-3"
    :class="running ? 'border-accent' : 'border-border'"
    :data-running="running"
    data-testid="feedback-pass"
  >
    <header>
      <button
        type="button"
        class="focus-ring flex w-full items-center gap-2 rounded-control py-1 text-left"
        :aria-expanded="expanded"
        data-testid="feedback-pass-toggle"
        @click="expanded = !expanded"
      >
        <span v-if="running" class="relative grid size-2 shrink-0 place-items-center" aria-hidden="true" data-testid="feedback-pass-live">
          <span class="absolute inline-flex size-2 animate-ping rounded-full bg-accent/60" />
          <span class="relative inline-flex size-1.5 rounded-full bg-accent" />
        </span>
        <span class="text-small font-medium text-text" data-testid="feedback-pass-title">{{ title }}</span>
        <span class="text-micro" :class="running ? 'font-medium text-accent' : 'text-text-subtle'" data-testid="feedback-pass-status">
          {{ running ? `${answered} of ${entries.length} answered` : 'Done' }}
        </span>
        <span v-if="detail" class="text-micro text-text-subtle">{{ detail }}</span>
        <span class="flex-1" />
        <span class="font-mono text-micro text-text-subtle">{{ entries.map((entry) => `#${entry.seq}`).join(' ') }}</span>
      </button>
    </header>

    <ol v-if="expanded" class="mt-1 flex flex-col gap-2 pb-2" aria-label="Revisions of the pass">
      <RevisionCard v-for="entry in entries" :key="entry.id" :entry="entry" @discard="emit('discard', $event)" />
    </ol>
    <ul v-else class="flex flex-col gap-0.5 pb-2" data-testid="feedback-pass-summary">
      <li v-for="entry in entries" :key="entry.id" class="flex gap-2 text-small text-text-subtle">
        <span class="shrink-0 font-mono text-micro">#{{ entry.seq }}</span>
        <span class="shrink-0 text-micro">{{ kindLabel(entry.kind) }}</span>
        <span class="min-w-0 truncate">{{ replyLine(entry) }}</span>
      </li>
    </ul>
  </section>
</template>
