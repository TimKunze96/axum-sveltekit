<script lang="ts">
  // The registration form: posts to the API and shows its field errors
  // inline.
  import { firstErrors, follow, submit, type Success } from '$lib/client';
  import { LOGIN_PATH } from '$lib/login';
  import type { Registration } from '$lib/types';
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

  let form = $state<Required<Registration>>({
    name: '',
    email: '',
    password: '',
    password_confirmation: '',
  });
  let errors = $state<Record<string, string>>({});
  let busy = $state(false);

  async function onsubmit(event: SubmitEvent) {
    event.preventDefault();
    busy = true;
    try {
      const outcome = await submit('POST', '/api/register', { ...form }, fetcher);
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
      <Label for="name">Name</Label>
      <Input
        id="name"
        type="text"
        name="name"
        autocomplete="name"
        required
        autofocus
        bind:value={form.name}
        placeholder="Full name"
      />
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
        placeholder="you@example.com"
      />
      <InputError message={errors.email} />
    </div>

    <div class="grid gap-2">
      <Label for="password">Password</Label>
      <Input
        id="password"
        type="password"
        name="password"
        autocomplete="new-password"
        required
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

    <Button type="submit" class="mt-2 w-full" disabled={busy}>Create account</Button>
  </div>

  <p class="text-center text-sm text-muted-foreground">
    Already registered?
    <a href={LOGIN_PATH} class="underline underline-offset-4">Log in</a>
  </p>
</form>
