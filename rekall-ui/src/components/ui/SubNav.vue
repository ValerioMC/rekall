<script setup lang="ts" generic="Id extends string">
import { RouterLink } from 'vue-router'

export type SubNavItem<Id extends string> = Readonly<{
  id: Id
  label: string
  count?: number
  /** The count waits on the reader: drawn as an amber pill instead of a quiet figure. */
  hot?: boolean
  to?: string
}>

/**
 * Sub-navigation under a page title: tabs with an amber rule under the one you are on and an
 * optional count. An item with `to` is a link, so a tab can be a route; otherwise it is a key.
 */
defineProps<{ items: readonly SubNavItem<Id>[]; label: string }>()

const active = defineModel<Id>({ required: true })
</script>

<template>
  <div class="flex gap-0.5 border-b border-border" role="tablist" :aria-label="label" data-testid="sub-nav">
    <component
      :is="item.to ? RouterLink : 'button'"
      v-for="item in items"
      :key="item.id"
      v-bind="item.to ? { to: item.to } : { type: 'button' }"
      role="tab"
      class="focus-ring relative flex cursor-pointer items-center gap-2 px-3 pb-2.5 pt-2 text-small transition-colors duration-(--motion-quick) after:absolute after:inset-x-2 after:-bottom-px after:h-0.5 after:rounded-full after:bg-accent after:transition-transform after:duration-(--motion-move)"
      :class="active === item.id ? 'text-text after:scale-x-100' : 'text-text-muted after:scale-x-0 hover:text-text'"
      :aria-selected="active === item.id"
      :data-testid="`sub-nav-${item.id}`"
      @click="active = item.id"
    >
      {{ item.label }}
      <span
        v-if="item.count !== undefined"
        class="font-mono text-caption tabular-nums"
        :class="item.hot && item.count > 0 ? 'rounded-full bg-accent px-1.5 font-medium leading-4 text-accent-ink' : 'text-text-subtle'"
        data-testid="sub-nav-count"
      >
        {{ item.count }}
      </span>
    </component>
  </div>
</template>
