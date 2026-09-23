<script setup lang="ts">
withDefaults(defineProps<{ size?: number }>(), { size: 32 })
</script>

<template>
  <!--
    The favicon itself rather than a copy of its markup, so the mark in the header, the browser
    tab and the dock icon rendered from it are the same drawing. As an <img> its gradient and
    filter ids stay inside the file, where two logos on one screen cannot collide.
  -->
  <img src="/favicon.svg" :width="size" :height="size" alt="Rekall" class="shrink-0" />
</template>
