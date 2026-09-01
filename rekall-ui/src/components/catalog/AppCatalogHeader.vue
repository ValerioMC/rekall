<script setup lang="ts">
import AppLogo from '@/components/ui/AppLogo.vue'
import AppNavSwitcher from '@/components/console/AppNavSwitcher.vue'

defineProps<{ title: string }>()
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

    <div class="flex min-w-0 flex-1 items-center gap-3">
      <h1 class="truncate text-[15px] font-semibold tracking-[-0.01em] text-text">{{ title }}</h1>
      <slot name="title-suffix" />
    </div>

    <div class="ml-auto flex shrink-0 items-center gap-3">
      <slot name="actions" />
    </div>
  </header>
</template>
