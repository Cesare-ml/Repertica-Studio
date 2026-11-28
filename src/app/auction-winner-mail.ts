/**
 * Draft of the artist's own e-mail to the winner of an auction, prepared for the mail window (Gestionale Opere:
 * "Prepara email al vincitore", template "Vittoria asta"). The artist reads and edits it before anything is sent;
 * nothing is sent from here.
 */
export interface WinnerMailDraft {
  readonly to: string;
  readonly subject: string;
  readonly body: string;
}

const oneLine = (value: string | null | undefined): string => (value ?? '').replace(/[\u0000-\u001f\u007f]/g, ' ').replace(/\s+/g, ' ').trim();

export interface WinnerMailInput {
  readonly email: string;
  readonly name: string | null;
  readonly title: string;
  /** Winning bid in cents. */
  readonly amountCents: number;
}

/** The winner's address and the name they gave come from the server; the text is the Gestionale template. */
export function winnerMailDraft(input: WinnerMailInput): WinnerMailDraft {
  const email = input.email.trim();
  if (email.length < 3 || email.length > 254 || !/^[^\s@,;<>]+@[^\s@,;<>]+$/.test(email)) throw new Error('invalid_email');
  if (!Number.isSafeInteger(input.amountCents) || input.amountCents < 0) throw new Error('invalid_amount');
  const winner = oneLine(input.name) || 'collezionista';
  const title = oneLine(input.title);
  const amount = new Intl.NumberFormat('it-IT', { style: 'currency', currency: 'EUR' }).format(input.amountCents / 100);
  return Object.freeze({
    to: email,
    subject: `Aggiudicazione di «${title}»`,
    body: `Ciao ${winner},\n\ncongratulazioni! La tua offerta di ${amount} si è aggiudicata l’opera «${title}».\n\n`
      + 'Ti scrivo per concordare i prossimi passaggi.',
  });
}
