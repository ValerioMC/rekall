<script setup lang="ts">
import { computed, nextTick, onMounted, ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import DatabaseFolderField from '@/components/setup/DatabaseFolderField.vue'
import RestartingOverlay from '@/components/setup/RestartingOverlay.vue'
import { useDatabaseSetup } from '@/composables/useDatabaseSetup'
import type { DatabaseEntry, DatabaseStatus } from '@/model/settings'

/**
 * The database that was active last time no longer answers where it was left — an external
 * drive that's unplugged, a cloud folder that moved or was renamed.
 *
 * Deliberately not the same screen as `FirstRunSetup`: falling back to a blank "let's get
 * started" wizard here would read as the previous database having been lost, when nothing has
 * actually been touched. This names what's missing and offers a way back to it, before offering
 * anything new.
 */
const props = defineProps<{ status: DatabaseStatus }>()

const { phase, error, switchTo } = useDatabaseSetup()
const addingNew = ref(false)
const switchingId = ref<string | null>(null)
const heading = ref<HTMLHeadingElement | null>(null)

const otherReachable = computed(() =>
  props.status.databases.filter((entry) => !entry.active && entry.reachable)
)

async function onSwitch(entry: DatabaseEntry): Promise<void> {
  switchingId.value = entry.id
  await switchTo(entry)
  switchingId.value = null
}

/**
 * This screen exists specifically to read as "not the same as a blank first run" — a
 * screen-reader user needs that announced, not just visible to a sighted one glancing at the
 * page.
 */
onMounted(async () => {
  await nextTick()
  heading.value?.focus()
})
</script>

<template>
  <div class="fade-in flex h-full items-center justify-center p-6" data-testid="database-unreachable">
    <div class="w-full max-w-[560px]">
      <div class="mb-6 flex items-center gap-3">
        <div
          class="grid size-10 shrink-0 place-items-center rounded-[var(--radius-control)] border border-danger bg-danger-soft text-[17px] font-bold text-danger"
          aria-hidden="true"
        >
          !
        </div>
        <div>
          <h1
            ref="heading"
            tabindex="-1"
            class="text-[17px] font-semibold tracking-[-0.015em] text-text outline-none"
          >
            Can't reach this database
          </h1>
          <p class="font-mono text-[10px] text-text-subtle">context, anchored</p>
        </div>
      </div>

      <AppCard>
        <RestartingOverlay v-if="phase === 'restarting'" />
        <template v-else>
          <p class="mb-1.5 text-[13px] leading-relaxed text-text-muted">
            <strong class="text-text">{{ props.status.active?.label }}</strong> was last configured at
          </p>
          <p
            class="mb-5 truncate rounded-[var(--radius-control)] border border-border bg-canvas px-3 py-2 font-mono text-[12.5px] text-text-subtle"
            :title="props.status.active?.path"
          >
            {{ props.status.active?.path }}
          </p>
          <p class="mb-5 text-[12.5px] leading-relaxed text-text-subtle">
            That folder isn't there right now. Nothing has been lost — reconnect it and reopen
            Rekall, pick another database below, or add a new one.
          </p>

          <p v-if="error" class="mb-4 text-[12.5px] text-danger" role="alert">{{ error }}</p>

          <template v-if="otherReachable.length && !addingNew">
            <p class="mb-2 text-[11px] font-semibold uppercase tracking-[0.09em] text-text-subtle">
              Other databases
            </p>
            <ul class="mb-5 space-y-2">
              <li
                v-for="entry in otherReachable"
                :key="entry.id"
                class="flex items-center justify-between gap-3 rounded-[var(--radius-control)] border border-border bg-canvas px-3.5 py-2.5"
              >
                <span class="min-w-0">
                  <span class="block truncate text-[13px] text-text">{{ entry.label }}</span>
                  <span class="block truncate font-mono text-[11px] text-text-subtle" :title="entry.path">
                    {{ entry.path }}
                  </span>
                </span>
                <AppButton
                  size="sm"
                  variant="secondary"
                  :loading="switchingId === entry.id"
                  :disabled="phase === 'submitting'"
                  @click="onSwitch(entry)"
                >
                  Switch
                </AppButton>
              </li>
            </ul>
          </template>

          <AppButton v-if="!addingNew" variant="ghost" size="sm" @click="addingNew = true">
            + Add a different database
          </AppButton>
          <DatabaseFolderField v-else autofocus cancellable submit-label="Add database" @cancel="addingNew = false" />
        </template>
      </AppCard>
    </div>
  </div>
</template>
