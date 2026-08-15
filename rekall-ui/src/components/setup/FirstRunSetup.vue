<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppLogo from '@/components/ui/AppLogo.vue'
import DatabaseFolderField from '@/components/setup/DatabaseFolderField.vue'

/**
 * Shown exactly once: the first time Rekall runs and no database has been configured yet, and
 * nothing was found to adopt from before this screen existed.
 *
 * After this, `~/.rekall/config.json` exists and the backend resolves straight to `READY` on
 * every future boot — this component never mounts again unless that file is removed.
 */
</script>

<template>
  <div class="fade-in flex h-full items-center justify-center p-6" data-testid="first-run-setup">
    <div class="w-full max-w-[520px]">
      <div class="mb-6 flex items-center gap-3">
        <AppLogo :size="40" class="halo rounded-[9px]" />
        <div>
          <p class="text-[17px] font-semibold tracking-[-0.015em] text-text">Welcome to Rekall</p>
          <p class="font-mono text-[10px] text-text-subtle">context, anchored</p>
        </div>
      </div>

      <AppCard>
        <h1 class="mb-1.5 text-[15px] font-semibold text-text">Where should Rekall keep its data?</h1>
        <p class="mb-5 text-[13px] leading-relaxed text-text-muted">
          Every company, project, task and note lives in one folder, as a single database file.
          Point Rekall at one now — an existing one from another machine, or a fresh, empty
          folder. You can add more, or switch between them, later from Settings.
        </p>
        <DatabaseFolderField autofocus submit-label="Get started" />
      </AppCard>
    </div>
  </div>
</template>
