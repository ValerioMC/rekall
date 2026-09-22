<script setup lang="ts">
/**
 * The cross on anything that closes or removes: a dialog's corner, a chip's tail, a row's delete.
 *
 * Drawn rather than typed. The `×` character sits on the text baseline, so in a square button it
 * rides a pixel or two low and takes whatever weight the surrounding font gives it; two strokes
 * on a 12-unit box are centred by construction and match the 1.2-1.5 stroke every other glyph
 * here is drawn at. `small` is for the round remove buttons inside chips and rows, where the full
 * size would crowd the label it sits beside.
 */
withDefaults(defineProps<{ small?: boolean }>(), { small: false })
</script>

<template>
  <svg
    :class="small ? 'size-2.5' : 'size-3.5'"
    viewBox="0 0 12 12"
    fill="none"
    aria-hidden="true"
    data-testid="close-glyph"
  >
    <path
      d="M3 3l6 6M9 3l-6 6"
      stroke="currentColor"
      :stroke-width="small ? 1.6 : 1.4"
      stroke-linecap="round"
    />
  </svg>
</template>
