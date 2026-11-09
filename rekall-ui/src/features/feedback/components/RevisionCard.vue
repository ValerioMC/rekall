<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import AppMarkdownEditor from '@/components/ui/AppMarkdownEditor.vue'
import RevisionEntry from '@/features/feedback/components/RevisionEntry.vue'
import { relativeTime } from '@/common/format/relative-time'
import { kindLabel } from '@/features/feedback/lib/feedback-labels'
import { isRunning, isWaiting, type Feedback } from '@/features/feedback/model/feedback'

/**
 * One revision of the description: what changed, what it is applied to, and what the session
 * replied. Waiting, it carries the accent edge and can be discarded; in a pass a session is working
 * on it; applied, it is sealed and recessed, and can be folded to its reply.
 */
const props = defineProps<{ entry: Feedback }>()
const emit = defineEmits<{ discard: [entry: Feedback] }>()

const collapsed = ref(false)
const replyArrived = ref(false)

const pending = computed(() => isWaiting(props.entry))
const inPass = computed(() => isRunning(props.entry))
const sealed = computed(() => !pending.value && !inPass.value)
const replyLine = computed(() => props.entry.replyMarkdown?.split('\n').find((line) => line.trim() !== '') ?? '')

watch(
  () => props.entry.state,
  (state, before) => {
    if (before !== 'APPLIED' && state === 'APPLIED') replyArrived.value = true
  }
)
</script>

<template>
  <li
    class="entry rounded-card border bg-surface"
    :class="sealed ? 'entry-applied border-border bg-canvas' : inPass ? 'entry-running border-border' : 'entry-pending border-border'"
    :data-state="entry.state"
    data-testid="revision-card"
  >
    <header class="flex items-center gap-2 px-3 py-2">
      <span class="rounded-control border border-border px-1.5 py-px text-micro font-medium text-text-muted" data-testid="revision-kind">
        {{ kindLabel(entry.kind) }}
      </span>
      <span class="font-mono text-micro text-text-subtle">#{{ entry.seq }}</span>
      <span class="text-micro text-text-subtle">{{ relativeTime(entry.createdAt) }}</span>
      <span v-if="pending" class="text-micro font-medium text-accent" data-testid="revision-state">Waiting for the next run</span>
      <span v-else-if="inPass" class="text-micro font-medium text-session-strong" data-testid="revision-state">In pass {{ entry.pass }}</span>
      <span v-else class="text-micro font-medium text-text-subtle" data-testid="revision-state">Applied</span>
      <span class="flex-1" />
      <button
        v-if="sealed"
        type="button"
        class="focus-ring rounded-control px-1.5 py-0.5 text-micro text-text-subtle hover:text-text"
        :aria-expanded="!collapsed"
        data-testid="revision-collapse"
        @click="collapsed = !collapsed"
      >
        {{ collapsed ? 'Show' : 'Collapse' }}
      </button>
    </header>

    <p v-if="sealed && collapsed" class="truncate px-3 pb-2 text-small text-text-subtle" data-testid="revision-summary">
      {{ replyLine || 'Applied without a reply.' }}
    </p>

    <div v-else class="px-3 pb-3">
      <RevisionEntry :entry="entry" @discard="emit('discard', $event)" />

      <div
        v-if="entry.replyMarkdown"
        class="reply mt-3 ml-4 rounded-control border-l-2 bg-surface-raised px-3 py-2"
        :class="replyArrived && 'reply-arrived'"
        data-testid="revision-reply"
      >
        <p class="mb-1 text-micro font-medium text-session-strong">
          Session replied <span class="font-normal text-text-subtle">{{ entry.appliedAt ? relativeTime(entry.appliedAt) : '' }}</span>
        </p>
        <AppMarkdownEditor :model-value="entry.replyMarkdown" readonly compact bare />
      </div>
    </div>
  </li>
</template>

<style scoped>
/* Pending is the live edge; applied is sealed, recessed onto the canvas and dimmed like a pass. */
.entry-pending {
  border-left: 2px solid var(--color-accent);
}

.entry-running {
  border-left: 2px solid var(--color-session);
}

.entry-applied {
  box-shadow: inset 0 1px 2px rgb(0 0 0 / 0.3);
}

.entry-applied :deep(.md-editor-preview) {
  opacity: 0.85;
}

.reply {
  border-left-color: var(--color-session);
}

.reply-arrived {
  animation: reply-in var(--motion-slow) ease-out;
}

@keyframes reply-in {
  from {
    opacity: 0;
    transform: translateY(-4px);
  }

  to {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>
