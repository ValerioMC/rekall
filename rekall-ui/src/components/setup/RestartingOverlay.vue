<script setup lang="ts">
/**
 * Shown from the moment a database-location change is submitted until the backend comes back
 * up.
 *
 * There is a real gap here — the old process has closed its port and the new one has not opened
 * it yet — and this is what fills it. It never has to be dismissed: {@link useDatabaseSetup}
 * reloads the page itself once the new process answers.
 */
</script>

<template>
  <div
    class="fade-in flex flex-col items-center gap-4 px-4 py-10 text-center"
    role="status"
    aria-live="polite"
    data-testid="restarting-overlay"
  >
    <span class="relative flex size-11 items-center justify-center" aria-hidden="true">
      <span class="absolute inset-0 animate-ping rounded-full bg-accent-soft" />
      <span class="relative size-3 rounded-full bg-accent" />
    </span>
    <p class="text-[14px] font-medium text-text">Reconfiguring&hellip;</p>
    <p class="max-w-[320px] text-[12.5px] leading-relaxed text-text-subtle">
      Rekall is restarting to connect to the new location. This takes a few seconds — the page
      reloads on its own once it's back.
    </p>
  </div>
</template>
