<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import NavAlertCount from '@/components/console/NavAlertCount.vue'
import { useNavAlerts } from '@/composables/useNavAlerts'
import { NAV_GROUPS, activeScreenIn, alertLabel, alertTone, isScreenActive, screensOf } from '@/model/nav-tab'

/**
 * The header's one navigation: a key of three lines that opens every screen under its section.
 * The key's label names the screen on display and sums the alerts, since the glyph carries neither.
 */
const route = useRoute()
const alerts = useNavAlerts()
const host = ref<HTMLElement | null>(null)
const trigger = ref<HTMLButtonElement | null>(null)
const menu = ref<HTMLElement | null>(null)
const open = ref(false)

const routeName = computed(() => (typeof route.name === 'string' ? route.name : null))
const current = computed(() => activeScreenIn(NAV_GROUPS, routeName.value))
const pending = computed(() => screensOf(NAV_GROUPS).reduce((total, screen) => total + alerts(screen.alert), 0))
const keyLabel = computed(() => {
  const screen = current.value ? `Screens: ${current.value.label}` : 'Screens'
  return pending.value > 0 ? `${screen}, ${pending.value} need attention` : screen
})

function menuItems(): HTMLElement[] {
  return Array.from(menu.value?.querySelectorAll<HTMLElement>('[role="menuitem"]') ?? [])
}

async function show(): Promise<void> {
  open.value = true
  await nextTick()
  const items = menuItems()
  const focused = items.findIndex((item) => item.getAttribute('aria-current') === 'page')
  items[Math.max(0, focused)]?.focus()
}

function hide(returnFocus: boolean): void {
  open.value = false
  if (returnFocus) trigger.value?.focus()
}

function toggle(): void {
  if (open.value) hide(false)
  else void show()
}

function onTriggerKeydown(event: KeyboardEvent): void {
  if (event.key !== 'ArrowDown' || open.value) return
  event.preventDefault()
  void show()
}

function onMenuKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape' || event.key === 'Tab') {
    if (event.key === 'Escape') event.preventDefault()
    hide(event.key === 'Escape')
    return
  }
  if (event.key !== 'ArrowDown' && event.key !== 'ArrowUp') return
  event.preventDefault()
  const items = menuItems()
  const focused = items.findIndex((item) => item === document.activeElement)
  const step = event.key === 'ArrowDown' ? 1 : -1
  items[(focused + step + items.length) % items.length]?.focus()
}

function onPointerDown(event: PointerEvent): void {
  if (host.value && event.target instanceof Node && !host.value.contains(event.target)) hide(false)
}

watch(open, (isOpen) => {
  if (isOpen) window.addEventListener('pointerdown', onPointerDown, true)
  else window.removeEventListener('pointerdown', onPointerDown, true)
})

watch(
  () => route.fullPath,
  () => hide(false)
)

onBeforeUnmount(() => window.removeEventListener('pointerdown', onPointerDown, true))
</script>

<template>
  <div ref="host" class="relative min-w-0" data-testid="nav-menu">
    <button
      ref="trigger"
      type="button"
      class="focus-ring grid size-8 shrink-0 cursor-pointer place-items-center rounded-control transition-colors"
      :class="open ? 'bg-surface-raised text-text' : 'text-text-muted hover:bg-surface-hover hover:text-text'"
      aria-haspopup="menu"
      :aria-expanded="open"
      :aria-label="keyLabel"
      data-testid="nav-menu-trigger"
      @click="toggle"
      @keydown="onTriggerKeydown"
    >
      <svg class="size-4" viewBox="0 0 16 16" fill="none" aria-hidden="true">
        <path d="M2.5 4h11M2.5 8h11M2.5 12h11" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
      </svg>
    </button>

    <Transition name="popover">
      <div
        v-if="open"
        ref="menu"
        class="absolute left-0 top-full z-(--z-overlay) mt-1.5 flex max-h-[calc(100vh-64px)] w-[248px] origin-top-left flex-col gap-0.5 overflow-y-auto rounded-card border border-border-strong bg-surface p-1 shadow-(--shadow-lift)"
        role="menu"
        aria-label="Screens"
        data-testid="nav-menu-list"
        @keydown="onMenuKeydown"
      >
        <template v-for="group in NAV_GROUPS" :key="group.label">
          <div class="px-3.5 pb-0.5 pt-2 text-caption font-medium text-text-subtle" role="presentation">
            {{ group.label }}
          </div>
          <RouterLink
            v-for="screen in group.screens"
            :key="screen.to"
            :to="screen.to"
            role="menuitem"
            :title="screen.hint"
            class="focus-ring flex cursor-pointer items-center gap-3 rounded-control py-2 pl-3.5 pr-3 outline-offset-[-2px] transition-colors"
            :class="isScreenActive(screen, routeName) ? 'selected-row' : 'hover:bg-surface-hover'"
            :aria-current="isScreenActive(screen, routeName) ? 'page' : undefined"
            data-testid="nav-menu-item"
          >
            <span class="min-w-0 flex-1 truncate text-small font-medium text-text">{{ screen.label }}</span>
            <NavAlertCount
              v-if="alerts(screen.alert) > 0"
              :count="alerts(screen.alert)"
              :label="alertLabel(screen.alert, alerts(screen.alert))"
              :tone="alertTone(screen.alert)"
            />
          </RouterLink>
        </template>
      </div>
    </Transition>
  </div>
</template>
