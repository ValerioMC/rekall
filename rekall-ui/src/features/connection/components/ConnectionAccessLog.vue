<script setup lang="ts">
import { onMounted, ref, watch } from 'vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { relativeTime } from '@/common/format/relative-time'
import { useConnectionsStore } from '@/features/connection/store/connections.store'
import type { ConnectionId } from '@/model/branded'

/** Every request a terminal made for this connection's credential, newest first. */
const props = defineProps<{ connectionId: ConnectionId }>()

const store = useConnectionsStore()
const { run, isRunning } = useAsyncAction()
const loaded = ref(false)

async function load(): Promise<void> {
  loaded.value = false
  await run(() => store.loadAccess(props.connectionId))
  loaded.value = true
}

onMounted(load)
watch(() => props.connectionId, load)

defineExpose({ reload: load })
</script>

<template>
  <div data-testid="access-log">
    <p v-if="loaded && !store.accessOf(connectionId).length" class="text-caption text-text-subtle">
      No terminal has asked for this credential yet.
    </p>
    <ol v-else class="flex flex-col divide-y divide-border">
      <li
        v-for="access in store.accessOf(connectionId)"
        :key="access.id"
        class="flex items-start gap-3 py-2"
        data-testid="access-row"
      >
        <AppBadge :tone="access.granted ? 'safe' : 'danger'" dot>{{
          access.granted ? 'Handed out' : 'Refused'
        }}</AppBadge>
        <span class="min-w-0 flex-1">
          <span class="block truncate text-small text-text">
            <template v-if="access.taskLabel">task:{{ access.taskLabel }}</template>
            <template v-else>a project terminal</template>
          </span>
          <span v-if="access.reason" class="block text-caption text-danger">{{ access.reason }}</span>
          <span class="block font-mono text-caption text-text-subtle"
            >terminal {{ access.terminalId.slice(0, 8) }}</span
          >
        </span>
        <span class="shrink-0 text-caption text-text-subtle">{{ relativeTime(access.accessedAt) }}</span>
      </li>
    </ol>
    <AppButton
      v-if="store.hasMoreAccess(connectionId)"
      size="sm"
      variant="ghost"
      class="mt-2"
      :loading="isRunning"
      @click="run(() => store.loadMoreAccess(connectionId))"
    >
      Older requests
    </AppButton>
  </div>
</template>
