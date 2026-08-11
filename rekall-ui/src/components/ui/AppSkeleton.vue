<script setup lang="ts">
withDefaults(defineProps<{ variant?: 'cards' | 'table' | 'list'; rows?: number }>(), {
  variant: 'cards',
  rows: 5
})
</script>

<template>
  <!--
    A placeholder shaped like the content it stands in for. A spinner tells you something is
    happening; this tells you what is about to arrive, and reserves its space so the page does
    not jump when it does.
  -->
  <div role="status" aria-live="polite" aria-busy="true">
    <span class="sr-only">Loading</span>

    <div v-if="variant === 'cards'" class="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
      <div
        v-for="index in 3"
        :key="index"
        class="rounded-[var(--radius-card)] border border-border bg-surface p-5"
      >
        <div class="skeleton h-4 w-1/3" />
        <div class="skeleton mt-3 h-3 w-4/5" />
        <div class="mt-4 flex gap-1.5">
          <div v-for="tag in 3" :key="tag" class="skeleton h-5 w-16" />
        </div>
      </div>
    </div>

    <div
      v-else-if="variant === 'table'"
      class="overflow-hidden rounded-[var(--radius-card)] border border-border bg-surface"
    >
      <div class="border-b border-border px-4 py-2.5">
        <div class="skeleton h-2.5 w-24" />
      </div>
      <div
        v-for="index in rows"
        :key="index"
        class="flex items-center gap-6 border-b border-border/60 px-4 last:border-0"
        :style="{ height: 'var(--spacing-row)' }"
      >
        <div class="skeleton h-3 w-40" />
        <div class="skeleton h-3 w-24" />
        <div class="skeleton h-3 w-32" />
      </div>
    </div>

    <div v-else class="space-y-2.5">
      <div
        v-for="index in rows"
        :key="index"
        class="rounded-[var(--radius-card)] border border-border bg-surface p-5"
      >
        <div class="skeleton h-3.5 w-1/2" />
        <div class="skeleton mt-3 h-14 w-full" />
      </div>
    </div>
  </div>
</template>
