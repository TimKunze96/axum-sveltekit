<script lang="ts">
  import Heading from '$lib/components/heading.svelte';
  import PasswordForm from '$lib/components/password-form.svelte';
  import SettingsLayout from '$lib/components/settings-layout.svelte';
  import { pageTitle } from '$lib/title';
</script>

<svelte:head>
  <title>{pageTitle('Password settings')}</title>
</svelte:head>

<h1 class="sr-only">Password settings</h1>

<SettingsLayout>
  <div class="space-y-6">
    <Heading
      variant="small"
      title="Update password"
      description="Use a long, random password to stay secure"
    />
    <PasswordForm />
  </div>
</SettingsLayout>
