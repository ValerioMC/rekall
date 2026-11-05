<script setup lang="ts">
import { ref } from 'vue'
import AppNavSwitcher from '@/components/console/AppNavSwitcher.vue'
import HistoryArrows from '@/components/shell/HistoryArrows.vue'
import { useDismissOnOutsidePress } from '@/composables/useDismissOnOutsidePress'

const host = ref<HTMLElement | null>(null)
const expanded = ref(false)

useDismissOnOutsidePress(host, expanded, () => (expanded.value = false))
</script>

<!-- Folded, the menu key stands beside the history arrows. Pressed, the screen tabs take the
     arrows' place; a press anywhere outside this block folds it back. -->
<template>
  <div ref="host" class="flex min-w-0 flex-1 items-center gap-2.5 self-stretch" data-testid="header-navigation">
    <button
      type="button"
      class="focus-ring grid size-8 shrink-0 place-items-center rounded-control transition-colors"
      :class="expanded ? 'bg-surface-raised text-text' : 'text-text-muted hover:bg-surface-hover hover:text-text'"
      :aria-expanded="expanded"
      aria-label="Menu"
      title="Menu"
      data-testid="header-menu-toggle"
      @click="expanded = !expanded"
    >
      <svg class="size-4" viewBox="0 0 16 16" fill="none" aria-hidden="true">
        <path d="M2.5 4h11M2.5 8h11M2.5 12h11" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
      </svg>
    </button>
    <div v-if="expanded" class="@container flex min-w-0 flex-1 self-stretch">
      <AppNavSwitcher />
    </div>
    <HistoryArrows v-else />
  </div>
</template>
