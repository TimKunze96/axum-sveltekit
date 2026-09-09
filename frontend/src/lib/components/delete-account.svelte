<script lang="ts">
  // The account deletion behind a confirmation and the password.
  import { firstErrors, follow, submit, type Success } from '$lib/client';
  import * as AlertDialog from '$lib/components/ui/alert-dialog';
  import { Button, buttonVariants } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import Heading from './heading.svelte';
  import InputError from './input-error.svelte';

  let {
    fetcher = globalThis.fetch,
    navigate = follow,
  }: {
    fetcher?: typeof globalThis.fetch;
    navigate?: (outcome: Success) => Promise<void>;
  } = $props();

  let open = $state(false);
  let password = $state('');
  let error = $state<string | undefined>();
  let busy = $state(false);

  async function confirm() {
    busy = true;
    try {
      const outcome = await submit('DELETE', '/api/settings/account', { password }, fetcher);
      if (outcome.ok) {
        open = false;
        await navigate(outcome);
      } else {
        error = firstErrors(outcome.errors).password ?? outcome.message;
      }
    } finally {
      busy = false;
    }
  }
</script>

<div class="space-y-6">
  <Heading
    variant="small"
    title="Delete account"
    description="Delete your account and all of its data"
  />
  <div
    class="space-y-4 rounded-lg border border-red-100 bg-red-50 p-4 dark:border-red-200/10 dark:bg-red-700/10"
  >
    <div class="relative space-y-0.5 text-red-600 dark:text-red-100">
      <p class="font-medium">Warning</p>
      <p class="text-sm">Please proceed with caution, this cannot be undone.</p>
    </div>
    <Button variant="destructive" onclick={() => (open = true)} data-testid="delete-account">
      Delete account
    </Button>
  </div>
</div>

<AlertDialog.Root bind:open>
  <AlertDialog.Content>
    <AlertDialog.Header>
      <AlertDialog.Title>Are you sure you want to delete your account?</AlertDialog.Title>
      <AlertDialog.Description>
        Once your account is deleted, all of its data will be permanently deleted. Enter your
        password to confirm.
      </AlertDialog.Description>
    </AlertDialog.Header>
    <div class="grid gap-2">
      <Label for="delete_password" class="sr-only">Password</Label>
      <Input
        id="delete_password"
        type="password"
        autocomplete="current-password"
        placeholder="Password"
        bind:value={password}
      />
      <InputError message={error} />
    </div>
    <AlertDialog.Footer>
      <AlertDialog.Cancel>Cancel</AlertDialog.Cancel>
      <Button variant="destructive" disabled={busy} onclick={confirm}>Delete account</Button>
    </AlertDialog.Footer>
  </AlertDialog.Content>
</AlertDialog.Root>
