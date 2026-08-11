<script setup lang="ts">
import type { Entity } from '@/model/schema'

defineProps<{
  entities: readonly Entity[]
  appliedEntities: readonly Entity[]
  pendingCount: number
}>()

/**
 * The active state is expressed as a group variant rather than scoped CSS, so the whole
 * appearance of a navigation item lives in one place instead of being split between a class
 * list and a stylesheet.
 */
const NAV_ITEM =
  'group focus-ring flex items-center gap-2.5 rounded-[var(--radius-control)] px-2 py-1.5 ' +
  'text-[13px] text-text-muted transition-colors hover:bg-surface-raised hover:text-text'

const NAV_ACTIVE = 'is-active bg-surface-hover text-text'

const NAV_DOT =
  'size-[5px] shrink-0 rounded-full bg-border-strong transition-all ' +
  'group-[.is-active]:bg-accent group-[.is-active]:shadow-[0_0_10px_var(--color-accent)]'
</script>

<template>
  <aside
    class="flex w-(--spacing-sidebar) shrink-0 flex-col gap-7 overflow-y-auto border-r border-border bg-surface px-3.5 py-5"
  >
    <div class="flex items-center gap-2.5 px-2">
      <div
        class="halo grid size-8 place-items-center rounded-[10px] bg-gradient-to-br from-accent-strong to-accent-deep text-[15px] font-bold text-accent-ink"
        aria-hidden="true"
      >
        R
      </div>
      <div class="leading-tight">
        <div class="text-[15px] font-semibold tracking-[-0.01em] text-text">Rekall</div>
        <div class="text-[11px] text-text-subtle">structured memory</div>
      </div>
    </div>

    <nav aria-label="Design">
      <p class="mb-1.5 px-2 text-[10px] font-semibold uppercase tracking-[0.09em] text-text-subtle">
        Design
      </p>
      <RouterLink to="/schema" :class="NAV_ITEM" :active-class="NAV_ACTIVE">
        <span :class="NAV_DOT" aria-hidden="true" />
        Schema
        <span class="ml-auto font-mono text-[11px] text-text-subtle">{{ entities.length }}</span>
      </RouterLink>
      <RouterLink to="/plan" :class="NAV_ITEM" :active-class="NAV_ACTIVE">
        <span :class="NAV_DOT" aria-hidden="true" />
        Plan
        <span
          v-if="pendingCount > 0"
          class="ml-auto rounded-md bg-accent-soft px-1.5 py-0.5 font-mono text-[10px] text-accent ring-1 ring-inset ring-accent/25"
        >
          {{ pendingCount }}
        </span>
      </RouterLink>
    </nav>

    <nav aria-label="Content">
      <p class="mb-1.5 px-2 text-[10px] font-semibold uppercase tracking-[0.09em] text-text-subtle">
        Content
      </p>
      <RouterLink to="/search" :class="NAV_ITEM" :active-class="NAV_ACTIVE">
        <span :class="NAV_DOT" aria-hidden="true" />
        Search
      </RouterLink>
      <RouterLink
        v-for="entity in appliedEntities"
        :key="entity.id"
        :to="`/data/${entity.physicalName}`"
        :class="NAV_ITEM"
        :active-class="NAV_ACTIVE"
      >
        <span :class="NAV_DOT" aria-hidden="true" />
        {{ entity.labelPlural }}
      </RouterLink>
      <p v-if="!appliedEntities.length" class="px-2 py-1.5 text-[12px] text-text-subtle">
        nothing applied yet
      </p>
    </nav>

    <!-- Discoverability: a shortcut nobody knows about is a shortcut nobody uses. -->
    <p class="mt-auto flex items-center gap-1.5 px-2 pt-4 text-[11px] text-text-subtle">
      <kbd class="rounded border border-border px-1.5 py-0.5 font-mono text-[10px]">&#8984;K</kbd>
      to jump anywhere
    </p>
  </aside>
</template>
