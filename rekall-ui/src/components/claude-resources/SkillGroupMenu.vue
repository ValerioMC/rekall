<script setup lang="ts">
import { nextTick, onUnmounted, ref } from 'vue'

/** Where one of the person's skills is filed in Rekall. Choosing moves nothing on disk. */
const props = defineProps<{
  skillName: string
  /** The group the skill is in now; null when it is in none. */
  current: string | null
  groups: readonly string[]
}>()

const emit = defineEmits<{ move: [group: string | null] }>()

const MENU_WIDTH = 224
const ROW_HEIGHT = 32
const EDGE = 8

const open = ref(false)
const naming = ref(false)
const newName = ref('')
const trigger = ref<HTMLButtonElement | null>(null)
const nameField = ref<HTMLInputElement | null>(null)
const placement = ref<Record<string, string>>({})

function place(): void {
  const box = trigger.value?.getBoundingClientRect()
  if (!box) return
  const height = ROW_HEIGHT * (props.groups.length + 3) + 16
  const left = Math.max(EDGE, Math.min(box.right - MENU_WIDTH, window.innerWidth - MENU_WIDTH - EDGE))
  const below = box.bottom + 4 + height <= window.innerHeight
  placement.value = below
    ? { left: `${left}px`, top: `${box.bottom + 4}px`, width: `${MENU_WIDTH}px` }
    : { left: `${left}px`, bottom: `${window.innerHeight - box.top + 4}px`, width: `${MENU_WIDTH}px` }
}

function close(returnFocus: boolean): void {
  open.value = false
  naming.value = false
  newName.value = ''
  document.removeEventListener('pointerdown', onOutside, true)
  if (returnFocus) trigger.value?.focus()
}

function onOutside(event: Event): void {
  const target = event.target as Node
  if (trigger.value?.contains(target) || document.getElementById(menuId)?.contains(target)) return
  close(false)
}

const menuId = `skill-group-menu-${Math.random().toString(36).slice(2, 8)}`

async function toggle(): Promise<void> {
  if (open.value) return close(true)
  place()
  open.value = true
  document.addEventListener('pointerdown', onOutside, true)
  await nextTick()
  document.getElementById(menuId)?.querySelector<HTMLElement>('[role="menuitem"]')?.focus()
}

function choose(group: string | null): void {
  close(true)
  if (group !== props.current) emit('move', group)
}

async function startNaming(): Promise<void> {
  naming.value = true
  await nextTick()
  nameField.value?.focus()
}

function createGroup(): void {
  const name = newName.value.trim()
  if (name !== '') choose(name)
}

function onKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape') {
    event.stopPropagation()
    return close(true)
  }
  const step = event.key === 'ArrowDown' ? 1 : event.key === 'ArrowUp' ? -1 : 0
  if (step === 0 || naming.value) return
  event.preventDefault()
  const items = [...(document.getElementById(menuId)?.querySelectorAll<HTMLElement>('[role="menuitem"]') ?? [])]
  const at = items.indexOf(document.activeElement as HTMLElement)
  items[(at + step + items.length) % items.length]?.focus()
}

onUnmounted(() => document.removeEventListener('pointerdown', onOutside, true))
</script>

<template>
  <button
    ref="trigger"
    type="button"
    class="focus-ring grid size-6 cursor-pointer place-items-center rounded-key text-text-subtle transition-colors hover:bg-surface hover:text-text"
    aria-haspopup="menu"
    :aria-expanded="open"
    :aria-label="`Move ${skillName} to a group`"
    title="Move to a group (Rekall only, the file stays put)"
    data-testid="skill-group-trigger"
    @click.stop="toggle"
  >
    <svg class="size-3.5" viewBox="0 0 16 16" fill="none" aria-hidden="true">
      <path d="M2 4.5A1.5 1.5 0 0 1 3.5 3h2.8l1.4 1.5h4.8A1.5 1.5 0 0 1 14 6v5.5a1.5 1.5 0 0 1-1.5 1.5h-9A1.5 1.5 0 0 1 2 11.5z" stroke="currentColor" stroke-width="1.3" stroke-linejoin="round" />
      <path d="M8 7v4M6 9h4" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" />
    </svg>
  </button>

  <Teleport to="body">
    <div
      v-if="open"
      :id="menuId"
      role="menu"
      aria-label="Move to a group"
      class="fixed z-50 rounded-card border border-border-strong bg-surface-raised p-1 shadow-lift"
      :style="placement"
      data-testid="skill-group-menu"
      @keydown="onKeydown"
    >
      <p class="px-2 pb-1 pt-0.5 text-micro text-text-subtle">Rekall groups only. {{ skillName }} stays where it is on disk.</p>
      <button
        v-for="group in groups"
        :key="group"
        type="button"
        role="menuitem"
        class="focus-ring flex h-8 w-full cursor-pointer items-center gap-2 rounded-control px-2 text-left text-small text-text transition-colors hover:bg-surface"
        @click="choose(group)"
      >
        <svg class="size-3 shrink-0" :class="group === current ? 'text-accent' : 'opacity-0'" viewBox="0 0 12 12" fill="none" aria-hidden="true">
          <path d="m2.5 6.5 2.2 2.2 4.8-5" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
        <span class="min-w-0 flex-1 truncate">{{ group }}</span>
      </button>
      <button
        v-if="current !== null"
        type="button"
        role="menuitem"
        class="focus-ring flex h-8 w-full cursor-pointer items-center gap-2 rounded-control px-2 text-left text-small text-text-muted transition-colors hover:bg-surface hover:text-text"
        @click="choose(null)"
      >
        <span class="size-3 shrink-0" aria-hidden="true" />
        Take out of {{ current }}
      </button>
      <div class="my-1 border-t border-border" aria-hidden="true" />
      <form v-if="naming" class="flex items-center gap-1 px-1 py-0.5" @submit.prevent="createGroup">
        <label class="sr-only" :for="`${menuId}-name`">New group name</label>
        <input
          :id="`${menuId}-name`"
          ref="nameField"
          v-model="newName"
          maxlength="60"
          autocomplete="off"
          spellcheck="false"
          placeholder="Group name"
          class="field h-7 min-w-0 flex-1 rounded-control px-2 text-small text-text"
          data-testid="skill-group-name"
        />
        <button type="submit" class="focus-ring h-7 cursor-pointer rounded-control px-2 text-small font-medium text-accent hover:bg-surface disabled:cursor-default disabled:opacity-40" :disabled="newName.trim() === ''">
          Add
        </button>
      </form>
      <button
        v-else
        type="button"
        role="menuitem"
        class="focus-ring flex h-8 w-full cursor-pointer items-center gap-2 rounded-control px-2 text-left text-small text-accent transition-colors hover:bg-surface"
        data-testid="skill-group-new"
        @click="startNaming"
      >
        <svg class="size-3 shrink-0" viewBox="0 0 12 12" fill="none" aria-hidden="true">
          <path d="M6 2.5v7M2.5 6h7" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
        </svg>
        New group
      </button>
    </div>
  </Teleport>
</template>
