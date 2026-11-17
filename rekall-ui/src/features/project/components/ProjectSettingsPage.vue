<script setup lang="ts">
import { ref } from 'vue'
import FormPanel from '@/components/ui/FormPanel.vue'
import ProjectChecksEditor from '@/features/check/components/ProjectChecksEditor.vue'
import ProjectGeneralPane from '@/features/project/components/ProjectGeneralPane.vue'
import ProjectRepositoryGroup from '@/features/project/components/ProjectRepositoryGroup.vue'
import ProjectSettingsRail from '@/features/project/components/ProjectSettingsRail.vue'
import ProjectTaskManagerPane from '@/features/project/components/ProjectTaskManagerPane.vue'
import type { Project } from '@/features/catalog/model/catalog'
import type { SettingsPane } from '@/features/project/model/project-settings'

/** Everything that can be changed on the project: a rail of settings beside the one open. */
defineProps<{ project: Project }>()

const emit = defineEmits<{ deleted: [] }>()

const pane = ref<SettingsPane>('general')
</script>

<template>
  <div
    class="grid h-full min-h-0 grid-cols-1 gap-4 md:grid-cols-[232px_minmax(0,1fr)]"
    aria-label="Project settings"
    data-testid="project-settings"
  >
    <ProjectSettingsRail :selected="pane" @select="pane = $event" />

    <section class="min-h-0 overflow-y-auto">
      <Transition name="fade-quick" mode="out-in">
        <div :key="pane">
          <ProjectGeneralPane v-if="pane === 'general'" :project="project" @deleted="emit('deleted')" />
          <div v-else class="form-grid">
            <FormPanel v-if="pane === 'repository'"><ProjectRepositoryGroup :project="project" /></FormPanel>
            <FormPanel v-else-if="pane === 'taskManager'"><ProjectTaskManagerPane :project="project" /></FormPanel>
            <FormPanel v-else class="col-span-full"><ProjectChecksEditor :project="project" /></FormPanel>
          </div>
        </div>
      </Transition>
    </section>
  </div>
</template>
