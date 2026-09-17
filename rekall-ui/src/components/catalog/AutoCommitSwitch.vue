<script setup lang="ts">
import { computed, useId } from 'vue'
import type { ProjectRepository } from '@/model/catalog'

/**
 * The per-project auto-commit control: a switch that only arms on a folder git recognises as a
 * repository. The strip above it says what git sees there (branch, identity), so the person
 * knows what a claim would commit as before turning it on.
 */
const props = withDefaults(
  defineProps<{
    modelValue: boolean
    repository: ProjectRepository | null
    /** The repository status is being read; the switch holds still meanwhile. */
    pending?: boolean
    /** The last toggle is being saved. */
    saving?: boolean
  }>(),
  { pending: false, saving: false }
)

const emit = defineEmits<{ 'update:modelValue': [value: boolean] }>()

const id = useId()
const descriptionId = `${id}-description`

type Readiness = 'unset' | 'missing' | 'not-repository' | 'no-identity' | 'ready'

const readiness = computed<Readiness>(() => {
  const repository = props.repository
  if (!repository || !repository.folder) return 'unset'
  if (!repository.exists) return 'missing'
  if (!repository.repository) return 'not-repository'
  if (!repository.userEmail) return 'no-identity'
  return 'ready'
})

const armed = computed(() => readiness.value === 'ready' || readiness.value === 'no-identity')
const disabled = computed(() => !armed.value || props.pending || props.saving)
const on = computed(() => props.modelValue && armed.value)

const identity = computed(() => {
  const repository = props.repository
  if (!repository?.userEmail) return null
  return repository.userName ? `${repository.userName} <${repository.userEmail}>` : repository.userEmail
})

const REASON: Record<Readiness, string | null> = {
  unset: 'Set the folder above first.',
  missing: 'That folder is not on this machine.',
  'not-repository': 'Not a git repository. Run git init there, or point at a folder that has one.',
  'no-identity':
    'git has no user.email here: a commit will fail until git config --global user.email is set.',
  ready: null
}

const reason = computed(() => REASON[readiness.value])

function toggle(): void {
  if (disabled.value) return
  emit('update:modelValue', !props.modelValue)
}
</script>

<template>
  <div
    class="auto-commit"
    :class="{ 'auto-commit-on': on, 'auto-commit-disarmed': !armed }"
    data-testid="auto-commit"
    :data-readiness="readiness"
  >
    <div class="flex items-start gap-3.5">
      <button
        :id="id"
        type="button"
        role="switch"
        :aria-checked="on"
        :aria-disabled="disabled"
        :aria-describedby="descriptionId"
        class="focus-ring auto-commit-switch shrink-0"
        :class="{ 'auto-commit-switch-busy': saving || pending }"
        data-testid="auto-commit-switch"
        @click="toggle"
      >
        <span class="auto-commit-track" aria-hidden="true">
          <svg class="auto-commit-graph" viewBox="0 0 44 22" fill="none">
            <path d="M8 11h28" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
            <circle cx="8" cy="11" r="2.2" fill="currentColor" />
            <circle cx="22" cy="11" r="2.2" fill="currentColor" />
            <circle cx="36" cy="11" r="2.2" fill="currentColor" />
          </svg>
          <span class="auto-commit-knob" />
        </span>
      </button>

      <div class="min-w-0 flex-1">
        <label :for="id" class="block cursor-pointer text-[13.5px] font-medium text-text">
          Commit on claim
        </label>
        <p :id="descriptionId" class="mt-1 max-w-[62ch] text-[12px] leading-relaxed text-text-subtle">
          When a session claims a step, or writes the wrapup of a task with no checklist,
          everything in this folder is committed under a generated message and logged against
          that step or task. The session is told not to commit by hand.
        </p>

        <p
          v-if="reason"
          class="mt-2 flex items-start gap-1.5 text-[12px] leading-relaxed"
          :class="readiness === 'no-identity' ? 'text-warn' : 'text-text-muted'"
          data-testid="auto-commit-reason"
        >
          <span class="mt-[7px] size-1.5 shrink-0 rounded-full bg-current opacity-70" aria-hidden="true" />
          <span>{{ reason }}</span>
        </p>

        <div
          v-if="repository?.repository"
          class="mt-2.5 flex flex-wrap items-center gap-x-3 gap-y-1 font-mono text-[11.5px]"
          data-testid="auto-commit-repository"
        >
          <span class="flex items-center gap-1.5 text-text-muted">
            <svg class="size-3.5 shrink-0 opacity-80" viewBox="0 0 16 16" fill="none" aria-hidden="true">
              <circle cx="4" cy="3.5" r="1.8" stroke="currentColor" stroke-width="1.4" />
              <circle cx="4" cy="12.5" r="1.8" stroke="currentColor" stroke-width="1.4" />
              <circle cx="12" cy="5.5" r="1.8" stroke="currentColor" stroke-width="1.4" />
              <path d="M4 5.3v5.4M12 7.3c0 2.6-2.3 3.2-4.6 3.4-1.6.2-3.4.7-3.4 2" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
            </svg>
            <span class="text-text">{{ repository.branch ?? 'no branch yet' }}</span>
          </span>
          <span v-if="identity" class="min-w-0 truncate text-text-subtle" :title="identity">
            commits as <span class="text-text-muted">{{ identity }}</span>
          </span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.auto-commit {
  --switch-tint: var(--color-text-subtle);
  position: relative;
  border-radius: var(--radius-control);
  padding: 12px 14px;
  border: 1px solid var(--color-border);
  background: var(--color-surface);
  transition:
    border-color 220ms ease,
    box-shadow 320ms ease;
}

