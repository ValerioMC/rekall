<script setup lang="ts">
import { computed } from 'vue'
import FeedbackPassGroup from '@/features/feedback/components/FeedbackPassGroup.vue'
import RevisionCard from '@/features/feedback/components/RevisionCard.vue'
import { answeredCount, groupByPass, passDetail } from '@/features/feedback/lib/feedback-passes'
import type { Feedback } from '@/features/feedback/model/feedback'

/**
 * Every revision of the description, newest first: what waits for the next run, then each pass,
 * then what was applied before passes existed. Revisions are made by editing the description, so
 * the only thing to do here is discard a waiting one.
 */
const props = defineProps<{ entries: readonly Feedback[] }>()
const emit = defineEmits<{ discard: [entry: Feedback] }>()

const groups = computed(() => groupByPass(props.entries))
const passesNewestFirst = computed(() => [...groups.value.passes].reverse())
const newestPass = computed(() => passesNewestFirst.value[0]?.pass ?? null)
</script>

<template>
  <div class="flex flex-col gap-4" data-testid="revisions-panel">
    <p v-if="entries.length === 0" class="max-w-prose text-body text-text-muted" data-testid="revisions-empty">
      No revisions yet. Once a task has been worked, every change to its description is kept here with
      what it was applied to and what the session answered.
    </p>

    <section v-if="groups.next.length > 0" aria-label="Waiting for the next run" data-testid="revisions-waiting">
      <h3 class="mb-1 text-small font-medium text-text">Waiting for the next run</h3>
      <ol class="flex flex-col gap-2" aria-label="Waiting revisions">
        <RevisionCard v-for="entry in groups.next" :key="entry.id" :entry="entry" @discard="emit('discard', $event)" />
      </ol>
    </section>

    <FeedbackPassGroup
      v-for="group in passesNewestFirst"
      :key="group.pass"
      :title="`Pass ${group.pass}`"
      :detail="passDetail(group)"
      :entries="group.entries"
      :running="group.running"
      :answered="answeredCount(group)"
      :open="group.pass === newestPass || group.running"
      @discard="emit('discard', $event)"
    />

    <FeedbackPassGroup
      v-if="groups.before.length > 0"
      title="Before passes"
      detail=""
      :entries="groups.before"
      :running="false"
      :answered="groups.before.length"
      :open="false"
      @discard="emit('discard', $event)"
    />
  </div>
</template>
