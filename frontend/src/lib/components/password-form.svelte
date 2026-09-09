<script lang="ts">
  // The password change, saved through the API.
  import { firstErrors, follow, submit, type Success } from '$lib/client';
  import type { PasswordInput } from '$lib/types';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import InputError from './input-error.svelte';

  let {
    fetcher = globalThis.fetch,
    navigate = follow,
  }: {
    fetcher?: typeof globalThis.fetch;
    navigate?: (outcome: Success) => Promise<void>;
  } = $props();

  const empty = (): Required<PasswordInput> => ({
    current_password: '',
    password: '',
    password_confirmation: '',
  });
  let form = $state(empty());
  let errors = $state<Record<string, string>>({});
  let busy = $state(false);

  async function onsubmit(event: SubmitEvent) {
    event.preventDefault();
    busy = true;
    try {
      const outcome = await submit('PUT', '/api/settings/password', { ...form }, fetcher);
      if (outcome.ok) {
        errors = {};
        form = empty();
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
    <Label for="current_password">Current password</Label>
    <Input
      id="current_password"
      type="password"
      name="current_password"
      autocomplete="current-password"
      required
      bind:value={form.current_password}
    />
    <InputError message={errors.current_password} />
  </div>

  <div class="grid gap-2">
    <Label for="password">New password</Label>
    <Input
      id="password"
      type="password"
      name="password"
      autocomplete="new-password"
      required
      bind:value={form.password}
    />
    <InputError message={errors.password} />
  </div>

  <div class="grid gap-2">
    <Label for="password_confirmation">Confirm new password</Label>
    <Input
      id="password_confirmation"
      type="password"
      name="password_confirmation"
      autocomplete="new-password"
      required
      bind:value={form.password_confirmation}
    />
    <InputError message={errors.password_confirmation} />
  </div>

  <Button type="submit" disabled={busy}>Save password</Button>
</form>
