<script setup lang="ts">
import { computed } from 'vue'
import DecisionFoundLine from '@/features/decisions/components/DecisionFoundLine.vue'
import DecisionFoundText from '@/features/decisions/components/DecisionFoundText.vue'
import DecisionSeal from '@/features/decisions/components/DecisionSeal.vue'
import DecisionSourceMark from '@/features/decisions/components/DecisionSourceMark.vue'
import { askedBy, discussedIn, readWhen, scopeLabel, SOURCE_LABEL, timeOf } from '@/features/decisions/lib/decision-labels'
import type { DecisionFind } from '@/features/decisions/lib/decision-search'
import type { Decision } from '@/features/decisions/model/decision'

/**
 * One decision as a row: its state, the question, the answer as it stands, and where it was taken.
 * Opened, it shows the conversation, the options and why the question was asked.
 */
const props = withDefaults(
  defineProps<{ decision: Decision; hasSteps: boolean; opened: boolean; fresh?: boolean; query?: string; find?: DecisionFind }>(),
  { fresh: false, query: '', find: undefined }
)
const emit = defineEmits<{ toggle: [] }>()

const unseen = computed(() => (props.find?.elsewhere ?? []).filter((found) => !props.opened || !found.shownWhenOpened))

const TAG = 'inline-flex h-[18px] items-center rounded-key border px-1.5 text-micro font-medium'
</script>

