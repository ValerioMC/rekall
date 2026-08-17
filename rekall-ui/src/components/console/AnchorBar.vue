<script setup lang="ts">
import { computed, ref } from 'vue'
import { storeToRefs } from 'pinia'
import AppButton from '@/components/ui/AppButton.vue'
import AppLogo from '@/components/ui/AppLogo.vue'
import { useConsoleStore } from '@/stores/console.store'

/**
 * The spine of the application.
 *
 * Not a palette you summon and dismiss: a field that is always there, in the same grammar as
 * the MCP tool. What you type here is what you would type after `/rk`, which is the one thing
 * this product has that nothing else does.
 */
const emit = defineEmits<{ newNote: []; openSettings: [] }>()

const store = useConsoleStore()
const {
  filter,
  scopePath,
  scopeName,
  saveState,
  navMode,
  visibleTasks,
  visibleDocuments,
  selectedTaskId
} = storeToRefs(store)

const input = ref<HTMLInputElement | null>(null)
const isFocused = ref(false)

const placeholder = computed(() =>
  scopePath.value.length === 0
    ? 'company:acme project:vega task:report-builder'
    : `Find in ${scopeName.value}`
)

const resultCount = computed(() =>
  navMode.value === 'tasks' ? visibleTasks.value.length : visibleDocuments.value.length
)

const SAVE_LABEL = { saved: 'Saved', unsaved: 'Unsaved', saving: 'Saving' } as const

const SAVE_DOT = {
  saved: 'bg-safe',
  unsaved: 'bg-warn',
  saving: 'bg-accent animate-pulse'
} as const

/** Enter opens the first thing the filter found, so search and navigation are one gesture. */
function openFirst(): void {
  if (navMode.value === 'notes') {
    const first = visibleDocuments.value[0]
    if (first) store.selectDocument(first.id)
    return
  }
  const first = visibleTasks.value[0]
  if (first) store.selectTask(first.id)
}

function clear(): void {
  filter.value = ''
  input.value?.blur()
}

defineExpose({ focus: () => { input.value?.focus(); input.value?.select() } })
</script>

