<script setup lang="ts">
import { nextTick, ref, watch } from 'vue'
import { adjacentMenuPlacement, focusMenuStart, moveMenuFocus, useAnchoredMenu } from '@/composables/useAnchoredMenu'
import SessionProviderMark from '@/features/claude/components/SessionProviderMark.vue'
import SessionProviderMenu from '@/features/claude/components/SessionProviderMenu.vue'
import { useSessionProviderOptions } from '@/features/claude/composables/useSessionProviderOptions'
import { RUN_WITH_PROVIDER_LABEL, type SessionProvider } from '@/features/claude/model/session-provider'
import { opensProviderMenu, type ProviderMenuAction, type RunMenuAction, type RunMenuSelection } from '@/features/claude/model/runMenu'
import type { TaskId } from '@/model/branded'

/**
 * The narrow key joined to the right of Run here. A separator splits its menu's groups: ways to run
 * the work, then ways to prepare it. It takes the main key's look through `ready` and `pressed`, and no press while `busy`.
 */
const props = defineProps<{
  taskId: TaskId
  ready: boolean
  pressed: boolean
  busy: boolean
  groups: readonly (readonly RunMenuAction[])[]
}>()

const emit = defineEmits<{ choose: [selection: RunMenuSelection] }>()

const MENU_WIDTH = 200
const PROVIDER_WIDTH = 240

const ENTRIES: Record<RunMenuAction, { label: string; testid: string }> = {
  CLAUDE: { label: RUN_WITH_PROVIDER_LABEL.CLAUDE, testid: 'run-with-CLAUDE' },
  CODEX: { label: RUN_WITH_PROVIDER_LABEL.CODEX, testid: 'run-with-CODEX' },
  RUN_IN_QUEUE: { label: 'Run in queue', testid: 'run-in-queue' },
  DISCUSS: { label: 'Discuss', testid: 'discuss' },
  PLAN: { label: 'Plan here', testid: 'plan-here' },
  PLAN_FLOW: { label: 'Plan a flow', testid: 'plan-flow' },
  REPLAN: { label: 'Replan', testid: 'replan' }
}

const arrow = ref<HTMLButtonElement | null>(null)
const providerMenu = ref<HTMLElement | null>(null)
const providerAction = ref<ProviderMenuAction | null>(null)
const actionRow = ref<HTMLElement | null>(null)
const providerPlacement = ref<Record<string, string> | null>(null)
const { missing } = useSessionProviderOptions(() => props.taskId)

const { open, menu, placement, place, toggle, close, handleKeydown } = useAnchoredMenu(
  () => arrow.value,
  MENU_WIDTH,
  { onPlace: placeProviders, relatedMenu: () => providerMenu.value }
)

watch(open, (visible) => {
  if (!visible) clearProviders()
})

function clearProviders(): void {
  providerAction.value = null
  providerPlacement.value = null
  actionRow.value = null
}

function placeProviders(): void {
  if (!menu.value || !actionRow.value) return
  const parent = menu.value.getBoundingClientRect()
  const row = actionRow.value.getBoundingClientRect()
  const placement = adjacentMenuPlacement(parent, row.height ? row : parent, providerMenu.value?.scrollHeight ?? 0, PROVIDER_WIDTH)
  const focused = document.activeElement
  const moved = Boolean(placement) !== Boolean(providerPlacement.value)
  providerPlacement.value = placement
  if (moved && focused instanceof HTMLElement && providerMenu.value?.contains(focused)) {
    void nextTick(() => focused.focus({ preventScroll: true }))
  }
}

async function showProviders(action: ProviderMenuAction, row: HTMLElement): Promise<void> {
  actionRow.value = row
  providerAction.value = action
  placeProviders()
  await nextTick()
  place()
  await nextTick()
  placeProviders()
  await nextTick()
  focusMenuStart(providerPlacement.value ? providerMenu.value : menu.value)
}

async function backToActions(): Promise<void> {
  const row = actionRow.value
  clearProviders()
  await nextTick()
  place()
  row?.focus({ preventScroll: true })
}

function choose(action: RunMenuAction, event: Event): void {
  if (props.busy) return
  if (opensProviderMenu(action)) {
    if (event.currentTarget instanceof HTMLElement) void showProviders(action, event.currentTarget)
    return
  }
  if (action !== 'REPLAN' && missing(action)) return
  close(true)
  emit('choose', action === 'REPLAN' ? { action } : { action: 'RUN', provider: action })
}

