<script setup lang="ts">
import AppButton from '@/components/ui/AppButton.vue'
</script>

<template>
  <section class="flex flex-col gap-3" aria-label="Connections" data-testid="connections-section">
    <h2 class="text-heading font-medium text-text">Connections</h2>
    <p class="max-w-[60ch] text-body text-text-muted">
      GitLab, GitHub, Kubernetes, Jira and Grafana accounts are shared with companies and bound to projects. They have a screen of their own, with their credentials and who uses them.
    </p>
    <div>
      <RouterLink to="/connections"><AppButton variant="secondary" size="sm">Open Connections</AppButton></RouterLink>
    </div>
  </section>
</template>
