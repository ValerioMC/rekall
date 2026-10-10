<script setup lang="ts">
/**
 * A sheet of a type the console has no components for, drawn by Mermaid itself and placed in
 * the canvas world at its natural size. Mermaid sanitises its own output at its default level.
 */
defineProps<{ svg: string }>()
</script>

<template>
  <!-- eslint-disable-next-line vue/no-v-html -- Mermaid's own sanitised SVG output -->
  <svg class="native-drawing" overflow="visible" v-html="svg" />
</template>
