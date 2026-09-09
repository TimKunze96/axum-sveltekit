<script lang="ts">
  import type { PageProps } from './$types';
  import Heading from '$lib/components/heading.svelte';
  import ResetPasswordForm from '$lib/components/reset-password-form.svelte';
  import { pageTitle } from '$lib/title';

  let { data }: PageProps = $props();
</script>

<svelte:head>
  <title>{pageTitle('Reset password')}</title>
</svelte:head>

<div class="space-y-2 text-center">
  <Heading title="Reset password" description="Choose a new password below" />
</div>
<ResetPasswordForm token={data.token} email={data.email} />
