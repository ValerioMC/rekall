<script setup lang="ts">
import StepSeal from '@/features/steps/components/StepSeal.vue'
import { sameScope, type ChangeScope } from '@/features/commits/model/changeScope'
import type { ChangeSource, ScopeGroup } from '@/features/commits/lib/change-sources'

/** What to compare, as one track: all changes, then each step with the commits logged against it, then the working tree. */
const props = defineProps<{ scope: ChangeScope; groups: readonly ScopeGroup[]; workingTree: ChangeSource | null }>()

const emit = defineEmits<{ select: [scope: ChangeScope] }>()

const BUTTON =
  'focus-ring inline-flex h-7 cursor-pointer items-center gap-1.5 rounded-control px-2.5 text-small font-medium whitespace-nowrap transition-colors duration-(--motion-quick)'

function classOf(target: ChangeScope): string {
  return sameScope(props.scope, target)
    ? 'bg-surface-raised text-text shadow-key shadow-[inset_0_-2px_0_var(--color-accent)]'
    : 'text-text-muted hover:bg-surface-raised hover:text-text'
}
</script>

<template>
  <div class="flex min-w-0 items-center gap-1 overflow-x-auto rounded-card border border-border bg-canvas p-0.75" role="group" aria-label="What to compare" data-testid="change-scope-track">
    <button type="button" :class="[BUTTON, classOf({ kind: 'all' })]" :aria-pressed="scope.kind === 'all'" title="Everything the sessions changed for this task" data-testid="scope-all" @click="emit('select', { kind: 'all' })">
      All changes
    </button>
    <template v-for="group in groups" :key="group.step?.id ?? 'task'">
      <span class="mx-1 h-4.5 w-px shrink-0 bg-border-strong" aria-hidden="true" />
      <button
        v-if="group.step"
        type="button"
        :class="[BUTTON, classOf({ kind: 'step', stepId: group.step.id })]"
        :aria-pressed="sameScope(scope, { kind: 'step', stepId: group.step.id })"
        :title="group.step.title"
        data-testid="scope-step"
        @click="emit('select', { kind: 'step', stepId: group.step.id })"
      >
        <span class="size-3.5" aria-hidden="true"><StepSeal :state="group.step.state" /></span>
        Step {{ group.step.number }}
      </button>
      <span v-else class="px-2 text-caption text-text-subtle">Task</span>
      <button
        v-for="source in group.sources"
        :key="source.id"
        type="button"
        :class="[BUTTON, classOf({ kind: 'commit', referenceId: source.id })]"
        :aria-pressed="sameScope(scope, { kind: 'commit', referenceId: source.id })"
        :title="source.subject"
        data-testid="scope-commit"
        @click="emit('select', { kind: 'commit', referenceId: source.id })"
      >
        <span class="font-mono text-caption">{{ source.commitHash?.slice(0, 7) }}</span>
      </button>
    </template>
    <template v-if="workingTree">
      <span class="mx-1 h-4.5 w-px shrink-0 bg-border-strong" aria-hidden="true" />
      <button
        type="button"
        :class="[BUTTON, classOf({ kind: 'working-tree' })]"
        :aria-pressed="scope.kind === 'working-tree'"
        title="What a session has written and not committed yet"
        data-testid="scope-working-tree"
        @click="emit('select', { kind: 'working-tree' })"
      >
        <span class="size-1.5 rounded-full bg-session motion-safe:animate-pulse" aria-hidden="true" />
        Working tree
      </button>
    </template>
  </div>
</template>
