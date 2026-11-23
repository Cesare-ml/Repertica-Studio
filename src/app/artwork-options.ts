/** Options of the technique and support menus (the same ones used by the Gestionale Opere). */
export const OTHER_CHOICE = 'Altro';

export const TECHNIQUES: readonly string[] = [
  'Olio', 'Acrilico', 'Tecnica mista', 'Tecnica mista e olio', 'Acquerello', 'Acquerello e matita',
  'Acquerello e inchiostri', 'Grafite', 'Carboncino', 'Pastello', 'Pennarello',
];

export const SUPPORTS: readonly string[] = [
  'Tela', 'Tela di lino', 'Carta', 'Carta Fabriano', 'Carta da acquerello', 'Tavola', 'Legno', 'Cartone telato', 'Pannello',
];

export interface MenuState {
  /** Value shown by the menu: '' (nothing chosen), one of the options, or `OTHER_CHOICE`. */
  choice: string;
  /** Free text, only meaningful when the choice is `OTHER_CHOICE`. */
  other: string;
}

/** Compare imported menu values without losing the original free-text entry.
 * Historical archives can use non-breaking spaces, compatibility-width Latin letters
 * or repeated whitespace, while the dropdown uses canonical option labels.
 */
function optionKey(value: string): string {
  return value.normalize('NFKC').trim().replace(/\s+/gu, ' ').toLowerCase();
}

/** Work out what the menu should show for a stored value; values that are not in the list become "Altro". */
export function menuStateFor(options: readonly string[], value: string | null | undefined): MenuState {
  const text = (value ?? '').trim();
  if (!text) return { choice: '', other: '' };
  const known = options.find(option => optionKey(option) === optionKey(text));
  return known ? { choice: known, other: '' } : { choice: OTHER_CHOICE, other: text };
}

/** The value to store for the current menu state (null when nothing is chosen or "Altro" is still empty). */
export function valueFor(choice: string, other: string): string | null {
  if (choice === OTHER_CHOICE) return other.trim() || null;
  return choice.trim() || null;
}
