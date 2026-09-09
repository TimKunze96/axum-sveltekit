<script lang="ts">
  import ForgotPasswordForm from '$lib/components/forgot-password-form.svelte';
  import Heading from '$lib/components/heading.svelte';
  import { pageTitle } from '$lib/title';
</script>

<svelte:head>
  <title>{pageTitle('Forgot password')}</title>
</svelte:head>

<div class="space-y-2 text-center">
  <Heading title="Forgot password" description="Enter your email to receive a reset link" />
</div>
<ForgotPasswordForm />
