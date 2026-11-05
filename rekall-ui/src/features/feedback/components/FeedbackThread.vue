<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import FeedbackComposer from '@/features/feedback/components/FeedbackComposer.vue'
import FeedbackEntry from '@/features/feedback/components/FeedbackEntry.vue'
import FeedbackPassGroup from '@/features/feedback/components/FeedbackPassGroup.vue'
import { answeredCount, groupByPass, passDetail } from '@/features/feedback/lib/feedback-passes'
import type { Feedback } from '@/features/feedback/model/feedback'

/**
 * The conversation under the description, read top to bottom as it happened: what was applied before
 * passes existed, each pass oldest first, then Next run. The composer is pinned under the scrolling
 * part, so a new message always lands just above where it was written.
 */
const props = defineProps<{ entries: readonly Feedback[] }>()
const emit = defineEmits<{
  edit: [entry: Feedback, bodyMarkdown: string]
  remove: [entry: Feedback]
  discard: [entry: Feedback]
}>()

const groups = computed(() => groupByPass(props.entries))
const composer = ref<InstanceType<typeof FeedbackComposer> | null>(null)
const scroller = ref<HTMLElement | null>(null)
const newestPass = computed(() => groups.value.passes.at(-1)?.pass ?? null)

function followUp(entry: Feedback): void {
  composer.value?.followUp(entry.seq)
}

async function showNewest(): Promise<void> {
  await nextTick()
  const area = scroller.value
  if (area) area.scrollTop = area.scrollHeight
}

onMounted(showNewest)
watch(() => props.entries.length, showNewest)
</script>

<template>
  <div class="flex min-h-0 flex-1 flex-col overflow-hidden" data-testid="feedback-thread">
    <div ref="scroller" class="flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto px-4 pb-3" data-testid="feedback-scroll">
      <FeedbackPassGroup
        v-if="groups.before.length > 0"
        title="Before passes"
        detail=""
        :entries="groups.before"
        :running="false"
        :answered="groups.before.length"
        :open="false"
        @edit="(target, body) => emit('edit', target, body)"
        @remove="emit('remove', $event)"
        @discard="emit('discard', $event)"
        @follow-up="followUp"
      />

      <FeedbackPassGroup
        v-for="group in groups.passes"
        :key="group.pass"
        :title="`Pass ${group.pass}`"
        :detail="passDetail(group)"
        :entries="group.entries"
        :running="group.running"
        :answered="answeredCount(group)"
        :open="group.pass === newestPass || group.running"
        @edit="(target, body) => emit('edit', target, body)"
        @remove="emit('remove', $event)"
        @discard="emit('discard', $event)"
        @follow-up="followUp"
      />

      <section aria-label="Next run" data-testid="feedback-next-run">
        <header class="mb-1 flex items-center gap-2">
          <h3 class="text-small font-medium text-text">Next run</h3>
          <span v-if="groups.next.length > 0" class="text-micro text-accent" data-testid="feedback-next-count">
            {{ groups.next.length }} waiting
          </span>
        </header>
        <ol v-if="groups.next.length > 0" class="flex flex-col gap-2" aria-label="Waiting entries">
          <FeedbackEntry
            v-for="entry in groups.next"
            :key="entry.id"
            :entry="entry"
            @edit="(target, body) => emit('edit', target, body)"
            @remove="emit('remove', $event)"
            @discard="emit('discard', $event)"
          />
        </ol>
        <p v-else class="text-small text-text-subtle" data-testid="feedback-empty">
          Nothing is waiting. Write below, then Run revision hands everything waiting to a session as one pass.
        </p>
      </section>
    </div>

    <div class="shrink-0 border-t border-border px-4 py-3">
      <FeedbackComposer ref="composer" />
    </div>
  </div>
</template>
