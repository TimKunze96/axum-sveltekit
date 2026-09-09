<script lang="ts">
  import Heading from '$lib/components/heading.svelte';
  import LoginForm from '$lib/components/login-form.svelte';
  import { pageTitle } from '$lib/title';
</script>

<svelte:head>
  <title>{pageTitle('Log in')}</title>
</svelte:head>

<div class="space-y-2 text-center">
  <Heading title="Log in to your account" description="Enter your email and password below" />
</div>
<LoginForm />
