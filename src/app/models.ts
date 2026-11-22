/** Personal email is separate from newsletter campaigns. Secrets are never returned by IPC. */
export interface MailAccount {
  id: string; email: string; display_name: string;
  imap_host: string; imap_port: number; imap_username: string;
  smtp_host: string; smtp_port: number; smtp_username: string;
  sent_folder: string; signature: string;
  created_at: string; updated_at: string;
}
export type MailAccountInput = Omit<MailAccount,'created_at'|'updated_at'|'id'> & {
  id?: string | null; imap_password?: string | null; smtp_password?: string | null;
};
export interface MailFolder { name: string; role: 'inbox'|'sent'|'drafts'|'trash'|'junk'|'other'; }
export interface MailHeader {
  uid: number; uid_validity: number; folder: string;
  subject: string; from: string; date: string; seen: boolean;
  flagged: boolean; size: number;
}
export interface MailList {
  folder: string; uid_validity: number; exists: number; items: MailHeader[];
}
export interface MailDetail {
  header: MailHeader; body: string; html_only: boolean; attachment_count: number;
}
export interface MailDraft {
  id: string; account_id: string; to: string; cc: string; bcc: string;
  subject: string; body: string; revision: number; updated_at: string;
  attachments: string[];
}
export type MailDraftInput = Omit<MailDraft,'updated_at'|'revision'|'id'> & {
  id?: string | null; revision?: number | null;
};
export interface MailSendInput {
  account_id: string; to: string; cc: string; bcc: string;
  subject: string; body: string; confirm: boolean; attachments: string[];
}
export interface MailSendResult {
  delivery: 'accepted'; sent_copy: 'saved'|'not_saved'|'not_configured';
}

export type ArtworkStatus = 'available' | 'reserved' | 'sold' | 'archived';

// Publication Engine documents are independent from the exported PDF snapshots.
export interface BrochureItem {
  artwork_id: string;
  template: 'classic' | 'full_bleed' | 'details';
  gallery_image_id: string | null;
  title_override: string;
  description_override: string;
  include_description: boolean;
}
export interface Brochure {
  id: string; title: string; subtitle: string; artist: string;
  introduction: string; contact_email: string; contact_website: string;
  contact_instagram: string; language: 'it' | 'en' | 'es' | 'fr' | 'de';
  items: BrochureItem[]; created_at: string; updated_at: string;
}


export interface Artwork {
  id: string;
  code: string;
  title: string;
  year: string | null;
  technique: string | null;
  support: string | null;
  width_cm: number | null;
  height_cm: number | null;
  depth_cm: number | null;
  description: string | null;
  location: string | null;
  status: ArtworkStatus;
  image_path: string | null;
  created_at: string;
  updated_at: string;
  /** The artist's signature is on the work. */
  signed: boolean;
  final_varnish: boolean;
  framed: boolean;
}
export type ArtworkInput = Pick<Artwork,
  'title' | 'year' | 'technique' | 'support' | 'width_cm' |
  'height_cm' | 'depth_cm' | 'description' | 'location' | 'status' | 'signed' | 'final_varnish' | 'framed'
> & { id?: string; code?: string | null };
/** For a new artwork the archive code is assigned by the application. */
export type NewArtworkRecord = ArtworkInput;
export interface NewSaleInput {
  collector_id: string | null; new_collector_name: string | null;
  amount_cents: number; sold_at: string; notes: string | null;
  tax_cents: number; shipping_cents: number; payment?: PaymentInput | null;
}
export type EditOperation =
  | { type: 'crop'; x: number; y: number; width: number; height: number }
  | { type: 'perspective'; points: { x: number; y: number }[]; outputAspectRatio: number | null }
  | { type: 'rotate'; degrees: number }
  | { type: 'flip'; horizontal: boolean; vertical: boolean }
  | { type: 'adjust'; exposure: number; contrast: number; saturation: number };
