<script setup lang="ts">
import CopyAnchorButton from '@/components/console/CopyAnchorButton.vue'
import AppButton from '@/components/ui/AppButton.vue'
import ReviewClaimBody from '@/features/review/components/ReviewClaimBody.vue'
import StepSeal from '@/features/steps/components/StepSeal.vue'
import { useReviewClaim } from '@/features/review/composables/useReviewClaim'
import { ageLabel } from '@/features/task/lib/age-label'
import { useNow } from '@/composables/useNow'
import type { ReviewItem } from '@/features/catalog/model/review'
import type { ClaimFactTone } from '@/features/review/lib/review-claim-facts'

/**
 * One claim in the feed: what is waiting, what was found on it, and the keys to decide it. The
 * open claim also carries the evidence; the others say it in a line, so no decision needs a second screen.
 */
const props = defineProps<{ item: ReviewItem; open: boolean }>()

defineEmits<{ open: []; accept: []; 'send-back': [] }>()

const MINUTE_MS = 60_000

const claim = useReviewClaim(() => props.item)
const now = useNow(MINUTE_MS)

const FACT_TONE: Readonly<Record<ClaimFactTone, string>> = {
  plain: 'text-text-muted',
  ok: 'text-text-muted',
  bad: 'text-danger',
  warn: 'text-warn'
}
const FACT_DOT: Readonly<Record<ClaimFactTone, string>> = {
  plain: 'bg-text-subtle',
  ok: 'bg-safe',
  bad: 'bg-danger',
  warn: 'bg-warn'
}
</script>

<template>
  <article
    v-if="claim.task.value"
    class="rounded-card border px-4 py-3 transition-colors duration-(--motion-quick)"
    :class="open ? 'border-border-strong bg-surface-raised shadow-raised' : 'border-border bg-surface hover:border-border-strong'"
    :data-open="open"
    data-testid="review-item"
  >
    <div class="grid grid-cols-[22px_minmax(0,1fr)_auto] items-start gap-x-3.5">
      <span class="mt-0.5 size-5.5" aria-hidden="true"><StepSeal state="CLAIMED" /></span>
      <div class="min-w-0">
        <button
          type="button"
          class="focus-ring block max-w-full rounded-key text-left text-title font-semibold text-text"
          :class="!open && 'hover:underline decoration-border-strong underline-offset-3'"
          :aria-expanded="open"
          data-testid="review-item-title"
          @click="$emit('open')"
        >
          {{ item.stepTitle ?? item.taskTitle }}
        </button>
        <p class="mt-1 flex flex-wrap items-center gap-x-2.5 gap-y-1 text-small text-text-muted">
          <span class="inline-flex items-center rounded-key border border-border bg-canvas pl-1.5 font-mono text-caption text-anchor">
            {{ item.anchor }}
            <CopyAnchorButton :anchor="item.anchor" class="!size-5" />
          </span>
          <span data-testid="review-item-line">{{ claim.line.value }}</span>
          <span class="text-text-subtle">{{ item.claimedAt ? `claimed ${ageLabel(item.claimedAt, now)} ago` : '' }}</span>
        </p>
        <ul class="mt-1.5 flex flex-wrap items-center gap-x-4.5 gap-y-1 text-small" data-testid="review-facts">
          <li v-for="fact in claim.facts.value" :key="fact.id" class="inline-flex items-center gap-1.5" :class="FACT_TONE[fact.tone]" :data-fact="fact.id">
            <span class="size-1.5 rounded-full" :class="FACT_DOT[fact.tone]" aria-hidden="true" />
            {{ fact.text }}
          </li>
        </ul>
      </div>
      <div class="flex items-center gap-2">
        <AppButton :variant="open ? 'secondary' : 'quiet'" data-testid="review-send-back" @click="$emit('send-back')">
          Send back
          <kbd v-if="open" class="font-mono text-micro text-text-subtle">S</kbd>
        </AppButton>
        <AppButton :variant="open ? 'primary' : 'quiet'" data-testid="review-accept" @click="$emit('accept')">
          {{ item.stepId ? 'Accept' : 'Accept task' }}
          <kbd v-if="open" class="font-mono text-micro">A</kbd>
        </AppButton>
      </div>
    </div>
    <ReviewClaimBody v-if="open" :task="claim.task.value" :claim="claim" />
  </article>
</template>
