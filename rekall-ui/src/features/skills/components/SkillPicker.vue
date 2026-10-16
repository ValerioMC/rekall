<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import SkillChip from '@/features/skills/components/SkillChip.vue'
import SkillGlyph from '@/features/skills/components/SkillGlyph.vue'
import { enterDialog, leaveDialog } from '@/common/a11y/dialog-stack'
import { useSkillCatalog } from '@/features/skills/composables/useSkillCatalog'
import { filterSkillOptions, groupSkillOptions, type SkillOption } from '@/features/skills/model/skill-options'
import { SKILLS_PER_OWNER_MAX } from '@/features/skills/model/skill-assignment'

/**
 * The catalogue of Claude skills, hung from the button that opened it, to tick any number of
 * them. Every tick is sent up at once, so the owner's list is always what the panel shows. A
 * name the owner holds that is no longer under `~/.claude` stays on top, to be taken off.
 * It opens above a modal and takes the keyboard from it, so Escape closes the panel alone.
 */
const props = withDefaults(
  defineProps<{
    modelValue: readonly string[]
    /** The element the panel hangs from: its left edge is the panel's left edge. */
    anchor: HTMLElement
    /** Skills already in force from a wider scope; shown ticked and left alone. */
    inherited?: readonly string[]
    title?: string
  }>(),
  { inherited: () => [], title: 'Skills' }
)

const emit = defineEmits<{ 'update:modelValue': [names: string[]]; close: [] }>()

const WIDTH = 380
const GUTTER = 8
const PANEL_HEIGHT = 420

const { options, names: known, loaded } = useSkillCatalog()
const panel = ref<HTMLElement | null>(null)
const filterField = ref<HTMLInputElement | null>(null)
const filter = ref('')
const highlighted = ref(0)
let dialogToken: symbol | null = null
const position = ref({ top: 0, left: 0, width: 0, opensUp: false })

const shown = computed(() => filterSkillOptions(options.value, filter.value))
const groups = computed(() => groupSkillOptions(shown.value))
const missing = computed(() => props.modelValue.filter((name) => loaded.value && !known.value.has(name)))
const full = computed(() => props.modelValue.length >= SKILLS_PER_OWNER_MAX)

function chosen(option: SkillOption): boolean {
  return props.modelValue.includes(option.name)
}

function fromWiderScope(option: SkillOption): boolean {
  return props.inherited.includes(option.name)
}

function toggle(option: SkillOption): void {
  if (fromWiderScope(option)) return
  if (chosen(option)) return remove(option.name)
  if (full.value) return
  emit('update:modelValue', [...props.modelValue, option.name])
}

function remove(name: string): void {
  emit(
    'update:modelValue',
    props.modelValue.filter((candidate) => candidate !== name)
  )
}

function indexOf(option: SkillOption): number {
  return shown.value.indexOf(option)
}

watch(filter, () => (highlighted.value = 0))

/** Opens under the button, or above it when the button sits too low for the panel. */
function place(): void {
  const rect = props.anchor.getBoundingClientRect()
  const width = Math.min(WIDTH, window.innerWidth - 2 * GUTTER)
  const left = Math.max(GUTTER, Math.min(rect.left, window.innerWidth - width - GUTTER))
  const opensUp =
    window.innerHeight - rect.bottom < PANEL_HEIGHT && rect.top > window.innerHeight - rect.bottom
  position.value = {
    top: opensUp ? window.innerHeight - rect.top + 6 : rect.bottom + 6,
    left,
    width,
    opensUp
  }
}

function onKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape') {
    event.stopPropagation()
    emit('close')
    return
  }
  if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
    event.preventDefault()
    event.stopPropagation()
    if (!shown.value.length) return
    const delta = event.key === 'ArrowDown' ? 1 : -1
    highlighted.value = (highlighted.value + delta + shown.value.length) % shown.value.length
    panel.value
      ?.querySelector<HTMLElement>(`[data-row-index="${highlighted.value}"]`)
      ?.scrollIntoView({ block: 'nearest' })
    return
  }
  if (event.key === 'Enter') {
    event.preventDefault()
    event.stopPropagation()
    const option = shown.value[highlighted.value]
    if (option) toggle(option)
    return
  }
  event.stopPropagation()
}

function onPointerDown(event: PointerEvent): void {
  const target = event.target as Node | null
  if (!target || panel.value?.contains(target) || props.anchor.contains(target)) return
  emit('close')
}

onMounted(async () => {
  dialogToken = enterDialog()
  place()
  window.addEventListener('keydown', onKeydown, true)
  window.addEventListener('pointerdown', onPointerDown, true)
  window.addEventListener('resize', place)
  await nextTick()
  filterField.value?.focus()
})

onUnmounted(() => {
  if (dialogToken !== null) leaveDialog(dialogToken)
  window.removeEventListener('keydown', onKeydown, true)
  window.removeEventListener('pointerdown', onPointerDown, true)
  window.removeEventListener('resize', place)
})
</script>

