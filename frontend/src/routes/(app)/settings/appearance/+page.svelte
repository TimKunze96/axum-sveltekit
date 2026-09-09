<script lang="ts">
  import AppearanceTabs from '$lib/components/appearance-tabs.svelte';
  import Heading from '$lib/components/heading.svelte';
  import SettingsLayout from '$lib/components/settings-layout.svelte';
  import { pageTitle } from '$lib/title';
</script>

<svelte:head>
  <title>{pageTitle('Appearance settings')}</title>
</svelte:head>

<h1 class="sr-only">Appearance settings</h1>

<SettingsLayout>
  <div class="space-y-6">
    <Heading
      variant="small"
      title="Appearance settings"
      description="Update your account's appearance settings"
    />
    <AppearanceTabs />
  </div>
</SettingsLayout>
