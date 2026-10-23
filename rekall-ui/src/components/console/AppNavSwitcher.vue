<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import AppNavGroup from '@/components/console/AppNavGroup.vue'
import AppNavMenu from '@/components/console/AppNavMenu.vue'
import NavAlertCount from '@/components/console/NavAlertCount.vue'
import { useNavAlerts } from '@/composables/useNavAlerts'
import { NAV_ENTRIES, alertLabel, isScreenActive } from '@/model/nav-tab'

const route = useRoute()
const alerts = useNavAlerts()
const routeName = computed(() => (typeof route.name === 'string' ? route.name : null))
</script>

<!-- Under 1280px the tabs fold into one key that opens them as a vertical menu, so the header's
     other cells keep their room in a narrow window. -->
<template>
  <nav class="flex shrink-0 self-stretch" aria-label="Screen">
    <div class="flex self-stretch max-[1279px]:hidden" data-testid="nav-tabs">
      <template
        v-for="entry in NAV_ENTRIES"
        :key="entry.kind === 'screen' ? entry.screen.to : entry.group.label"
      >
        <RouterLink
          v-if="entry.kind === 'screen'"
          :to="entry.screen.to"
          :replace="isScreenActive(entry.screen, routeName)"
          :title="entry.screen.hint"
          data-testid="nav-switcher-tab"
          class="nav-tab focus-ring"
          :class="isScreenActive(entry.screen, routeName) && 'nav-tab-current'"
          :aria-current="isScreenActive(entry.screen, routeName) ? 'page' : undefined"
        >
          {{ entry.screen.label }}
          <NavAlertCount
            v-if="alerts(entry.screen.alert) > 0"
            :count="alerts(entry.screen.alert)"
            :label="alertLabel(entry.screen.alert, alerts(entry.screen.alert))"
          />
        </RouterLink>
        <AppNavGroup v-else :group="entry.group" />
      </template>
    </div>
    <AppNavMenu class="hidden max-[1279px]:flex" />
  </nav>
</template>
