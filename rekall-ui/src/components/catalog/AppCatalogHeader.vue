<script setup lang="ts">
import AppLogo from '@/components/ui/AppLogo.vue'
import AppNavSwitcher from '@/components/console/AppNavSwitcher.vue'

/** `parent` is what the title sits inside (a project's company): shown before it, stepped back. */
withDefaults(defineProps<{ title: string; parent?: string | null }>(), { parent: null })
</script>

<template>
  <header
    class="glass sticky top-0 z-(--z-sticky) flex h-(--spacing-header) shrink-0 items-center gap-3.5 border-b border-border px-4"
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

    <AppNavSwitcher />

    <!-- A hairline, then the title with the same short accent tick the panes hang on their
         kicker: the switcher says which screen, the title says what is on it, and the two no
         longer run together as one string of words. -->
    <span class="h-5 w-px shrink-0 bg-border-strong" aria-hidden="true" />

    <div class="flex min-w-0 flex-1 items-center gap-3">
      <h1 class="flex min-w-0 items-center gap-2 text-[15px] font-semibold tracking-[-0.01em] text-text">
        <span class="h-3 w-[3px] shrink-0 rounded-full bg-accent" aria-hidden="true" />
        <span v-if="parent" class="shrink-0 font-medium text-text-subtle">
          {{ parent }}<span class="px-1.5 text-border-strong" aria-hidden="true">/</span>
        </span>
        <span class="truncate">{{ title }}</span>
      </h1>
      <slot name="title-suffix" />
    </div>

    <div class="ml-auto flex shrink-0 items-center gap-3">
      <slot name="actions" />
    </div>
  </header>
</template>
