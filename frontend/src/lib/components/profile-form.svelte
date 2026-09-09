<script lang="ts">
  // The name and email, saved through the API.
  import { untrack } from 'svelte';
  import { firstErrors, follow, submit, type Success } from '$lib/client';
  import type { ProfileInput, User } from '$lib/types';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import InputError from './input-error.svelte';

  let {
    user,
    fetcher = globalThis.fetch,
    navigate = follow,
  }: {
    user: User;
    fetcher?: typeof globalThis.fetch;
    navigate?: (outcome: Success) => Promise<void>;
  } = $props();

  // The form starts from the account as loaded; the page re-keys it when
  // the account changes.
  let form = $state<ProfileInput>(untrack(() => ({ name: user.name, email: user.email })));
  let errors = $state<Record<string, string>>({});
  let busy = $state(false);

  async function onsubmit(event: SubmitEvent) {
    event.preventDefault();
    busy = true;
    try {
      const outcome = await submit('PUT', '/api/settings/profile', { ...form }, fetcher);
      if (outcome.ok) {
        errors = {};
        await navigate(outcome);
      } else {
        errors = firstErrors(outcome.errors);
      }
    } finally {
      busy = false;
    }
  }
</script>

<form {onsubmit} class="space-y-6" novalidate>
  <div class="grid gap-2">
    <Label for="name">Name</Label>
    <Input id="name" name="name" autocomplete="name" required bind:value={form.name} />
    <InputError message={errors.name} />
  </div>

  <div class="grid gap-2">
    <Label for="email">Email</Label>
    <Input
      id="email"
      type="email"
      name="email"
      autocomplete="email"
      required
      bind:value={form.email}
    />
    <InputError message={errors.email} />
  </div>

  <Button type="submit" disabled={busy}>Save</Button>
</form>
