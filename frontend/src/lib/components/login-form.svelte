<script lang="ts">
  // The login form: posts to the API and shows its field errors inline.
  import { firstErrors, follow, submit, type Success } from '$lib/client';
  import { FORGOT_PASSWORD_PATH, REGISTER_PATH } from '$lib/login';
  import type { Credentials } from '$lib/types';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import InputError from './input-error.svelte';

  let {
    fetcher = globalThis.fetch,
    navigate = follow,
  }: {
    /** Injected by tests; the browser's fetch otherwise. */
    fetcher?: typeof globalThis.fetch;
    navigate?: (outcome: Success) => Promise<void>;
  } = $props();

  let form = $state<Credentials>({ email: '', password: '' });
  let errors = $state<Record<string, string>>({});
  let busy = $state(false);

  async function onsubmit(event: SubmitEvent) {
    event.preventDefault();
    busy = true;
    try {
      const outcome = await submit('POST', '/api/login', { ...form }, fetcher);
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
  <div class="grid gap-6">
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

    <div class="grid gap-2">
      <div class="flex items-center">
        <Label for="password">Password</Label>
        <a href={FORGOT_PASSWORD_PATH} class="ml-auto text-sm underline underline-offset-4">
          Forgot your password?
        </a>
      </div>
      <Input
        id="password"
        type="password"
        name="password"
        autocomplete="current-password"
        required
        bind:value={form.password}
        placeholder="Password"
      />
      <InputError message={errors.password} />
    </div>

    <Button type="submit" class="mt-2 w-full" disabled={busy}>Log in</Button>
  </div>

  <p class="text-center text-sm text-muted-foreground">
    No account yet?
    <a href={REGISTER_PATH} class="underline underline-offset-4">Create one</a>
  </p>
</form>
