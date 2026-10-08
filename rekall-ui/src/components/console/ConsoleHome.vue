<script setup lang="ts">
/**
 * The console with nothing open: a desk rather than a list. What runs and what waits come first,
 * because reviewing claimed work is the most frequent next action, then the notes touched last,
 * then the keys. Laid out on a container query, since the pane is 420px wide at the window's floor.
 */
import HomeSituation from '@/components/console/home/HomeSituation.vue'
import HomeReviewCard from '@/components/console/home/HomeReviewCard.vue'
import HomeSessionsCard from '@/components/console/home/HomeSessionsCard.vue'
import HomeRecentNotes from '@/components/console/home/HomeRecentNotes.vue'
import HomeKeyStrip from '@/components/console/home/HomeKeyStrip.vue'
</script>

<template>
  <div class="@container min-h-0 flex-1 overflow-y-auto" data-testid="console-home">
    <div class="mx-auto flex min-h-full max-w-[1120px] flex-col gap-8 px-6 py-8 @3xl:px-12 @3xl:py-10">
      <HomeSituation />
      <div class="grid grid-cols-1 items-start gap-4 @3xl:grid-cols-2">
        <HomeReviewCard />
        <HomeSessionsCard />
      </div>
      <HomeRecentNotes />
      <div class="mt-auto">
        <HomeKeyStrip />
      </div>
    </div>
  </div>
</template>
