<script lang="ts">
  // The new password for the account the mailed link names; the token
  // and email come from the link's query string.
  import { firstErrors, follow, submit, type Success } from '$lib/client';
  import type { ResetPasswordInput } from '$lib/types';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import InputError from './input-error.svelte';

  let {
    token,
    email,
    fetcher = globalThis.fetch,
    navigate = follow,
  }: {
    token: string;
    email: string;
    fetcher?: typeof globalThis.fetch;
    navigate?: (outcome: Success) => Promise<void>;
  } = $props();

  let form = $state({ password: '', password_confirmation: '' });
  let errors = $state<Record<string, string>>({});
  let busy = $state(false);

  async function onsubmit(event: SubmitEvent) {
    event.preventDefault();
    busy = true;
    try {
      const body: Required<ResetPasswordInput> = { token, email, ...form };
      const outcome = await submit('POST', '/api/reset-password', body, fetcher);
      if (outcome.ok) {
        await navigate(outcome);
      } else {
        errors = firstErrors(outcome.errors);
        if (!Object.keys(errors).length) errors = { email: outcome.message };
      }
    } finally {
      busy = false;
    }
  }
</script>

<form {onsubmit} class="flex flex-col gap-6" novalidate>
  <div class="grid gap-2">
    <Label for="email">Email</Label>
    <Input id="email" type="email" name="email" value={email} readonly />
    <InputError message={errors.email} />
  </div>

  <div class="grid gap-2">
    <Label for="password">New password</Label>
    <Input
      id="password"
      type="password"
      name="password"
      autocomplete="new-password"
      required
      autofocus
      bind:value={form.password}
      placeholder="At least 8 characters"
    />
    <InputError message={errors.password} />
  </div>

  <div class="grid gap-2">
    <Label for="password_confirmation">Confirm password</Label>
    <Input
      id="password_confirmation"
      type="password"
      name="password_confirmation"
      autocomplete="new-password"
      required
      bind:value={form.password_confirmation}
      placeholder="Repeat the password"
    />
    <InputError message={errors.password_confirmation} />
  </div>

  <Button type="submit" class="w-full" disabled={busy}>Reset password</Button>
</form>