<template>
  <header
    class="glass flex h-(--spacing-header) shrink-0 items-center gap-3.5 border-b border-border px-4"
  >
    <div class="flex w-(--spacing-nav) shrink-0 items-center gap-2.5 pr-2.5">
      <AppLogo :size="32" class="halo rounded-[7px]" />
      <span class="min-w-0">
        <span class="block text-[14.5px] font-semibold leading-tight tracking-[-0.015em] text-text">
          Rekall
        </span>
        <span class="block font-mono text-[10px] leading-tight text-text-subtle">
          context, anchored
        </span>
      </span>
    </div>

    <div class="relative max-w-[640px] flex-1">
      <span
        class="pointer-events-none absolute left-3.5 top-1/2 -translate-y-1/2 font-mono text-[13px] text-anchor"
        aria-hidden="true"
      >
        /rk
      </span>
      <input
        ref="input"
        v-model="filter"
        data-testid="anchor-input"
        type="text"
        autocomplete="off"
        spellcheck="false"
        class="focus-ring h-10 w-full rounded-[var(--radius-control)] border bg-canvas pl-[48px] pr-[92px] font-mono text-[13px] text-text outline-none transition-all placeholder:text-text-subtle"
        :class="
          isFocused
            ? 'border-accent shadow-[0_0_0_3px_var(--color-accent-soft)]'
            : 'border-border hover:border-border-strong'
        "
        :placeholder="placeholder"
        aria-label="Find a task or a note"
        @focus="isFocused = true"
        @blur="isFocused = false"
        @keydown.enter="openFirst"
        @keydown.esc="clear"
      />
      <!-- The count while searching, the shortcut while not: the same corner, never both. -->
      <span
        class="pointer-events-none absolute right-2.5 top-1/2 flex -translate-y-1/2 items-center gap-1"
        aria-hidden="true"
      >
        <template v-if="filter.trim()">
          <span class="font-mono text-[11px] text-text-subtle">{{ resultCount }} found</span>
        </template>
        <template v-else>
          <kbd
            class="rounded border border-border bg-surface-raised px-1.5 py-0.5 font-mono text-[10px] text-text-subtle"
          >
            &#8984;
          </kbd>
          <kbd
            class="rounded border border-border bg-surface-raised px-1.5 py-0.5 font-mono text-[10px] text-text-subtle"
          >
            K
          </kbd>
        </template>
      </span>
    </div>

    <div class="ml-auto flex items-center gap-3">
      <p
        class="flex min-w-[92px] items-center gap-2 text-[12px]"
        :class="saveState === 'unsaved' ? 'text-warn' : 'text-text-subtle'"
        role="status"
        aria-live="polite"
        data-testid="save-state"
      >
        <span
          class="size-1.5 rounded-full transition-colors"
          :class="SAVE_DOT[saveState]"
          aria-hidden="true"
        />
        {{ SAVE_LABEL[saveState] }}
      </p>
      <!--
        A plain link, not a fetch: the browser is already good at saving a file, and routing the
        bytes through the client to rebuild a download would only add ways to fail.
      -->
      <a
        href="/api/export"
        download
        data-testid="export-link"
        class="focus-ring inline-flex h-8 items-center gap-1.5 rounded-[var(--radius-control)] border border-border-strong bg-surface-raised px-3 text-[12.5px] text-text transition-colors hover:border-accent hover:bg-surface-hover"
        title="Download every company, project, task and note as a folder tree"
      >
        <svg class="size-3.5" viewBox="0 0 24 24" fill="none" aria-hidden="true">
          <path
            d="M12 3v12m0 0l-4-4m4 4l4-4M4 19h16"
            stroke="currentColor"
            stroke-width="1.8"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
        Export
      </a>
      <RouterLink
        to="/projects"
        data-testid="open-catalog"
        class="focus-ring inline-flex h-8 items-center gap-1.5 rounded-[var(--radius-control)] border border-border-strong bg-surface-raised px-3 text-[12.5px] text-text transition-colors hover:border-accent hover:bg-surface-hover"
        title="Browse and edit companies and projects"
      >
        <svg class="size-3.5" viewBox="0 0 24 24" fill="none" aria-hidden="true">
          <rect x="3.5" y="4" width="7" height="7" rx="1.3" stroke="currentColor" stroke-width="1.6" />
          <rect x="13.5" y="4" width="7" height="7" rx="1.3" stroke="currentColor" stroke-width="1.6" />
          <rect x="3.5" y="13" width="7" height="7" rx="1.3" stroke="currentColor" stroke-width="1.6" />
          <rect x="13.5" y="13" width="7" height="7" rx="1.3" stroke="currentColor" stroke-width="1.6" />
        </svg>
        Projects
      </RouterLink>
      <button
        type="button"
        data-testid="settings-trigger"
        aria-label="Settings"
        title="Settings"
        class="focus-ring grid size-8 shrink-0 place-items-center rounded-[var(--radius-control)] border border-border-strong bg-surface-raised text-text-subtle transition-colors hover:border-accent hover:bg-surface-hover hover:text-text"
        @click="emit('openSettings')"
      >
        <svg class="size-4" viewBox="0 0 24 24" fill="none" aria-hidden="true">
          <!--
            The Lucide `settings` gear, unmodified. The path it replaced was the Feather gear
            with its coordinates hand-edited to fit a smaller box, and three of those edits had
            broken it: two pairs of arcs collapsed into single large-arc ones, and one arc
            rewritten as a cubic, which is the spur that used to stick out of the right side.
            Rescale with the viewBox, never by editing the numbers.
          -->
          <path
            d="M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6Z"
            stroke="currentColor"
            stroke-width="1.6"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
          <path
            d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"
            stroke="currentColor"
            stroke-width="1.6"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
      </button>
      <span class="group relative inline-block">
        <AppButton
          variant="primary"
          size="sm"
          :disabled="!selectedTaskId"
          @click="emit('newNote')"
        >
          New note
        </AppButton>
        <span
          v-if="!selectedTaskId"
          class="pointer-events-none absolute right-0 top-full z-(--z-sticky) mt-2 whitespace-nowrap rounded-[var(--radius-control)] border border-border-strong bg-surface-raised px-2.5 py-1.5 text-xs text-text opacity-0 shadow-lift transition-opacity duration-100 group-hover:opacity-100"
        >
          Select a task to create a new note on it
        </span>
      </span>
    </div>
  </header>
</template>
