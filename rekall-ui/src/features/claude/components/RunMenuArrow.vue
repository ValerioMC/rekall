<script setup lang="ts">
import { computed, ref } from 'vue'
import { useAnchoredMenu } from '@/composables/useAnchoredMenu'
import SessionProviderMark from '@/features/claude/components/SessionProviderMark.vue'
import { useSessionProviderOptions } from '@/features/claude/composables/useSessionProviderOptions'
import { SESSION_PROVIDER_LABEL, type SessionProvider } from '@/features/claude/model/session-provider'
import type { RunMenuAction } from '@/features/claude/model/runMenu'
import type { TaskId } from '@/model/branded'

/**
 * The narrow key joined to the right of Run here. A separator splits its menu's groups: ways to run
 * the work, then ways to prepare it, all with `provider`. It takes the main key's look through `ready` and `pressed`, and no press while `busy`.
 */
const props = defineProps<{
  taskId: TaskId
  provider: SessionProvider
  ready: boolean
  pressed: boolean
  busy: boolean
  groups: readonly (readonly RunMenuAction[])[]
}>()

const emit = defineEmits<{ choose: [action: RunMenuAction] }>()

const MENU_WIDTH = 208

const ENTRIES: Record<RunMenuAction, { label: string; testid: string }> = {
  RUN_IN_QUEUE: { label: 'Run in queue', testid: 'run-in-queue' },
  DISCUSS: { label: 'Discuss', testid: 'discuss' },
  PLAN: { label: 'Plan here', testid: 'plan-here' },
  PLAN_FLOW: { label: 'Plan a flow', testid: 'plan-flow' },
  REPLAN: { label: 'Replan', testid: 'replan' }
}

const arrow = ref<HTMLButtonElement | null>(null)
const providerOptions = useSessionProviderOptions(() => props.taskId)
const queueOptions = useSessionProviderOptions(() => props.taskId, () => true)
const unavailable = computed(() => providerOptions.missing(props.provider))
const queueWarning = computed(
  () => queueOptions.options().find((option) => option.provider === props.provider)?.detail ?? null
)
const menuLabel = computed(() => `More ways to run with ${SESSION_PROVIDER_LABEL[props.provider]}`)

const { open, menu, placement, toggle, close, handleKeydown } = useAnchoredMenu(() => arrow.value, MENU_WIDTH)

function choose(action: RunMenuAction): void {
  if (props.busy || unavailable.value) return
  close(true)
  emit('choose', action)
}
</script>

<template>
  <button
    ref="arrow"
    type="button"
    class="focus-ring relative inline-flex h-7 shrink-0 items-center justify-center rounded-key rounded-l-none px-1.5"
    :class="[
      ready
        ? pressed
          ? 'key-lit'
          : 'key-gold font-semibold'
        : 'bg-transparent text-text-subtle transition-colors hover:bg-surface-raised hover:text-text-muted',
      busy && 'cursor-default'
    ]"
    aria-haspopup="menu"
    :aria-expanded="open"
    :aria-disabled="busy"
    aria-label="More ways to run"
    title="More ways to run"
    data-testid="run-menu-arrow"
    @click="busy ? undefined : toggle()"
  >
    <svg
      class="size-3.5 transition-transform"
      :class="open ? 'rotate-180' : ''"
      viewBox="0 0 16 16"
      fill="none"
      aria-hidden="true"
    >
      <path
        d="m4 6 4 4 4-4"
        stroke="currentColor"
        stroke-width="1.6"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
    </svg>
  </button>

  <Teleport to="body">
    <div
      v-if="open"
      ref="menu"
      role="menu"
      :aria-label="menuLabel"
      class="fixed z-(--z-popover) overflow-y-auto rounded-card border border-border-strong bg-surface-raised p-1 shadow-lift"
      :style="placement"
      data-anchored-menu
      data-testid="run-menu"
      @keydown="handleKeydown"
    >
      <p class="flex items-center gap-1.5 px-2.5 pt-1.5 pb-1 text-caption text-text-muted" data-testid="run-menu-provider">
        <SessionProviderMark :provider="provider" :size="12" />
        With {{ SESSION_PROVIDER_LABEL[provider] }}
      </p>
      <template v-if="unavailable">
        <p class="px-2.5 pb-1 text-caption leading-snug text-text-muted">{{ unavailable }}</p>
        <RouterLink
          to="/settings/sessions"
          role="menuitem"
          tabindex="-1"
          class="focus-ring block rounded-control px-2.5 py-2 text-small text-text-muted hover:bg-surface hover:text-text"
        >
          Agent setup in Settings
        </RouterLink>
      </template>
      <template v-for="(group, index) in groups" :key="index">
        <div v-if="index > 0" class="mx-1 my-1 h-px bg-border" role="separator" />
        <button
          v-for="action in group"
          :key="action"
          type="button"
          role="menuitem"
          tabindex="-1"
          class="focus-ring flex w-full items-start justify-between gap-3 rounded-control px-2.5 py-2 text-left transition-colors hover:bg-surface"
          :class="unavailable ? 'cursor-default' : 'cursor-pointer'"
          :aria-disabled="unavailable ? true : undefined"
          :data-testid="ENTRIES[action].testid"
          @click="choose(action)"
        >
          <span class="text-small font-medium" :class="unavailable ? 'text-text-muted' : 'text-text'">
            {{ ENTRIES[action].label }}
            <span v-if="action === 'RUN_IN_QUEUE' && queueWarning" class="mt-0.5 block text-caption font-normal leading-snug text-text-muted">
              {{ queueWarning }}
            </span>
          </span>
          <svg v-if="action === 'REPLAN'" class="size-4 shrink-0 text-text-subtle" viewBox="0 0 16 16" fill="none" aria-hidden="true">
            <path d="M13 8a5 5 0 1 1-1.6-3.7m1.6-1.5v2.7h-2.7" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
        </button>
      </template>
    </div>
  </Teleport>
</template>
