<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import NavAlertCount from '@/components/console/NavAlertCount.vue'
import { useNavAlerts } from '@/composables/useNavAlerts'
import { NAV_ENTRIES, activeScreenIn, alertLabel, isScreenActive, screensOf, type NavEntry } from '@/model/nav-tab'

/**
 * The switcher folded into one key for a narrow window: the screen on display beside a menu glyph,
 * opening every screen as a vertical list under it, a group's members under the group's name. An
 * alert on any screen shows on the key, so folding the tabs never hides a warning.
 */
const route = useRoute()
const alerts = useNavAlerts()
const host = ref<HTMLElement | null>(null)
const trigger = ref<HTMLButtonElement | null>(null)
const menu = ref<HTMLElement | null>(null)
const open = ref(false)

const routeName = computed(() => (typeof route.name === 'string' ? route.name : null))
const current = computed(() => activeScreenIn(NAV_ENTRIES, routeName.value))
const pending = computed(() => NAV_ENTRIES.flatMap(screensOf).reduce((total, screen) => total + alerts(screen.alert), 0))

function entryKey(entry: NavEntry): string {
  return entry.kind === 'screen' ? entry.screen.to : entry.group.label
}

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
  <div ref="host" class="relative self-stretch" data-testid="nav-menu">
    <button
      ref="trigger"
      type="button"
      class="nav-tab nav-tab-current focus-ring cursor-pointer gap-2 px-2.5"
      aria-haspopup="menu"
      :aria-expanded="open"
      :aria-label="current ? `Screens: ${current.label}` : 'Screens'"
      data-testid="nav-menu-trigger"
      @click="toggle"
      @keydown="onTriggerKeydown"
    >
      <svg class="size-4 shrink-0 text-text-muted" viewBox="0 0 16 16" fill="none" aria-hidden="true">
        <path d="M2.5 4h11M2.5 8h11M2.5 12h11" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
      </svg>
      <span class="truncate">{{ current?.label ?? 'Screens' }}</span>
      <NavAlertCount v-if="pending > 0" :count="pending" :label="`${pending} need attention`" />
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
        <template v-for="entry in NAV_ENTRIES" :key="entryKey(entry)">
          <div
            v-if="entry.kind === 'group'"
            class="px-3.5 pb-0.5 pt-2 text-caption font-medium text-text-subtle"
            role="presentation"
          >
            {{ entry.group.label }}
          </div>
          <RouterLink
            v-for="screen in screensOf(entry)"
            :key="screen.to"
            :to="screen.to"
            role="menuitem"
            class="focus-ring flex cursor-pointer items-center gap-3 rounded-control py-2 pr-3 outline-offset-[-2px] transition-colors"
            :class="[
              isScreenActive(screen, routeName) ? 'selected-row' : 'hover:bg-surface-hover',
              entry.kind === 'group' ? 'pl-6' : 'pl-3.5'
            ]"
            :aria-current="isScreenActive(screen, routeName) ? 'page' : undefined"
            data-testid="nav-menu-item"
          >
            <span class="flex min-w-0 flex-1 flex-col gap-0.5">
              <span class="text-small font-medium leading-tight text-text">{{ screen.label }}</span>
              <span class="text-caption leading-tight text-text-muted">{{ screen.hint }}</span>
            </span>
            <NavAlertCount
              v-if="alerts(screen.alert) > 0"
              :count="alerts(screen.alert)"
              :label="alertLabel(screen.alert, alerts(screen.alert))"
            />
          </RouterLink>
        </template>
      </div>
    </Transition>
  </div>
</template>
