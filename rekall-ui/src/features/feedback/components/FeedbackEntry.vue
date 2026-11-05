<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppMarkdownEditor from '@/components/ui/AppMarkdownEditor.vue'
import RevisionEntry from '@/features/feedback/components/RevisionEntry.vue'
import { relativeTime } from '@/common/format/relative-time'
import { kindLabel } from '@/features/feedback/lib/feedback-labels'
import { isRunning, isWaiting, type Feedback } from '@/features/feedback/model/feedback'

/**
 * One message of the thread. Waiting, it carries the accent edge and can be changed; in a pass, a
 * session is working on it and it is fixed; applied, it is sealed and recessed like a pass of a
 * step, with the session's reply below it and a way to add detail to it.
 */
const props = defineProps<{ entry: Feedback }>()
const emit = defineEmits<{
  edit: [entry: Feedback, bodyMarkdown: string]
  remove: [entry: Feedback]
  discard: [entry: Feedback]
  followUp: [entry: Feedback]
}>()

const editing = ref(false)
const draft = ref('')
const collapsed = ref(false)
const replyArrived = ref(false)

const pending = computed(() => isWaiting(props.entry))
const inPass = computed(() => isRunning(props.entry))
const sealed = computed(() => !pending.value && !inPass.value)
const isRevision = computed(() => props.entry.kind === 'REVISION')
const canChange = computed(() => pending.value && !isRevision.value)
const replyLine = computed(() => props.entry.replyMarkdown?.split('\n').find((line) => line.trim() !== '') ?? '')

watch(
  () => props.entry.state,
  (state, before) => {
    if (before !== 'APPLIED' && state === 'APPLIED') replyArrived.value = true
  }
)

function startEditing(): void {
  draft.value = props.entry.bodyMarkdown
  editing.value = true
}

function save(): void {
  if (draft.value.trim() === '') return
  emit('edit', props.entry, draft.value)
  editing.value = false
}
</script>

<template>
  <li
    class="entry rounded-card border bg-surface"
    :class="sealed ? 'entry-applied border-border bg-canvas' : inPass ? 'entry-running border-border' : 'entry-pending border-border'"
    :data-state="entry.state"
    data-testid="feedback-entry"
  >
    <header class="flex items-center gap-2 px-3 py-2">
      <span class="rounded-control border border-border px-1.5 py-px text-micro font-medium text-text-muted" data-testid="feedback-kind">
        {{ kindLabel(entry.kind) }}
      </span>
      <span class="font-mono text-micro text-text-subtle">#{{ entry.seq }}</span>
      <span class="text-micro text-text-subtle">{{ relativeTime(entry.createdAt) }}</span>
      <span v-if="entry.followsSeq !== null" class="text-micro text-text-muted" data-testid="feedback-follows">Follows #{{ entry.followsSeq }}</span>
      <span v-if="pending" class="text-micro font-medium text-accent" data-testid="feedback-state">Waiting for the next run</span>
      <span v-else-if="inPass" class="text-micro font-medium text-session-strong" data-testid="feedback-state">In pass {{ entry.pass }}</span>
      <span v-else class="text-micro font-medium text-text-subtle" data-testid="feedback-state">Applied</span>
      <span class="flex-1" />
      <template v-if="canChange && !editing">
        <AppButton size="sm" variant="quiet" data-testid="feedback-edit" @click="startEditing">Edit</AppButton>
        <AppButton size="sm" variant="danger-quiet" data-testid="feedback-delete" @click="emit('remove', entry)">Delete</AppButton>
      </template>
      <AppButton v-if="sealed" size="sm" variant="quiet" data-testid="feedback-follow-up" @click="emit('followUp', entry)">
        Add detail
      </AppButton>
      <button
        v-if="sealed"
        type="button"
        class="focus-ring rounded-control px-1.5 py-0.5 text-micro text-text-subtle hover:text-text"
        :aria-expanded="!collapsed"
        data-testid="feedback-collapse"
        @click="collapsed = !collapsed"
      >
        {{ collapsed ? 'Show' : 'Collapse' }}
      </button>
    </header>

    <p v-if="sealed && collapsed" class="truncate px-3 pb-2 text-small text-text-subtle" data-testid="feedback-summary">
      {{ replyLine || 'Applied without a reply.' }}
    </p>

    <div v-else class="px-3 pb-3">
      <RevisionEntry v-if="isRevision" :entry="entry" @discard="emit('discard', $event)" />
      <div v-else-if="editing" data-testid="feedback-editing">
        <AppMarkdownEditor v-model="draft" height="160px" compact />
        <div class="mt-2 flex gap-2">
          <AppButton size="sm" variant="primary" :disabled="draft.trim() === ''" data-testid="feedback-save" @click="save">Save</AppButton>
          <AppButton size="sm" variant="quiet" data-testid="feedback-cancel" @click="editing = false">Cancel</AppButton>
        </div>
      </div>
      <AppMarkdownEditor v-else :model-value="entry.bodyMarkdown" readonly compact bare data-testid="feedback-body" />

      <div
        v-if="entry.replyMarkdown"
        class="reply mt-3 ml-4 rounded-control border-l-2 bg-surface-raised px-3 py-2"
        :class="replyArrived && 'reply-arrived'"
        data-testid="feedback-reply"
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
