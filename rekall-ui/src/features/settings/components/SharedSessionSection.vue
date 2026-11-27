<script setup lang="ts">
import { computed, ref } from 'vue'
import { storeToRefs } from 'pinia'
import FormPanel from '@/components/ui/FormPanel.vue'
import FormRow from '@/components/ui/FormRow.vue'
import SegmentedControl, { type SegmentOption } from '@/components/ui/SegmentedControl.vue'
import { preferredSessionKind, setPreferredSessionKind } from '@/common/config/claude-launch'
import { preferredProvider, setPreferredProvider } from '@/common/config/session-launch'
import { SESSION_VIEWS, sessionViewChoiceLabel, type SessionView } from '@/features/claude/model/claude'
import { SESSION_PROVIDERS, SESSION_PROVIDER_LABEL, type SessionProvider } from '@/features/claude/model/session-provider'
import { useClaudeConnectionStore } from '@/features/claude/store/claudeConnection.store'

const PROVIDER_OPTIONS: readonly SegmentOption<SessionProvider>[] = SESSION_PROVIDERS.map((value) => ({
  value,
  label: SESSION_PROVIDER_LABEL[value]
}))
const VIEW_OPTIONS: readonly SegmentOption<SessionView>[] = SESSION_VIEWS.map((value) => ({
  value,
  label: sessionViewChoiceLabel(value)
}))

const { installation } = storeToRefs(useClaudeConnectionStore())

const provider = computed(preferredProvider)

const sessionView = ref(preferredSessionKind())

function chooseSessionView(choice: SessionView): void {
  sessionView.value = choice
  setPreferredSessionKind(choice)
}
</script>

<template>
  <FormPanel
    title="Both providers"
    description="Claude Code and Codex share these."
    aria-label="Both providers"
    data-testid="shared-session-settings"
  >
    <FormRow
      label="Session provider"
      hint="Starts new interactive sessions. You can also change it in the top bar."
    >
      <SegmentedControl
        :model-value="provider"
        :options="PROVIDER_OPTIONS"
        label="Session provider"
        data-testid="session-provider"
        @update:model-value="setPreferredProvider"
      />
    </FormRow>
    <FormRow
      label="Session view"
      hint="Chat shows the conversation and approval requests full-size, under the open task tabs. The run queue has its own view."
    >
      <SegmentedControl
        :model-value="sessionView"
        :options="VIEW_OPTIONS"
        label="Session view"
        data-testid="session-view"
        @update:model-value="chooseSessionView"
      />
    </FormRow>
    <FormRow label="MCP server" hint="Where Rekall serves its tools. Connecting either provider registers this address.">
      <code class="min-w-0 break-all font-mono text-small text-text" data-testid="mcp-endpoint">
        {{ installation?.endpoint ?? 'Not read yet' }}
      </code>
    </FormRow>
  </FormPanel>
</template>