<template>
  <div
    class="rounded-control"
    :class="[opened ? 'my-1 bg-surface ring-1 ring-accent/35' : '', fresh ? 'decision-fresh' : '']"
    data-testid="decision-row"
    :data-key="decision.key"
  >
    <button
      type="button"
      class="focus-ring grid w-full cursor-pointer grid-cols-[28px_minmax(0,5fr)_minmax(0,4fr)_minmax(0,11.5rem)] items-start gap-x-3.5 gap-y-1.5 rounded-control px-2 py-2.5 text-left"
      :class="opened ? '' : 'hover:bg-surface-hover'"
      :aria-expanded="opened"
      data-testid="decision-toggle"
      @click="emit('toggle')"
    >
      <DecisionSeal :state="decision.state" :opened="opened" />
      <span class="min-w-0">
        <span class="block text-body leading-[1.35] text-text text-pretty"><DecisionFoundText :text="decision.question" :query="query" /></span>
        <span class="mt-0.5 flex flex-wrap items-center gap-x-2.5 gap-y-0.5 text-caption text-text-subtle">
          <span>{{ askedBy(decision) }}</span>
          <span v-if="hasSteps" :class="TAG" class="border-border-strong text-text-muted" data-testid="decision-scope">{{ scopeLabel(decision) }}</span>
        </span>
      </span>
      <span class="min-w-0 text-body leading-[1.35] text-text">
        <template v-if="decision.state === 'OPEN'">
          <span class="block text-text-subtle">Still being discussed in {{ discussedIn(decision.source) }}</span>
          <span v-if="decision.options.length > 0" class="mt-0.5 block text-caption text-text-subtle" data-testid="decision-offered">
            Offered:
            <template v-for="(option, index) in decision.options" :key="option.label">
              <template v-if="index > 0"> · </template>
              <b v-if="option.recommended" class="font-medium text-text-muted"><DecisionFoundText :text="option.label" :query="query" /></b>
              <DecisionFoundText v-else :text="option.label" :query="query" />
            </template>
          </span>
        </template>
        <span v-else-if="decision.state === 'ON_HOLD'" class="block text-warn" data-testid="decision-hold">On hold: <DecisionFoundText :text="decision.holdNote ?? ''" :query="query" /></span>
        <template v-else>
          <span data-testid="decision-answer"><DecisionFoundText :text="decision.answer ?? ''" :query="query" /></span>
          <span v-if="decision.previousAnswer !== null" class="ml-1.5 text-caption text-text-subtle" data-testid="decision-was">
            changed{{ decision.changedAt === null ? '' : ` ${timeOf(decision.changedAt)}` }}, was <DecisionFoundText :text="decision.previousAnswer" :query="query" />
          </span>
          <span v-for="detail in decision.details" :key="detail.addedAt + detail.text" class="mt-0.5 block text-small text-text-muted" data-testid="decision-detail">
            <time class="mr-1.5 font-mono text-caption text-text-subtle">{{ timeOf(detail.addedAt) }}</time><DecisionFoundText :text="detail.text" :query="query" />
          </span>
        </template>
      </span>
      <span class="flex justify-end text-caption whitespace-nowrap">
        <DecisionSourceMark :source="decision.source" />
      </span>
      <DecisionFoundLine v-if="find && unseen.length > 0" :places="unseen" :in-row="find.inRow" :query="query" />
    </button>

    <div
      v-if="opened"
      class="grid grid-cols-1 gap-x-10 gap-y-5 pb-4 pl-[50px] pr-4 pt-1 @4xl/task:grid-cols-[minmax(0,1fr)_18.75rem]"
      data-testid="decision-drawer"
    >
      <div class="grid min-w-0 max-w-[72ch] content-start gap-4.5">
        <section v-if="decision.lines.length > 0" aria-labelledby="said-title">
          <h3 id="said-title" class="mb-1.5 text-caption font-medium text-text-subtle">In the conversation</h3>
          <ul class="grid gap-1.5">
            <li
              v-for="(line, index) in decision.lines"
              :key="index"
              class="grid grid-cols-[4.6rem_minmax(0,1fr)] items-baseline gap-2.5 rounded-control bg-canvas px-2.5 py-1.5 shadow-(--shadow-well) text-small"
              data-testid="decision-line"
            >
              <span class="text-caption font-medium" :class="line.speaker === 'PERSON' ? 'text-session-strong' : 'text-text-subtle'">
                {{ line.speaker === 'PERSON' ? 'You' : 'Assistant' }}
                <time class="block font-mono text-micro font-normal text-text-subtle">{{ timeOf(line.spokenAt) }}</time>
              </span>
              <span class="whitespace-pre-wrap text-text"><DecisionFoundText :text="line.text" :query="query" /></span>
            </li>
          </ul>
        </section>

        <section v-if="decision.options.length > 0" aria-labelledby="offered-title">
          <h3 id="offered-title" class="mb-1.5 text-caption font-medium text-text-subtle">Options</h3>
          <ul class="grid gap-0.5">
            <li
              v-for="option in decision.options"
              :key="option.label"
              class="rounded-control px-2.5 py-1.5"
              :class="decision.state === 'ANSWERED' && decision.answer === option.label ? 'bg-surface-raised' : ''"
              data-testid="decision-option"
            >
              <span class="flex items-center gap-2 text-body font-medium text-text">
                <span><DecisionFoundText :text="option.label" :query="query" /></span>
                <span v-if="option.recommended" :class="TAG" class="border-border-strong text-text-muted">Recommended</span>
                <span v-if="decision.state === 'ANSWERED' && decision.answer === option.label" :class="TAG" class="border-safe/30 bg-safe-soft text-safe">Your answer</span>
              </span>
              <span v-if="option.effect !== null" class="mt-px block text-small leading-[1.45] text-text-muted"><DecisionFoundText :text="option.effect" :query="query" /></span>
            </li>
          </ul>
        </section>

        <slot name="actions" />
      </div>

      <aside
        class="flex min-w-0 flex-col gap-4.5 border-t border-border pt-4 @4xl/task:border-l @4xl/task:border-t-0 @4xl/task:pl-7 @4xl/task:pt-0"
        aria-label="Context for this decision"
      >
        <div v-if="decision.why !== null">
          <h4 class="mb-1.5 text-caption font-medium text-text-subtle">Why this was asked</h4>
          <p class="text-small leading-[1.55] text-text-muted"><DecisionFoundText :text="decision.why" :query="query" /></p>
        </div>
        <div>
          <h4 class="mb-1.5 text-caption font-medium text-text-subtle">Used when it runs</h4>
          <p class="text-small leading-[1.55] text-text-muted">{{ readWhen(decision) }}</p>
        </div>
        <div v-if="decision.previousAnswer !== null">
          <h4 class="mb-1.5 text-caption font-medium text-text-subtle">What changed</h4>
          <p class="text-small leading-[1.55] text-text-muted">
            <time v-if="decision.changedAt !== null" class="mr-2 font-mono text-caption text-text-subtle">{{ timeOf(decision.changedAt) }}</time><DecisionFoundText :text="decision.previousAnswer" :query="query" /> became <DecisionFoundText :text="decision.answer ?? ''" :query="query" />
          </p>
        </div>
        <div v-if="decision.source === 'PERSON'">
          <h4 class="mb-1.5 text-caption font-medium text-text-subtle">{{ SOURCE_LABEL.PERSON }}</h4>
          <p class="text-small leading-[1.55] text-text-muted">Sessions see this correction marked as your edit.</p>
        </div>
      </aside>
    </div>
  </div>
</template>

<style scoped>
@media (prefers-reduced-motion: no-preference) {
  .decision-fresh {
    animation: decision-arrive 2.8s cubic-bezier(0.22, 1, 0.36, 1);
  }
}

@keyframes decision-arrive {
  from {
    background-color: var(--color-accent-soft);
  }
}
</style>