function chooseProvider(provider: SessionProvider): void {
  if (props.busy || !providerAction.value) return
  const action = providerAction.value
  close(true)
  emit('choose', { action, provider })
}

function onActionKeydown(event: KeyboardEvent, action: RunMenuAction): void {
  if (event.key !== 'ArrowRight' || !opensProviderMenu(action)) return
  event.preventDefault()
  event.stopPropagation()
  choose(action, event)
}

function onProviderKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape' || event.key === 'ArrowLeft') {
    event.preventDefault()
    event.stopPropagation()
    void backToActions()
    return
  }
  if (event.key === 'Tab') {
    handleKeydown(event)
    return
  }
  moveMenuFocus(event, providerPlacement.value ? providerMenu.value : menu.value)
}

function onRootKeydown(event: KeyboardEvent): void {
  if (providerAction.value && !providerPlacement.value) onProviderKeydown(event)
  else handleKeydown(event)
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
      :aria-label="providerAction && !providerPlacement ? ENTRIES[providerAction].label : 'More ways to run'"
      class="fixed z-(--z-popover) overflow-y-auto rounded-card border border-border-strong bg-surface-raised p-1 shadow-lift"
      :style="placement"
      data-anchored-menu
      data-testid="run-menu"
      @keydown="onRootKeydown"
    >
      <div :hidden="Boolean(providerAction && !providerPlacement)">
        <template v-for="(group, index) in groups" :key="index">
          <div v-if="index > 0" class="mx-1 my-1 h-px bg-border" role="separator" />
          <button
            v-for="action in group"
            :key="action"
            type="button"
            role="menuitem"
            tabindex="-1"
            class="focus-ring flex w-full cursor-pointer items-start justify-between gap-3 rounded-control px-2.5 py-2 text-left transition-colors hover:bg-surface"
            :class="providerAction === action ? 'bg-surface' : undefined"
            :aria-haspopup="opensProviderMenu(action) ? 'menu' : undefined"
            :aria-expanded="opensProviderMenu(action) ? providerAction === action : undefined"
            :aria-disabled="(action === 'CLAUDE' || action === 'CODEX') && missing(action) ? true : undefined"
            :data-testid="ENTRIES[action].testid"
            @keydown="onActionKeydown($event, action)"
            @click="choose(action, $event)"
          >
            <span class="text-small font-medium text-text">
              {{ ENTRIES[action].label }}
              <span v-if="(action === 'CLAUDE' || action === 'CODEX') && missing(action)" class="block text-caption font-normal text-text-muted">
                {{ missing(action) }}
              </span>
            </span>
            <svg v-if="opensProviderMenu(action)" class="size-4 shrink-0 text-text-subtle" viewBox="0 0 16 16" fill="none" aria-hidden="true">
              <path d="m6 4 4 4-4 4" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
            <SessionProviderMark v-else-if="action === 'CLAUDE' || action === 'CODEX'" :provider="action" :size="16" />
            <svg v-else class="size-4 shrink-0 text-text-subtle" viewBox="0 0 16 16" fill="none" aria-hidden="true">
              <path d="M13 8a5 5 0 1 1-1.6-3.7m1.6-1.5v2.7h-2.7" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
          </button>
        </template>
      </div>
      <Teleport to="body" :disabled="!providerPlacement">
        <div
          v-if="providerAction"
          ref="providerMenu"
          :role="providerPlacement ? 'menu' : undefined"
          :aria-label="providerPlacement ? ENTRIES[providerAction].label : undefined"
          :class="providerPlacement && 'fixed z-(--z-popover) overflow-y-auto rounded-card border border-border-strong bg-surface-raised p-1 shadow-lift'"
          :style="providerPlacement ?? undefined"
          data-anchored-menu
          @keydown="providerPlacement && onProviderKeydown($event)"
        >
          <SessionProviderMenu
            :task-id="taskId"
            :queueing="providerAction === 'RUN_IN_QUEUE'"
            :busy="busy"
            :back="!providerPlacement"
            @choose="chooseProvider"
            @back="backToActions"
          />
        </div>
      </Teleport>
    </div>
  </Teleport>
</template>
