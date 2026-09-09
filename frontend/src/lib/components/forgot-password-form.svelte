<script lang="ts">
  // Asks the API to mail a reset link; the answer is the same whether or
  // not the email is registered.
  import { firstErrors, follow, submit, type Success } from '$lib/client';
  import { LOGIN_PATH } from '$lib/login';
  import type { ForgotPasswordInput } from '$lib/types';
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

  let form = $state<ForgotPasswordInput>({ email: '' });
  let errors = $state<Record<string, string>>({});
  let busy = $state(false);

  async function onsubmit(event: SubmitEvent) {
    event.preventDefault();
    busy = true;
    try {
      const outcome = await submit('POST', '/api/forgot-password', { ...form }, fetcher);
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
    <Input
      id="email"
      type="email"
      name="email"
      autocomplete="email"
      required
      autofocus
      bind:value={form.email}
      placeholder="you@example.com"
    />
    <InputError message={errors.email} />
  </div>

  <Button type="submit" class="w-full" disabled={busy}>Email a reset link</Button>

  <p class="text-center text-sm text-muted-foreground">
    Or, return to
    <a href={LOGIN_PATH} class="underline underline-offset-4">log in</a>
  </p>
</form>
