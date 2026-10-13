<script setup lang="ts">
import { ref } from 'vue'
import { inspectorGroupCollapsed, setInspectorGroupCollapsed } from '@/common/config/inspector-groups'

/**
 * One concern of the project page's inspector: a heading that folds its body away, remembered per
 * browser. Folded, `summary` stands in for the body so the group still says what it holds.
 * `group` also names the element ids and test ids, so each group is addressable on its own.
 */
const props = withDefaults(defineProps<{ group: string; title: string; count?: number | null; summary?: string }>(), {
  count: null,
  summary: ''
})

const collapsed = ref(inspectorGroupCollapsed(props.group))

function toggle(): void {
  collapsed.value = !collapsed.value
  setInspectorGroupCollapsed(props.group, collapsed.value)
}

/** Opens a folded group, for an action in its heading that adds something to the body. */
function expand(): void {
  if (!collapsed.value) return
  toggle()
}

defineExpose({ expand })
</script>

<template>
  <section class="px-4 py-3" :aria-labelledby="`${group}-heading`" :data-testid="`inspector-${group}`">
    <div class="flex min-h-7 items-center justify-between gap-2">
      <button
        type="button"
        class="focus-ring -ml-1.5 flex min-w-0 cursor-pointer items-center gap-1.5 rounded-control px-1.5 py-1 text-left"
        :aria-expanded="!collapsed"
        :aria-controls="`${group}-body`"
        :data-testid="`${group}-toggle`"
        @click="toggle"
      >
        <svg
          class="size-3 shrink-0 text-text-subtle transition-transform duration-(--motion-quick)"
          :class="{ '-rotate-90': collapsed }"
          viewBox="0 0 14 14"
          fill="none"
          aria-hidden="true"
        >
          <path d="m3.5 5.5 3.5 3.5 3.5-3.5" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
        <h2 :id="`${group}-heading`" class="eyebrow text-caption uppercase tracking-[0.06em]">{{ title }}</h2>
        <span
          v-if="count"
          class="rounded-full bg-surface-raised px-1.5 py-px font-mono text-micro tabular-nums text-text-subtle"
          :data-testid="`${group}-count`"
        >
          {{ count }}
        </span>
      </button>
      <slot name="actions" />
    </div>

    <p v-if="collapsed && summary" class="mt-0.5 truncate pl-5 text-small text-text-subtle" :data-testid="`${group}-summary`">
      {{ summary }}
    </p>

    <div v-show="!collapsed" :id="`${group}-body`" class="mt-2">
      <slot />
    </div>
  </section>
</template>
