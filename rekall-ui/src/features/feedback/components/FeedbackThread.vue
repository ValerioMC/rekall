<script setup lang="ts">
import { computed, ref } from 'vue'
import FeedbackComposer from '@/features/feedback/components/FeedbackComposer.vue'
import FeedbackEntry from '@/features/feedback/components/FeedbackEntry.vue'
import FeedbackPassGroup from '@/features/feedback/components/FeedbackPassGroup.vue'
import { answeredCount, groupByPass, passDetail } from '@/features/feedback/lib/feedback-passes'
import type { Feedback } from '@/features/feedback/model/feedback'

/**
 * The thread under the description, read as groups: what waits for the next run, with the composer
 * under it, then each pass newest first, then what was applied before passes existed.
 */
const props = defineProps<{ entries: readonly Feedback[] }>()
const emit = defineEmits<{
  edit: [entry: Feedback, bodyMarkdown: string]
  remove: [entry: Feedback]
  discard: [entry: Feedback]
}>()

const groups = computed(() => groupByPass(props.entries))
const composer = ref<InstanceType<typeof FeedbackComposer> | null>(null)

function followUp(entry: Feedback): void {
  composer.value?.followUp(entry.seq)
}
</script>

<template>
  <div class="flex flex-col gap-4" data-testid="feedback-thread">
    <section aria-label="Next run" data-testid="feedback-next-run">
      <header class="mb-1 flex items-center gap-2">
        <h3 class="text-small font-medium text-text">Next run</h3>
        <span v-if="groups.next.length > 0" class="text-micro text-accent" data-testid="feedback-next-count">
          {{ groups.next.length }} waiting
        </span>
      </header>
      <ol v-if="groups.next.length > 0" class="mb-3 flex flex-col gap-2" aria-label="Waiting entries">
        <FeedbackEntry
          v-for="entry in groups.next"
          :key="entry.id"
          :entry="entry"
          @edit="(target, body) => emit('edit', target, body)"
          @remove="emit('remove', $event)"
          @discard="emit('discard', $event)"
        />
      </ol>
      <p v-else class="mb-3 text-small text-text-subtle" data-testid="feedback-empty">
        Nothing is waiting. Rewrite the description or write feedback below, then Run revision hands
        everything waiting to a session as one pass.
      </p>
      <FeedbackComposer ref="composer" />
    </section>

    <FeedbackPassGroup
      v-for="(group, index) in groups.passes"
      :key="group.pass"
      :title="`Pass ${group.pass}`"
      :detail="passDetail(group)"
      :entries="group.entries"
      :running="group.running"
      :answered="answeredCount(group)"
      :open="index === 0 || group.running"
      @edit="(target, body) => emit('edit', target, body)"
      @remove="emit('remove', $event)"
      @discard="emit('discard', $event)"
      @follow-up="followUp"
    />

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
  </div>
</template>
