<script setup lang="ts">
/*
 * The brand mark, in one place so the header, the first-run screen and public/favicon.svg
 * cannot drift apart. Change the geometry here and in the favicon together.
 *
 * The mark is the product: an open ring is the context Rekall holds, the rounded record at
 * the centre is the one task in focus, and the cyan dot in the ring's opening is the anchor —
 * the same colour `project:vega` is drawn in everywhere else, and the only cyan on the mark.
 *
 * Gradient ids are per-instance. Two logos on one page with the same ids would make the
 * second one paint with the first one's fills.
 */
import { useId } from 'vue'

withDefaults(defineProps<{ size?: number }>(), { size: 32 })

const uid = useId()
</script>

<template>
  <svg
    :width="size"
    :height="size"
    viewBox="0 0 512 512"
    role="img"
    aria-label="Rekall"
    class="shrink-0"
  >
    <defs>
      <linearGradient :id="`${uid}-tile`" x1="0.1" y1="0" x2="0.85" y2="1">
        <stop offset="0" stop-color="#1e2634" />
        <stop offset="0.52" stop-color="#12161d" />
        <stop offset="1" stop-color="#08090c" />
      </linearGradient>
      <linearGradient :id="`${uid}-sheen`" x1="0" y1="0" x2="0" y2="1">
        <stop offset="0" stop-color="#ffffff" stop-opacity="0.1" />
        <stop offset="0.55" stop-color="#ffffff" stop-opacity="0" />
      </linearGradient>
      <linearGradient :id="`${uid}-ring`" x1="0.08" y1="0" x2="0.92" y2="1">
        <stop offset="0" stop-color="#ffd08a" />
        <stop offset="0.44" stop-color="#f5a524" />
        <stop offset="1" stop-color="#a8681a" />
      </linearGradient>
      <linearGradient :id="`${uid}-core`" x1="0" y1="0" x2="1" y2="1">
        <stop offset="0" stop-color="#ffdca4" />
        <stop offset="1" stop-color="#e09a20" />
      </linearGradient>
      <linearGradient :id="`${uid}-node`" x1="0.15" y1="0" x2="0.85" y2="1">
        <stop offset="0" stop-color="#c9f2f9" />
        <stop offset="1" stop-color="#4cc3d5" />
      </linearGradient>
      <radialGradient :id="`${uid}-glow`">
        <stop offset="0.5" stop-color="#5fd0e0" stop-opacity="0.28" />
        <stop offset="1" stop-color="#5fd0e0" stop-opacity="0" />
      </radialGradient>
    </defs>

    <rect width="512" height="512" rx="115" :fill="`url(#${uid}-tile)`" />
    <rect width="512" height="512" rx="115" :fill="`url(#${uid}-sheen)`" />
    <rect
      x="3"
      y="3"
      width="506"
      height="506"
      rx="112.5"
      fill="none"
      stroke="#ffd9a8"
      stroke-opacity="0.16"
      stroke-width="6"
    />

    <path
      d="M362.79 178.41 A132 132 0 1 0 362.79 333.59"
      fill="none"
      :stroke="`url(#${uid}-ring)`"
      stroke-width="56"
      stroke-linecap="round"
    />

    <rect x="200" y="200" width="112" height="112" rx="34" :fill="`url(#${uid}-core)`" />

    <circle cx="388" cy="256" r="76" :fill="`url(#${uid}-glow)`" />
    <circle cx="388" cy="256" r="38" :fill="`url(#${uid}-node)`" />
  </svg>
</template>
