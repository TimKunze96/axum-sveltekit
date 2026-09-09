<script lang="ts">
  import Heading from '$lib/components/heading.svelte';
  import RegisterForm from '$lib/components/register-form.svelte';
  import { pageTitle } from '$lib/title';
</script>

<svelte:head>
  <title>{pageTitle('Create an account')}</title>
</svelte:head>

<div class="space-y-2 text-center">
  <Heading title="Create an account" description="Enter your details below to get started" />
</div>
<RegisterForm />