.auto-commit-on {
  --switch-tint: var(--color-accent);
  border-color: color-mix(in srgb, var(--color-accent) 45%, var(--color-border));
  box-shadow: var(--shadow-glow);
}

.auto-commit-disarmed {
  background: color-mix(in srgb, var(--color-surface) 70%, var(--color-canvas));
}

.auto-commit-switch {
  display: block;
  border-radius: 999px;
  cursor: pointer;
  padding: 0;
  background: transparent;
  border: 0;
}

.auto-commit-switch[aria-disabled='true'] {
  cursor: not-allowed;
}

.auto-commit-track {
  position: relative;
  display: block;
  width: 44px;
  height: 22px;
  border-radius: 999px;
  overflow: hidden;
  border: 1px solid var(--color-border-strong);
  background: var(--color-canvas);
  color: var(--switch-tint);
  transition:
    border-color 220ms ease,
    background-color 220ms ease;
}

.auto-commit-on .auto-commit-track {
  border-color: var(--color-accent-deep);
  background: color-mix(in srgb, var(--color-accent) 18%, var(--color-canvas));
}

.auto-commit-disarmed .auto-commit-track {
  opacity: 0.45;
}

/* The track is a three-commit graph. Off, it sits dim behind the knob; on, it lights up
   and the knob slides past every commit, which is what a claim does to the history. */
.auto-commit-graph {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  opacity: 0.35;
  transition: opacity 260ms ease;
}

.auto-commit-on .auto-commit-graph {
  opacity: 1;
}

.auto-commit-knob {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 16px;
  height: 16px;
  border-radius: 999px;
  background: var(--color-text-muted);
  box-shadow: 0 1px 2px rgb(0 0 0 / 0.6);
  transition:
    transform 260ms cubic-bezier(0.2, 0.8, 0.2, 1),
    background-color 220ms ease,
    box-shadow 220ms ease;
}

.auto-commit-on .auto-commit-knob {
  transform: translateX(22px);
  background: var(--color-accent-strong);
  box-shadow:
    0 1px 2px rgb(0 0 0 / 0.6),
    0 0 10px 1px color-mix(in srgb, var(--color-accent) 55%, transparent);
}

.auto-commit-switch-busy .auto-commit-knob {
  animation: auto-commit-breathe 900ms ease-in-out infinite alternate;
}

@keyframes auto-commit-breathe {
  from {
    opacity: 1;
  }
  to {
    opacity: 0.45;
  }
}

@media (prefers-reduced-motion: reduce) {
  .auto-commit,
  .auto-commit-track,
  .auto-commit-graph,
  .auto-commit-knob {
    transition: none;
  }

  .auto-commit-switch-busy .auto-commit-knob {
    animation: none;
    opacity: 0.6;
  }
}
</style>
