<script setup lang="ts">
import { computed } from 'vue'
import { kindLabel } from '@/features/feedback/lib/feedback-labels'
import type { PassGroup } from '@/features/feedback/lib/feedback-passes'
import { isRunning, type Feedback } from '@/features/feedback/model/feedback'
import type { CommitReference } from '@/features/commits/model/commitReference'

/**
 * What the last pass did, for the person deciding whether to accept: the reply to each entry it
 * took, the entries it left unanswered, and the commits logged since it started. With no pass, the
 * session worked from the description alone, so only the commits are shown.
 */
const props = defineProps<{ pass: PassGroup | null; commits: readonly CommitReference[] }>()

const SHOWN_COMMITS = 5

const commitsOfPass = computed(() => {
  const since = props.pass?.takenAt ?? null
  return since === null ? props.commits : props.commits.filter((commit) => commit.createdAt >= since)
})
const shownCommits = computed(() => commitsOfPass.value.slice(0, SHOWN_COMMITS))
const hiddenCommits = computed(() => commitsOfPass.value.length - shownCommits.value.length)
const title = computed(() => (props.pass === null ? 'What the session did' : `What pass ${props.pass.pass} did`))

function replyLine(entry: Feedback): string {
  return entry.replyMarkdown?.split('\n').find((line) => line.trim() !== '') ?? ''
}

function firstLine(text: string): string {
  return text.split('\n').find((line) => line.trim() !== '') ?? ''
}
</script>

<template>
  <section class="mt-2.5 max-h-[30vh] overflow-y-auto border-t border-border pt-2" aria-label="What the last pass did" data-testid="pass-summary">
    <h3 class="mb-1 text-small font-medium text-text" data-testid="pass-summary-title">{{ title }}</h3>
    <ul v-if="pass" class="mb-1.5 flex flex-col gap-0.5" data-testid="pass-summary-entries">
      <li v-for="entry in pass.entries" :key="entry.id" class="flex gap-2 text-small" :data-state="entry.state" data-testid="pass-summary-entry">
        <span v-if="isRunning(entry)" class="shrink-0 text-micro font-medium text-danger" data-testid="pass-summary-no-reply">No reply</span>
        <span v-else class="shrink-0 text-accent" aria-label="Answered">✓</span>
        <span class="shrink-0 font-mono text-micro text-text-subtle">#{{ entry.seq }}</span>
        <span class="shrink-0 text-micro text-text-subtle">{{ kindLabel(entry.kind) }}</span>
        <span class="min-w-0 break-words text-text-muted" data-testid="pass-summary-reply">{{ replyLine(entry) }}</span>
      </li>
    </ul>
    <ul v-if="commitsOfPass.length > 0" class="flex flex-col gap-0.5" data-testid="pass-summary-commits">
      <li v-for="commit in shownCommits" :key="commit.id" class="flex gap-2 text-small" data-testid="pass-summary-commit">
        <span class="shrink-0 font-mono text-micro text-text-subtle">{{ commit.commitHash.slice(0, 7) }}</span>
        <span class="min-w-0 break-words text-text-muted">{{ firstLine(commit.comment) }}</span>
      </li>
      <li v-if="hiddenCommits > 0" class="text-micro text-text-subtle" data-testid="pass-summary-more">and {{ hiddenCommits }} more</li>
    </ul>
    <p v-else class="text-small text-text-subtle" data-testid="pass-summary-no-commits">No commit logged yet.</p>
  </section>
</template>
