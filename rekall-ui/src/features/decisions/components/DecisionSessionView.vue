<script setup lang="ts">
/** The text a session loads for the task's decisions, as the server renders it: current answers only, a question mark where one is open. */
defineProps<{ block: string; taskCode: string }>()
</script>

<template>
  <div class="min-h-0 flex-1 overflow-y-auto px-6 pb-7 pt-5" data-testid="decision-session-view">
    <div class="max-w-[55rem]">
      <p class="mb-3.5 max-w-[70ch] text-body leading-[1.55] text-text-muted">
        This is the block <code class="rounded-key bg-surface-raised px-1.5 py-px font-mono text-small text-text-muted">/rk task:{{ taskCode }}</code>
        hands a session with the task. Task decisions come first, followed by the decisions for each step.
        It holds the current answer to each question, what an answer replaced, and a question mark where one is still open. It does not hold the conversation.
      </p>
      <pre
        class="whitespace-pre-wrap rounded-card border border-border bg-canvas px-4 py-3.5 font-mono text-small leading-[1.7] text-text shadow-(--shadow-well)"
        aria-label="Decisions as the session reads them"
        data-testid="decision-block"
      >{{ block }}</pre>
    </div>
  </div>
</template>
