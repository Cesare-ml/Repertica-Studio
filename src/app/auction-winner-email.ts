/**
 * How the winner is reached when an auction closes (owner decision, 27 November 2026), and the template offered
 * for the artist's own winner e-mail. The server sends it as plain text, replaces the placeholders and sets the
 * artist's e-mail as the reply address (Repertica-auctions docs/ARTIST_API_V1.md, "Contatto con il vincitore").
 */
import type { AppLanguage } from './i18n';

export type WinnerMode = 'contact_privately' | 'custom_email';
export const WINNER_MODES: readonly WinnerMode[] = ['contact_privately', 'custom_email'];

/** Replaced by the server when it sends the e-mail. */
export const WINNER_PLACEHOLDERS: readonly { readonly token: string; readonly label: string }[] = [
  { token: '{name}', label: 'nome del vincitore' },
  { token: '{title}', label: 'titolo dell’asta' },
  { token: '{amount}', label: 'offerta vincente' },
  { token: '{link}', label: 'pagina dell’asta' },
  { token: '{artist}', label: 'il tuo nome pubblico' },
];

export const WINNER_SUBJECT_MAX = 160;
export const WINNER_BODY_MIN = 20;
export const WINNER_BODY_MAX = 2500;

/** Bank details the artist saved for earlier drafts (bank transfer terms before 27 November 2026). */
export interface SavedBankDetails {
  readonly beneficiary: string;
  readonly iban: string;
  readonly bicSwift: string;
  readonly bankNameAndAddress: string;
}

const TEMPLATES: Record<AppLanguage, { subject: string; lines: (bank: SavedBankDetails | null) => string[] }> = {
  it: {
    subject: 'Hai vinto l’asta: {title}',
    lines: bank => ['Ciao {name},', '', 'complimenti: la tua offerta di {amount} si è aggiudicata «{title}».', '',
      'Per completare l’acquisto puoi pagare con bonifico bancario:',
      `Beneficiario: ${bank?.beneficiary || '…'}`, `IBAN: ${bank?.iban || '…'}`,
      ...(bank?.bicSwift ? [`BIC/SWIFT: ${bank.bicSwift}`] : []), ...(bank?.bankNameAndAddress ? [`Banca: ${bank.bankNameAndAddress}`] : []),
      'Causale: {title}', '', 'Spedizione: …', '',
      'Rispondi a questa email per qualsiasi domanda o per concordare la consegna.', '', 'Un saluto,', '{artist}'],
  },
  en: {
    subject: 'You won the auction: {title}',
    lines: bank => ['Hello {name},', '', 'congratulations: your bid of {amount} won “{title}”.', '',
      'To complete the purchase you can pay by bank transfer:',
      `Beneficiary: ${bank?.beneficiary || '…'}`, `IBAN: ${bank?.iban || '…'}`,
      ...(bank?.bicSwift ? [`BIC/SWIFT: ${bank.bicSwift}`] : []), ...(bank?.bankNameAndAddress ? [`Bank: ${bank.bankNameAndAddress}`] : []),
      'Reference: {title}', '', 'Shipping: …', '',
      'Reply to this e-mail with any question or to arrange delivery.', '', 'Best regards,', '{artist}'],
  },
  es: {
    subject: 'Has ganado la subasta: {title}',
    lines: bank => ['Hola {name}:', '', 'enhorabuena: tu oferta de {amount} ha ganado «{title}».', '',
      'Para completar la compra puedes pagar por transferencia bancaria:',
      `Beneficiario: ${bank?.beneficiary || '…'}`, `IBAN: ${bank?.iban || '…'}`,
      ...(bank?.bicSwift ? [`BIC/SWIFT: ${bank.bicSwift}`] : []), ...(bank?.bankNameAndAddress ? [`Banco: ${bank.bankNameAndAddress}`] : []),
      'Concepto: {title}', '', 'Envío: …', '',
      'Responde a este correo para cualquier pregunta o para acordar la entrega.', '', 'Un saludo,', '{artist}'],
  },
  fr: {
    subject: 'Vous avez remporté l’enchère : {title}',
    lines: bank => ['Bonjour {name},', '', 'félicitations : votre offre de {amount} a remporté « {title} ».', '',
      'Pour finaliser l’achat, vous pouvez payer par virement bancaire :',
      `Bénéficiaire : ${bank?.beneficiary || '…'}`, `IBAN : ${bank?.iban || '…'}`,
      ...(bank?.bicSwift ? [`BIC/SWIFT : ${bank.bicSwift}`] : []), ...(bank?.bankNameAndAddress ? [`Banque : ${bank.bankNameAndAddress}`] : []),
      'Référence : {title}', '', 'Livraison : …', '',
      'Répondez à cet e-mail pour toute question ou pour convenir de la livraison.', '', 'Cordialement,', '{artist}'],
  },
  de: {
    subject: 'Sie haben die Auktion gewonnen: {title}',
    lines: bank => ['Hallo {name},', '', 'herzlichen Glückwunsch: Ihr Gebot von {amount} hat „{title}“ gewonnen.', '',
      'Zum Abschluss des Kaufs können Sie per Banküberweisung bezahlen:',
      `Empfänger: ${bank?.beneficiary || '…'}`, `IBAN: ${bank?.iban || '…'}`,
      ...(bank?.bicSwift ? [`BIC/SWIFT: ${bank.bicSwift}`] : []), ...(bank?.bankNameAndAddress ? [`Bank: ${bank.bankNameAndAddress}`] : []),
      'Verwendungszweck: {title}', '', 'Versand: …', '',
      'Antworten Sie auf diese E-Mail bei Fragen oder um die Lieferung zu vereinbaren.', '', 'Viele Grüße', '{artist}'],
  },
};

/** The template offered for the winner e-mail, in the interface language; saved bank details fill it in. */
export function defaultWinnerEmail(language: AppLanguage, bank: SavedBankDetails | null = null): { subject: string; body: string } {
  const template = TEMPLATES[language] ?? TEMPLATES.en;
  const usable = bank && (bank.beneficiary || bank.iban) ? bank : null;
  return { subject: template.subject, body: template.lines(usable).join('\n') };
}

/** Same substitution as the server, for the preview in the review step. */
export function fillWinnerTemplate(template: string, values: Partial<Record<'name' | 'title' | 'amount' | 'link' | 'artist', string>>): string {
  return template.replace(/\{(name|title|amount|link|artist)\}/g, (match, key: 'name' | 'title' | 'amount' | 'link' | 'artist') => values[key] ?? match);
}
