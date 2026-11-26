/**
 * Asks the person to confirm and waits for the answer.
 *
 * In the Desktop app Tauri's dialog plugin replaces `window.confirm` with an asynchronous native dialog that
 * returns a Promise (it needs the `dialog:allow-message` permission). A plain `if (!confirm(...)) return;` never
 * stops, because a Promise is truthy, and the action runs before (or without) any answer. Awaiting the result
 * works with both that and a browser's synchronous confirm; a dialog that cannot be shown counts as "no".
 */
export async function confirmed(message: string): Promise<boolean> {
  try {
    const answer: unknown = await (window.confirm(message) as unknown);
    return answer === true;
  } catch {
    return false;
  }
}
