<script lang="ts">
  import type { PageProps } from './$types';
  import DeleteAccount from '$lib/components/delete-account.svelte';
  import Heading from '$lib/components/heading.svelte';
  import ProfileForm from '$lib/components/profile-form.svelte';
  import SettingsLayout from '$lib/components/settings-layout.svelte';
  import { pageTitle } from '$lib/title';

  let { data }: PageProps = $props();
</script>

<svelte:head>
  <title>{pageTitle('Profile settings')}</title>
</svelte:head>

<h1 class="sr-only">Profile settings</h1>

<SettingsLayout>
  <div class="space-y-6">
    <Heading
      variant="small"
      title="Profile information"
      description="Update your name and email address"
    />
    {#key data.user.id}
      <ProfileForm user={data.user} />
    {/key}
  </div>
  <DeleteAccount />
</SettingsLayout>
