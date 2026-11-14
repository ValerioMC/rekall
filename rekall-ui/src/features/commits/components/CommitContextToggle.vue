<script setup lang="ts">
import { ref } from 'vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useConsoleStore } from '@/stores/console.store'
import type { CommitReference } from '@/features/commits/model/commitReference'

/**
 * Chooses a commit for `/rk`, or unchooses it. No toast: the state change is the confirmation.
 * `icon` is the always-visible check at the end of a list row; `labelled` is the full control
 * for the header of a single commit.
 */
const props = withDefaults(defineProps<{ reference: CommitReference; variant?: 'icon' | 'labelled' }>(), {
  variant: 'icon'
})

const store = useConsoleStore()
const { run } = useAsyncAction()
const working = ref(false)

async function toggle(): Promise<void> {
  if (working.value) return
  working.value = true
  try {
    await run(() => store.setCommitReferenceInContext(props.reference.id, !props.reference.inContext))
  } finally {
    working.value = false
  }
}
</script>

<template>
  <button
    v-if="variant === 'icon'"
    type="button"
    class="focus-ring hit-target relative grid size-5 shrink-0 place-items-center rounded-control border transition-colors duration-(--motion-quick) disabled:opacity-50"
    :class="
      reference.inContext
        ? 'border-anchor-line bg-anchor-soft text-anchor'
        : 'border-border-strong text-transparent hover:border-anchor-line hover:text-anchor/60'
    "
    :aria-pressed="reference.inContext"
    :aria-label="reference.inContext ? 'Remove this commit from the /rk context' : 'Add this commit to the /rk context'"
    :title="reference.inContext ? 'In the /rk context: click to remove' : 'Add this commit and its diff to the /rk context'"
    :disabled="working"
    data-testid="commit-reference-context-toggle"
    :data-in-context="reference.inContext ? 'true' : 'false'"
    @click.stop="toggle"
  >
    <svg class="size-3" viewBox="0 0 12 12" fill="none" aria-hidden="true">
      <path d="m2.6 6.2 2.4 2.4 4.4-5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
    </svg>
  </button>

  <button
    v-else
    type="button"
    class="focus-ring inline-flex h-8 shrink-0 items-center gap-2 rounded-control border px-3 text-small font-medium whitespace-nowrap transition-colors duration-(--motion-quick) disabled:opacity-50"
    :class="
      reference.inContext
        ? 'border-anchor-line bg-anchor-soft text-anchor hover:bg-anchor-soft/60'
        : 'border-border-strong bg-surface-raised text-text hover:border-anchor-line hover:text-anchor'
    "
    :aria-pressed="reference.inContext"
    :disabled="working"
    data-testid="commit-reference-context-toggle"
    :data-in-context="reference.inContext ? 'true' : 'false'"
    @click="toggle"
  >
    <svg class="size-3.5" viewBox="0 0 12 12" fill="none" aria-hidden="true">
      <path
        v-if="reference.inContext"
        d="m2.6 6.2 2.4 2.4 4.4-5"
        stroke="currentColor"
        stroke-width="1.5"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
      <path v-else d="M6 2.5v7M2.5 6h7" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
    </svg>
    {{ reference.inContext ? 'In the /rk context' : 'Add to /rk context' }}
  </button>
</template>
