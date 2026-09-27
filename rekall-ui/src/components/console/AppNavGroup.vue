<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import { activeScreenOf, isScreenActive, type NavGroup } from '@/model/nav-tab'

/**
 * One switcher slot holding several screens, opened as a menu under itself. The slot wears the
 * name of the member on screen, and every candidate name is stacked in the same grid cell so the
 * slot is as wide as its longest one and never resizes the header as the reader moves between them.
 */
const props = defineProps<{ group: NavGroup }>()

const route = useRoute()
const host = ref<HTMLElement | null>(null)
const trigger = ref<HTMLButtonElement | null>(null)
const menu = ref<HTMLElement | null>(null)
const open = ref(false)

const routeName = computed(() => (typeof route.name === 'string' ? route.name : null))
const current = computed(() => activeScreenOf(props.group, routeName.value))
const shownLabel = computed(() => current.value?.label ?? props.group.label)
const candidateLabels = computed(() => [
  props.group.label,
  ...props.group.screens.map((member) => member.label)
])

function menuItems(): HTMLElement[] {
  return Array.from(menu.value?.querySelectorAll<HTMLElement>('[role="menuitem"]') ?? [])
}

async function show(focusIndex: number): Promise<void> {
  open.value = true
  await nextTick()
  menuItems()[focusIndex]?.focus()
}

function hide(returnFocus: boolean): void {
  open.value = false
  if (returnFocus) trigger.value?.focus()
}

function toggle(): void {
  if (open.value) hide(false)
  else
    void show(
      Math.max(
        0,
        props.group.screens.findIndex((member) => member === current.value)
      )
    )
}

function onTriggerKeydown(event: KeyboardEvent): void {
  if (event.key !== 'ArrowDown' || open.value) return
  event.preventDefault()
  void show(0)
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
  <div ref="host" class="relative flex" data-testid="nav-group">
    <button
      ref="trigger"
      type="button"
      class="focus-ring flex h-7 items-center gap-1.5 rounded-[6px] pl-3 pr-2 text-[12px] transition-colors"
      :class="
        current
          ? 'bg-surface-raised text-text shadow-[0_1px_2px_rgb(0_0_0/0.4)]'
          : open
            ? 'text-text'
            : 'text-text-subtle hover:text-text'
      "
      aria-haspopup="menu"
      :aria-expanded="open"
      :aria-label="current ? `${group.label}: ${current.label}` : group.label"
      data-testid="nav-group-trigger"
      @click="toggle"
      @keydown="onTriggerKeydown"
    >
      <span class="grid" aria-hidden="true">
        <span
          v-for="label in candidateLabels"
          :key="label"
          class="[grid-area:1/1]"
          :class="label === shownLabel ? '' : 'invisible'"
        >
          {{ label }}
        </span>
      </span>
      <svg
        class="size-2.5 text-text-subtle transition-transform duration-150"
        :class="open && 'rotate-180'"
        viewBox="0 0 10 10"
        fill="none"
        aria-hidden="true"
      >
        <path
          d="m2.5 3.8 2.5 2.5 2.5-2.5"
          stroke="currentColor"
          stroke-width="1.3"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
    </button>

    <Transition name="popover">
      <div
        v-if="open"
        ref="menu"
        class="absolute left-0 top-full z-(--z-overlay) mt-2 w-[208px] origin-top-left rounded-[var(--radius-card)] border border-border-strong bg-surface p-1 shadow-(--shadow-lift)"
        role="menu"
        :aria-label="group.label"
        data-testid="nav-group-menu"
        @keydown="onMenuKeydown"
      >
        <RouterLink
          v-for="member in group.screens"
          :key="member.to"
          :to="member.to"
          role="menuitem"
          class="focus-ring flex flex-col gap-0.5 rounded-[var(--radius-control)] py-2 pl-3.5 pr-3 outline-offset-[-2px] transition-colors"
          :class="isScreenActive(member, routeName) ? 'selected-row' : 'hover:bg-surface-hover'"
          :aria-current="isScreenActive(member, routeName) ? 'page' : undefined"
          data-testid="nav-group-item"
        >
          <span class="text-[12.5px] font-medium leading-tight text-text">{{ member.label }}</span>
          <span class="text-[11.5px] leading-tight text-text-muted">{{ member.hint }}</span>
        </RouterLink>
      </div>
    </Transition>
  </div>
</template>
