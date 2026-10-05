<script setup lang="ts">
import { nextTick, ref, watch } from 'vue'
import type { Crumb } from '@/composables/useDirectoryBrowser'

/**
 * Where the picker is: the breadcrumb, scrolled to its deep end, and the two places most often
 * wanted instead (the project, home), with the switch for hidden files.
 */
const props = defineProps<{
  crumbs: readonly Crumb[]
  hasProject: boolean
  atProject: boolean
  atHome: boolean
  showHidden: boolean
}>()

defineEmits<{ crumb: [index: number]; project: []; home: []; 'toggle-hidden': [] }>()

const crumbsEl = ref<HTMLElement | null>(null)

/** A path is read from its end: the breadcrumb keeps its last characters in view. */
watch(
  () => props.crumbs,
  () =>
    void nextTick(() => {
      const element = crumbsEl.value
      if (!element) return
      element.scrollLeft = element.scrollWidth
      element.dataset.overflowing = String(element.scrollWidth > element.clientWidth + 1)
    })
)

const PLACE_BUTTON = 'focus-ring grid size-7 shrink-0 place-items-center rounded-[7px] transition-colors'
const ACTIVE = 'bg-accent-soft text-accent'
const IDLE = 'text-text-subtle hover:bg-surface-hover hover:text-text'
</script>

<template>
  <div class="flex h-9 shrink-0 items-center gap-1 border-b border-border pl-1.5 pr-1.5">
    <nav ref="crumbsEl" class="path-crumbs min-w-0 flex-1 overflow-x-auto" aria-label="Folder in view" data-testid="path-picker-crumbs">
      <ol class="flex w-max items-center font-mono text-[11px] leading-none">
        <li v-for="(crumb, index) in crumbs" :key="crumb.path" class="flex items-center">
          <span v-if="index > 0 && !/[\\/]$/.test(crumbs[index - 1]!.label)" class="px-px text-text-subtle/50" aria-hidden="true">/</span>
          <button
            type="button"
            class="focus-ring flex h-6 items-center gap-1 rounded-[6px] px-1.5 transition-colors"
            :class="index === crumbs.length - 1 ? 'text-text' : 'text-text-subtle hover:bg-surface-hover hover:text-text'"
            :aria-current="index === crumbs.length - 1 ? 'location' : undefined"
            :title="crumb.path"
            :data-project="crumb.project || undefined"
            data-testid="path-picker-crumb"
            @click="$emit('crumb', index)"
          >
            <!-- The project's own folder carries the accent dot: the place this picker is about. -->
            <span v-if="crumb.project" class="crumb-project-dot" aria-hidden="true" />
            {{ crumb.label }}
          </button>
        </li>
      </ol>
    </nav>

    <button
      v-if="hasProject"
      type="button"
      :class="[PLACE_BUTTON, atProject ? ACTIVE : IDLE]"
      :aria-pressed="atProject"
      title="Project folder"
      aria-label="Go to the project folder"
      data-testid="path-picker-project"
      @click="$emit('project')"
    >
      <svg class="size-3.5" viewBox="0 0 16 16" fill="none" aria-hidden="true">
        <path
          d="M1.75 4.25c0-.55.45-1 1-1h3.1l1.4 1.5h6c.55 0 1 .45 1 1v6.5c0 .55-.45 1-1 1H2.75c-.55 0-1-.45-1-1v-8Z"
          stroke="currentColor"
          stroke-width="1.25"
          stroke-linejoin="round"
        />
        <circle cx="8" cy="8.9" r="1.35" fill="currentColor" />
      </svg>
    </button>
    <button
      type="button"
      :class="[PLACE_BUTTON, atHome ? ACTIVE : IDLE]"
      :aria-pressed="atHome"
      title="Home folder  ⌘⇧H"
      aria-label="Go to your home folder"
      data-testid="path-picker-home"
      @click="$emit('home')"
    >
      <svg class="size-3.5" viewBox="0 0 16 16" fill="none" aria-hidden="true">
        <path
          d="M2.5 7.1 8 2.6l5.5 4.5M4 6v6.9c0 .3.2.5.5.5h2.4V10h2.2v3.4h2.4c.3 0 .5-.2.5-.5V6"
          stroke="currentColor"
          stroke-width="1.25"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
    </button>
    <button
      type="button"
      class="focus-ring grid h-7 shrink-0 place-items-center rounded-[7px] px-1.5 font-mono text-[11px] transition-colors"
      :class="showHidden ? ACTIVE : IDLE"
      :aria-pressed="showHidden"
      :title="showHidden ? 'Hide hidden files  ⌘⇧.' : 'Show hidden files  ⌘⇧.'"
      aria-label="Show hidden files"
      data-testid="path-picker-hidden"
      @click="$emit('toggle-hidden')"
    >
      .*
    </button>
  </div>
</template>

<style scoped>
/* The breadcrumb scrolls sideways under a fade rather than wrapping: the deep end is what matters. */
.path-crumbs {
  scrollbar-width: none;
}

.path-crumbs::-webkit-scrollbar {
  display: none;
}

.path-crumbs[data-overflowing='true'] {
  mask-image: linear-gradient(90deg, transparent 0, #000 22px);
}

.crumb-project-dot {
  width: 5px;
  height: 5px;
  border-radius: 999px;
  background: var(--color-accent);
  box-shadow: 0 0 6px color-mix(in srgb, var(--color-accent) 70%, transparent);
}
</style>