/** A picture of a new artwork: a file (optionally edited first) or an item of the library. */
export interface PhotoSource { path?: string | null; media_id?: string | null; operations: EditOperation[]; kind?: PhotoKind; }
export interface NewArtworkInput {
  artwork: NewArtworkRecord; cover: PhotoSource | null; gallery: PhotoSource[]; raw_path: string | null; sale: NewSaleInput | null;
}
export interface RawFile {
  artwork_id: string; media_path: string; original_name: string; size_bytes: number; sha256: string; created_at: string;
}
export interface MediaTag { id: string; name: string; }
export interface MediaRevision {
  id: string; number: number; kind: 'ORIGINAL' | 'EDIT'; image_path: string; width: number; height: number;
  size_bytes: number; sha256: string; operations_json: string | null; created_at: string;
}
export type CopyrightStatus = 'UNKNOWN' | 'OWNED' | 'LICENSED' | 'PUBLIC_DOMAIN';
export interface MediaItem {
  id: string; title: string | null; description: string | null; alt_text: string | null; credits: string | null;
  copyright_status: CopyrightStatus; original_name: string; state: 'ACTIVE' | 'TRASHED'; current_revision_id: string;
  image_path: string; width: number; height: number; size_bytes: number; revision_number: number;
  tags: MediaTag[]; usage_count: number; created_at: string; updated_at: string;
}
export interface MediaUsage { kind: 'ARTWORK_COVER' | 'ARTWORK_GALLERY' | 'COLLECTOR_PROFILE' | 'COLLECTOR_PHOTO'; owner_id: string; label: string; }
export interface MediaDetail { item: MediaItem; revisions: MediaRevision[]; usages: MediaUsage[]; collection_ids: string[]; }
export interface MediaPage { items: MediaItem[]; total: number; }
export interface MediaCollection { id: string; name: string; item_count: number; }
export interface MediaMetadataInput {
  title: string | null; description: string | null; alt_text: string | null; credits: string | null; copyright_status: CopyrightStatus;
}
export interface PreviewData { data_url: string; width: number; height: number; }
export interface ImportResult { imported: MediaItem[]; failed: { path: string; message: string }[]; }
export interface DuplicateGroup { sha256: string; size_bytes: number; items: MediaItem[]; }
export interface SimilarPair { first: MediaItem; second: MediaItem; distance: number; similarity: number; }
export interface MediaExportOptions { format?: 'jpeg' | 'png'; quality?: number; max_width?: number | null; max_height?: number | null; }
export interface CodePolicy {
  prefix: string; scope: 'GLOBAL' | 'YEARLY'; padding: number; separator: '-' | '.' | '_';
}

/** What an additional photograph shows (stable codes stored in the archive). */
export type PhotoKind = 'DETAIL' | 'SIGNATURE' | 'DIGITAL_MOCKUP' | 'IN_SITU' | 'OTHER' | 'FRAMED' | 'WORK_IN_PROGRESS';
/** Labels are Italian interface strings (translated with `tr`). */
export const PHOTO_KINDS: { code: PhotoKind; label: string }[] = [
  { code: 'DETAIL', label: 'Dettaglio' },
  { code: 'SIGNATURE', label: 'Firma' },
  { code: 'DIGITAL_MOCKUP', label: 'Mockup digitale' },
  { code: 'IN_SITU', label: 'Opera ambientata' },
  { code: 'FRAMED', label: 'Opera incorniciata' },
  { code: 'WORK_IN_PROGRESS', label: 'Work in progress' },
  { code: 'OTHER', label: 'Altro' },
];
export interface GalleryImage {
  id: string; artwork_id: string; image_path: string;
  caption: string | null; created_at: string; sort_order: number; kind: PhotoKind;
}

export type MediaProblem = 'unsafe_path' | 'missing' | 'unreadable' | 'not_regular' | 'invalid_size' | 'invalid_header' | 'invalid_snapshot';
export interface MediaIssue {
  artworkId: string; imageId: string | null; role: 'primary' | 'gallery' | 'certificate' | 'collector' | 'media';
  path: string; problem: MediaProblem;
}
export interface MediaAudit {
  checked: number; healthy: number; problems: number; truncated: boolean; issues: MediaIssue[];
}

export interface Collector {
  id: string; name: string; first_name: string; last_name: string | null;
  email: string | null; phone: string | null; address: string | null; notes: string | null;
  birth_date: string | null; birth_time: string | null; birth_place: string | null;
  instagram: string | null; facebook: string | null; tiktok: string | null;
  website: string | null; linkedin: string | null;
  profile_image_id: string | null; photo_count: number;
  created_at: string; updated_at: string;
}
export type CollectorInput = Pick<Collector, 'first_name' | 'last_name' | 'email' | 'phone' | 'address' | 'notes' |
  'birth_date' | 'birth_time' | 'birth_place' | 'instagram' | 'facebook' | 'tiktok' | 'website' | 'linkedin'> & { id?: string };
export interface CollectorImage {
  id: string; collector_id: string; kind: 'PROFILE' | 'PHOTO'; image_path: string; created_at: string;
}
export interface NewCollectorInput { collector: CollectorInput; profile_path: string | null; photo_paths: string[]; }

export interface Sale {
  id: string; artwork_id: string; collector_id: string; amount_cents: number;
  currency: 'EUR'; sold_at: string; notes: string | null;
  artwork_title: string; collector_name: string; created_at: string;
}
export interface InstallmentLineInput { due_date: string; amount_cents: number; notes: string | null; }
export interface PaymentInput {
  mode: 'UNPAID' | 'PAID' | 'PARTIAL' | 'INSTALLMENTS';
  received_cents: number | null; received_at: string | null;
  method: 'bank_transfer' | 'cash' | 'card' | 'other'; reference: string | null;
  installments: InstallmentLineInput[];
}
export interface SaleInput {
  artwork_id: string; collector_id: string; amount_cents: number; sold_at: string; notes: string | null; tax_cents: number; shipping_cents: number;
  payment?: PaymentInput | null;
}
export type InstallmentStatus = 'PAID' | 'PARTIAL' | 'OVERDUE' | 'DUE';
export interface Installment {
  id: string; sale_id: string; position: number; due_date: string; amount_cents: number;
  notes: string | null; paid_cents: number; status: InstallmentStatus;
}
export interface PlanLine { due_date: string; amount_cents: number; }
export interface Certificate {
  id: string; artwork_id: string; serial: string; issued_at: string;
  artist_name: string; collector_name: string | null; notes: string | null;
  snapshot_json: string; artwork_title: string; created_at: string;
  language: 'it' | 'en' | 'bilingual'; issued_place: string | null;
}
export interface CertificateInput {
  artwork_id: string;
  /** Left empty: the artist name saved in the settings is used. */
  artist_name?: string | null;
  collector_name: string | null; notes: string | null;
  language: 'it' | 'en' | 'bilingual'; place: string | null;
}
export interface Overview {
  artworks: number; collectors: number; sales: number; certificates: number; revenue_cents: number;
}