<template>
  <Teleport to="body">
    <div
      ref="panel"
      class="fixed z-(--z-popover) flex max-h-[min(420px,calc(100vh-24px))] flex-col overflow-hidden rounded-card border border-border-strong bg-surface shadow-lift"
      :style="{
        left: `${position.left}px`,
        width: `${position.width}px`,
        ...(position.opensUp ? { bottom: `${position.top}px` } : { top: `${position.top}px` })
      }"
      role="dialog"
      :aria-label="title"
      data-testid="skill-picker"
    >
      <div class="flex items-center gap-2 border-b border-border px-3 py-2">
        <SkillGlyph :size="14" class="text-anchor" />
        <label class="sr-only" for="skill-picker-filter">Find a skill</label>
        <input
          id="skill-picker-filter"
          ref="filterField"
          v-model="filter"
          type="search"
          autocomplete="off"
          spellcheck="false"
          placeholder="Find a skill"
          class="focus-ring h-7 min-w-0 flex-1 rounded-control border border-border bg-canvas px-2.5 text-small text-text placeholder:text-text-subtle"
          data-testid="skill-picker-filter"
        />
        <span
          class="shrink-0 font-mono text-micro tabular-nums text-text-subtle"
          data-testid="skill-picker-count"
        >
          {{ modelValue.length }} / {{ SKILLS_PER_OWNER_MAX }}
        </span>
      </div>

      <div v-if="missing.length" class="border-b border-border px-3 py-2" data-testid="skill-picker-missing">
        <p class="eyebrow mb-1.5 text-micro">Not in ~/.claude any more</p>
        <div class="flex flex-wrap gap-1.5">
          <SkillChip v-for="name in missing" :key="name" :name="name" missing @remove="remove(name)" />
        </div>
      </div>

      <div class="min-h-0 flex-1 overflow-y-auto py-1">
        <p v-if="!loaded" class="px-3.5 py-3 text-caption text-text-subtle">Reading your skills.</p>
        <p
          v-else-if="!options.length"
          class="px-3.5 py-3 text-caption leading-relaxed text-text-subtle"
          data-testid="skill-picker-empty"
        >
          No skills found under ~/.claude. Add one from the Claude screen.
        </p>
        <p
          v-else-if="!shown.length"
          class="px-3.5 py-3 text-caption text-text-subtle"
          data-testid="skill-picker-empty"
        >
          No skill matches that.
        </p>
        <div v-else role="listbox" aria-multiselectable="true" :aria-label="title">
          <section v-for="group in groups" :key="group.provider">
            <h3 class="eyebrow sticky top-0 bg-surface px-3.5 pb-1 pt-2 text-micro">{{ group.provider }}</h3>
            <button
              v-for="option in group.options"
              :key="option.name"
              type="button"
              role="option"
              class="focus-ring relative flex w-full items-start gap-2.5 px-3.5 py-1.5 text-left transition-colors disabled:cursor-default"
              :class="indexOf(option) === highlighted ? 'bg-surface-raised' : 'hover:bg-surface-raised'"
              :aria-selected="chosen(option) || fromWiderScope(option)"
              :disabled="fromWiderScope(option) || (full && !chosen(option))"
              :data-row-index="indexOf(option)"
              data-testid="skill-picker-row"
              @mousemove="highlighted = indexOf(option)"
              @click="toggle(option)"
            >
              <span
                class="mt-0.5 grid size-4 shrink-0 place-items-center rounded-key border"
                :class="
                  chosen(option) || fromWiderScope(option)
                    ? 'border-anchor bg-anchor text-canvas'
                    : 'border-border-strong'
                "
                aria-hidden="true"
              >
                <svg
                  v-if="chosen(option) || fromWiderScope(option)"
                  class="size-2.5"
                  viewBox="0 0 10 10"
                  fill="none"
                >
                  <path
                    d="M2 5.2l2 2 4-4.4"
                    stroke="currentColor"
                    stroke-width="1.6"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  />
                </svg>
              </span>
              <span class="min-w-0 flex-1">
                <span class="flex min-w-0 items-center gap-2">
                  <span class="truncate font-mono text-small font-medium text-text">{{ option.name }}</span>
                  <span v-if="fromWiderScope(option)" class="shrink-0 text-micro text-text-subtle"
                    >already on the task</span
                  >
                </span>
                <span v-if="option.description" class="line-clamp-2 text-caption text-text-subtle">{{
                  option.description
                }}</span>
              </span>
            </button>
          </section>
        </div>
      </div>

      <div class="flex items-center justify-between gap-2 border-t border-border bg-canvas px-3.5 py-2">
        <span class="text-micro text-text-subtle">
          <kbd class="rounded-key border border-border px-1 font-mono text-micro">↑↓</kbd>
          move,
          <kbd class="rounded-key border border-border px-1 font-mono text-micro">↵</kbd>
          tick or untick
        </span>
        <button
          type="button"
          class="focus-ring rounded-key px-1.5 text-micro font-medium text-text-muted transition-colors hover:text-text"
          data-testid="skill-picker-done"
          @click="emit('close')"
        >
          Done
        </button>
      </div>
    </div>
  </Teleport>
</template>
