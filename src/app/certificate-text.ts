import type { Artwork } from './models';

/** Language of a printed certificate: Italian, English or both on the same sheet. */
export type CertificateLanguage = 'it' | 'en' | 'bilingual';

export interface CertificateLabels {
  artist: string; technique: string; dimensions: string; signature: string;
  verification: string; support: string; missingImage: string; recipient: string;
}
export interface CertificateTexts {
  titleLines: string[];
  labels: CertificateLabels;
  declarations: string[];
  /** Locale for the date of issue (bilingual certificates use the Italian one). */
  locale: string;
}

const LABELS_IT: CertificateLabels = {
  artist: 'Artista', technique: 'Tecnica', dimensions: 'Dimensioni', signature: 'Firma', verification: 'Verifica',
  support: ' su ', missingImage: 'Immagine opera non disponibile', recipient: 'Intestatario',
};
const LABELS_EN: CertificateLabels = {
  artist: 'Artist', technique: 'Technique', dimensions: 'Dimensions', signature: 'Signature', verification: 'Verification',
  support: ' on ', missingImage: 'Artwork image unavailable', recipient: 'Owner',
};
const LABELS_BILINGUAL: CertificateLabels = {
  artist: 'Artista / Artist', technique: 'Tecnica / Technique', dimensions: 'Dimensioni / Dimensions', signature: 'Firma / Signature',
  verification: 'Verifica / Verification', support: ' su / on ', missingImage: 'Immagine opera non disponibile',
  recipient: 'Intestatario / Owner',
};

export function declarationIt(artist: string): string {
  return `Il sottoscritto ${artist} dichiara che l’opera qui riprodotta e descritta è un’opera originale, autentica e realizzata personalmente dall’artista. L’immagine, i dati tecnici, il codice d’archivio e il numero del presente certificato costituiscono elementi identificativi dell’opera.`;
}
export function declarationEn(artist: string): string {
  return `I, ${artist}, certify that the artwork reproduced and described herein is an original and authentic work personally created by the artist. The image, technical information, archive code and certificate number constitute identifying elements of the artwork.`;
}

export function normalizeLanguage(value: string | null | undefined): CertificateLanguage {
  return value === 'en' || value === 'bilingual' ? value : 'it';
}

export function certificateTexts(language: string | null | undefined, artist: string): CertificateTexts {
  const lang = normalizeLanguage(language);
  if (lang === 'en') {
    return { titleLines: ['Certificate of Authenticity'], labels: LABELS_EN, declarations: [declarationEn(artist)], locale: 'en-GB' };
  }
  if (lang === 'it') {
    return { titleLines: ['Certificato di Autenticità'], labels: LABELS_IT, declarations: [declarationIt(artist)], locale: 'it-IT' };
  }
  return {
    titleLines: ['Certificato di Autenticità', 'Certificate of Authenticity'], labels: LABELS_BILINGUAL,
    declarations: [declarationIt(artist), declarationEn(artist)], locale: 'it-IT',
  };
}

/** "12 novembre 2026" / "12 November 2026"; never normalize a corrupt historical date. */
export function formatIssueDate(value: string | null | undefined, language: string | null | undefined): string {
  if (!value) return '';
  const original = String(value);
  const raw = original.slice(0, 10);
  const parts = /^(\\d{4})-(\\d{2})-(\\d{2})(?:$|T)/.exec(original);
  if (!parts) return raw;
  const [year, month, day] = parts.slice(1).map(Number);
  if (year === 0) return raw;
  // Use UTC and an explicit calendar round-trip: JS Date silently rolls 31 April
  // into May, which would print a different issue date on an immutable certificate.
  const date = new Date(0);
  date.setUTCHours(12, 0, 0, 0);
  date.setUTCFullYear(year, month - 1, day);
  if (date.getUTCFullYear() !== year || date.getUTCMonth() !== month - 1 || date.getUTCDate() !== day) return raw;
  return new Intl.DateTimeFormat(certificateTexts(language, '').locale, {
    day: '2-digit', month: 'long', year: 'numeric', timeZone: 'UTC',
  }).format(date);
}

/** Technique/support as written on the work, in the certificate's language (bilingual: "Olio / Oil"). */
export function localizedValue(value: string | null | undefined, language: string | null | undefined, toEnglish: (text: string) => string): string {
  const text = (value ?? '').trim();
  if (!text) return '';
  const lang = normalizeLanguage(language);
  if (lang === 'it') return text;
  const english = toEnglish(text);
  return lang === 'en' ? english : english === text ? text : `${text} / ${english}`;
}

export function dimensionsText(artwork: Pick<Artwork, 'height_cm' | 'width_cm' | 'depth_cm'>): string {
  const parts = [artwork.height_cm, artwork.width_cm, artwork.depth_cm].filter(v => v !== null && v !== undefined);
  return parts.length ? `${parts.join(' × ')} cm` : '';
}

/** SHA-256 in hexadecimal of the text (the frozen snapshot and the serial identify an issued certificate). */
export async function sha256Hex(text: string): Promise<string> {
  const bytes = new TextEncoder().encode(text);
  const digest = await crypto.subtle.digest('SHA-256', bytes);
  return Array.from(new Uint8Array(digest)).map(b => b.toString(16).padStart(2, '0')).join('');
}