export interface BackupSummary {
  directory: string; createdAt: string; files: number; totalBytes: number;
  artworks: number; collectors: number; sales: number; certificates: number;
  schemaVersion: number; verified: boolean;
}
export interface RestoreSummary { restored: BackupSummary; previousArchive: string }

export interface StudioSettings {
  artist_coefficient: number;
  base_price_cm2_cents: number;
  personal_bp: number;
  activity_bp: number;
  savings_bp: number;
  treasury_start_date: string | null;
  default_tax_bp: number;
  ui_language: 'it' | 'en' | 'es' | 'fr' | 'de' | null;
  /** Name printed on certificates as the issuing artist. */
  artist_name: string | null;
}
export interface Expense {
  id:string; expense_date:string; category:string; description:string;
  amount_cents:number; fund:string|null; supplier:string|null;
  purchase_url:string|null; notes:string|null; created_at:string;updated_at:string;
}
export type ExpenseInput = Pick<Expense,'expense_date'|'category'|'description'|'amount_cents'|'fund'|'supplier'|'purchase_url'|'notes'> & {id?:string};
export interface PartAmount { part_id:string; share_cents:number; expense_cents:number; }
export interface MonthlyFinance {
  month:string; gross_cents:number; taxes_cents:number; shipping_cents:number; net_cents:number;
  expense_cents:number; margin_cents:number; parts:PartAmount[];
}
export interface PartTotal {
  part_id:string; name:string; active:boolean; pays_expenses:boolean;
  share_cents:number; expense_cents:number; balance_cents:number;
}
export interface FinanceOverview {
  year:number; months:MonthlyFinance[]; gross_cents:number; net_cents:number;
  expense_cents:number; margin_cents:number; allocation_enabled:boolean; parts:PartTotal[];
}
export interface AllocationPart {
  id:string; name:string; share_bp:number; pays_expenses:boolean; position:number; active:boolean;
}
export interface AllocationPlan { enabled:boolean; parts:AllocationPart[]; }
export interface AllocationPartInput { id?:string; name:string; share_bp:number; pays_expenses:boolean; }
export interface AllocationPlanInput { enabled:boolean; parts:AllocationPartInput[]; }

export type SaleReceiptMethod = 'bank_transfer' | 'cash' | 'card' | 'other';
export interface SaleReceipt {
  id: string; sale_id: string; received_at: string; amount_cents: number;
  method: SaleReceiptMethod; reference: string | null; notes: string | null; created_at: string;
}
export interface ReceiptAttachment {
  receipt_id: string; media_path: string; original_name: string;
  mime_type: 'application/pdf' | 'image/jpeg' | 'image/png' | 'image/webp';
  size_bytes: number; sha256: string; created_at: string;
}
export type SaleReceiptInput = Pick<SaleReceipt, 'sale_id' | 'received_at' | 'amount_cents' | 'method' | 'reference' | 'notes'>;
export interface MonthlyReceipt {
  month: string; amount_cents: number; count: number;
}
export interface ReceiptYearOverview {
  year: number; months: MonthlyReceipt[]; total_cents: number; count: number;
}
export interface SaleBalance {
  sale_id: string; sale_cents: number; received_cents: number; outstanding_cents: number;
}

export interface BackupScheduleStatus {
  enabled: boolean; frequency: 'DAILY' | 'WEEKLY' | 'MONTHLY'; time_of_day: string;
  weekday: number; day_of_month: number; destination: string | null; keep_last: number;
  last_success_at: string | null; last_attempt_at: string | null; last_status: 'OK' | 'ERROR' | null;
  last_message: string | null; last_directory: string | null; next_run_at: string | null;
}
export type BackupScheduleInput = Pick<BackupScheduleStatus,
  'enabled' | 'frequency' | 'time_of_day' | 'weekday' | 'day_of_month' | 'destination' | 'keep_last'>;
export interface McpClientStatus {
  id: string; name: string; configPath: string; appFound: boolean; configured: boolean; upToDate: boolean; error: string | null;
}
export interface McpSetup { command: string; args: string[]; dataDir: string; }

export interface UpdateReply {
  status: 'available' | 'current' | 'not_configured' | 'check_failed' | 'invalid_release' | 'busy' | 'stale'
    | 'backup_failed' | 'download_failed' | 'signature_invalid' | 'install_failed' | 'installing';
  currentVersion: string; version: string | null; notes: string | null; date: string | null;
}
export interface DownloadProgress { downloaded: number; total: number | null; }
