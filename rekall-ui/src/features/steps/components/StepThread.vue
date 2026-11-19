<script setup lang="ts">
/**
 * The line that ties the steps of one Claude session together: it runs through the seals, so one
 * unbroken thread reads as one context, and a break in it is where a fresh session starts. Its
 * look follows the session it sits in (`data-standing` on the session group).
 */
defineProps<{
  /** `above` reaches the seal from the row's top edge, `below` leaves it to the bottom edge, `through` fills a gap, `from-orb` leaves a session header's orb. */
  placement: 'above' | 'below' | 'through' | 'from-orb'
}>()

const PLACEMENT = {
  above: 'top-0 h-[19px]',
  below: 'top-[19px] bottom-0',
  through: 'inset-y-0',
  'from-orb': 'top-[18px] bottom-0'
} as const
</script>

<template>
  <span
    class="pointer-events-none absolute left-[18px] z-0 w-0.5 rounded-full bg-session/60 group-data-[standing=accepted]/session:bg-border-strong group-data-[standing=idle]/session:w-0 group-data-[standing=idle]/session:border-l-2 group-data-[standing=idle]/session:border-dashed group-data-[standing=idle]/session:border-border-field group-data-[standing=idle]/session:bg-transparent"
    :class="PLACEMENT[placement]"
    aria-hidden="true"
    data-testid="session-thread"
  />
</template>
