<script setup lang="ts">
/** `parent` is what the title sits inside (a project's company): shown before it, stepped back. */
withDefaults(defineProps<{ title: string; parent?: string | null }>(), { parent: null })
</script>

<template>
  <!-- The logo/tabs chrome is rendered once by `AppHeaderChrome`, outside the router view, so it
       never unmounts between screens. This teleports the catalog-specific title and actions into
       its `#app-header-extra` slot instead of drawing a second header. -->
  <Teleport to="#app-header-extra">
    <!-- Two children, for the header's centre and right cells. A hairline, then the title with
         the same short accent tick the panes hang on their kicker: the switcher says which
         screen, the title says what is on it. -->
    <div class="flex min-w-0 items-center gap-3">
      <span class="h-5 w-px shrink-0 bg-border-strong" aria-hidden="true" />
      <h1 class="flex min-w-0 items-center gap-2 text-title font-semibold tracking-[-0.01em] text-text">
        <slot name="leading">
          <span class="h-3 w-[3px] shrink-0 rounded-full bg-accent" aria-hidden="true" />
        </slot>
        <span v-if="parent" class="shrink-0 font-medium text-text-subtle">
          {{ parent }}<span class="px-1.5 text-border-strong" aria-hidden="true">/</span>
        </span>
        <span class="truncate">{{ title }}</span>
      </h1>
      <slot name="title-suffix" />
    </div>

    <div class="flex min-w-0 items-center justify-end gap-3">
      <slot name="actions" />
    </div>
  </Teleport>
</template>
