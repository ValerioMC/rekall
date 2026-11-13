<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import HeaderStatus from '@/components/console/HeaderStatus.vue'
import SettingsTrigger from '@/components/console/SettingsTrigger.vue'
import { usePaletteStore } from '@/features/search/store/palette.store'
import { useConsoleStore } from '@/stores/console.store'

const emit = defineEmits<{ openSettings: []; openTags: [] }>()

const store = useConsoleStore()
const palette = usePaletteStore()
const { saveState, selectedTask, scopeAnchor } = storeToRefs(store)

/** Where the reader is, as the anchors a session would be handed: the task, or the scope. */
const location = computed<string[]>(() => {
  const anchors = selectedTask.value?.anchor ?? scopeAnchor.value
  return anchors.split(' ').filter((part) => part.length > 0)
})

const CURRENT_CHIP = 'min-w-[5rem] shrink-[1000] text-anchor-strong'
const PARENT_CHIP = 'min-w-[3.5rem] shrink'

const SAVE_LABEL = { saved: 'Saved', unsaved: 'Unsaved', saving: 'Saving' } as const

const SAVE_DOT = {
  saved: 'bg-safe',
  unsaved: 'bg-warn',
  saving: 'bg-accent animate-pulse'
} as const

const justSaved = ref(false)
watch(saveState, (value, previous) => {
  if (value === 'saved' && previous !== 'saved') {
    justSaved.value = false
    requestAnimationFrame(() => {
      justSaved.value = true
      setTimeout(() => (justSaved.value = false), 340)
    })
  }
})
</script>

<template>
  <Teleport to="#app-header-extra">
    <div data-header-search class="relative min-w-0">
      <button
        type="button"
        data-testid="anchor-trigger"
        class="field focus-ring flex h-9 w-full items-center gap-2 rounded-card px-3 text-left font-mono text-small text-text"
        aria-label="Search tasks, projects and notes"
        aria-haspopup="dialog"
        @click="palette.show()"
      >
        <span class="shrink-0 text-anchor" aria-hidden="true">/rk</span>
        <span
          v-if="location.length > 0"
          class="flex min-w-0 flex-1 items-center gap-1.5 overflow-hidden"
          data-testid="anchor-location"
        >
          <span class="sr-only">Showing</span>
          <template v-for="(part, index) in location" :key="part">
            <span v-if="index > 0" class="shrink-0 text-text-subtle" aria-hidden="true">&#8250;</span>
            <span
              class="anchor-chip truncate px-1.5 text-caption leading-5"
              :class="index === location.length - 1 ? CURRENT_CHIP : PARENT_CHIP"
            >
              {{ part }}
            </span>
          </template>
        </span>
        <span v-else class="min-w-0 flex-1 truncate text-text-subtle">project:vega task:setup</span>
        <kbd
          class="shrink-0 rounded-key border border-b-2 border-border-strong bg-surface-raised px-1.5 font-mono text-micro leading-4 text-text-subtle max-[1099px]:hidden"
          aria-hidden="true"
        >
          &#8984;K
        </kbd>
      </button>
    </div>

    <div class="@container flex min-w-0 items-center gap-4">
      <div class="flex min-w-0 flex-1 justify-center">
        <HeaderStatus />
      </div>

      <div class="header-capsule" role="group" aria-label="Tools">
        <!-- The dot's colour carries the save state; the word stays for screen readers and as the tooltip. -->
        <p
          class="grid size-5 place-items-center text-small"
          :class="saveState === 'unsaved' ? 'text-warn' : 'text-text-subtle'"
          :title="SAVE_LABEL[saveState]"
          role="status"
          aria-live="polite"
          data-testid="save-state"
        >
          <span
            class="size-1.5 rounded-full transition-colors"
            :class="[SAVE_DOT[saveState], justSaved && 'settle']"
            aria-hidden="true"
          />
          <span class="sr-only">{{ SAVE_LABEL[saveState] }}</span>
        </p>
        <button
          type="button"
          class="header-key focus-ring"
          data-testid="tags-trigger"
          aria-label="Tags"
          title="Tags"
          @click="emit('openTags')"
        >
          <!-- Drawn in the chrome's grey like the cog beside it: a lit crimson glyph here was the
               only coloured icon in the bar, and colour up here means state, not a menu. -->
          <svg class="size-4" viewBox="0 0 24 24" fill="none" aria-hidden="true">
            <path
              d="M3.5 12.1V4.6a1.1 1.1 0 0 1 1.1-1.1h7.5a1.1 1.1 0 0 1 .78.32l7.8 7.8a1.1 1.1 0 0 1 0 1.56l-7.5 7.5a1.1 1.1 0 0 1-1.56 0l-7.8-7.8a1.1 1.1 0 0 1-.32-.78Z"
              stroke="currentColor"
              stroke-width="1.6"
              stroke-linejoin="round"
            />
            <circle cx="8.2" cy="8.2" r="1.5" fill="currentColor" />
          </svg>
        </button>
        <SettingsTrigger @open="emit('openSettings')" />
      </div>
    </div>
  </Teleport>
</template>
