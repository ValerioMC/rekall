<script setup lang="ts">
/**
 * The hint on anything copyable: two stacked sheets, which become a check for the moment after
 * the click. It carries no text of its own; the button it sits in says what gets copied.
 */
defineProps<{ copied: boolean }>()
</script>

<template>
  <span
    class="relative grid size-3.5 shrink-0 place-items-center"
    :class="copied ? 'text-accent' : 'opacity-70'"
    data-testid="copy-glyph"
    :data-copied="copied ? 'true' : 'false'"
    aria-hidden="true"
  >
    <Transition name="copy-glyph" mode="out-in">
      <svg v-if="copied" key="check" class="size-3.5" viewBox="0 0 12 12" fill="none">
        <path
          d="M2.1 6.3 4.6 8.8 9.9 3.4"
          stroke="currentColor"
          stroke-width="1.5"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
      <svg v-else key="sheets" class="size-3.5" viewBox="0 0 12 12" fill="none">
        <rect x="4" y="4" width="6.5" height="6.5" rx="1.4" stroke="currentColor" stroke-width="1.2" />
        <path
          d="M8 3.2V2.6A1.1 1.1 0 0 0 6.9 1.5H2.6A1.1 1.1 0 0 0 1.5 2.6v4.3A1.1 1.1 0 0 0 2.6 8h.6"
          stroke="currentColor"
          stroke-width="1.2"
          stroke-linecap="round"
        />
      </svg>
    </Transition>
  </span>
</template>

<style scoped>
.copy-glyph-enter-active,
.copy-glyph-leave-active {
  transition:
    opacity 110ms ease,
    transform 110ms ease;
}
.copy-glyph-enter-from,
.copy-glyph-leave-to {
  opacity: 0;
  transform: scale(0.55);
}
</style>
