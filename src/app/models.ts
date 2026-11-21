export type ArtworkStatus = 'available' | 'reserved' | 'sold' | 'archived';

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
}
export type ArtworkInput = Pick<Artwork,
  'title' | 'year' | 'technique' | 'support' | 'width_cm' |
  'height_cm' | 'depth_cm' | 'description' | 'location' | 'status'
> & { id?: string; code?: string | null };
/** For a new artwork the archive code is assigned by the application. */
export type NewArtworkRecord = ArtworkInput;
export interface NewSaleInput {
  collector_id: string | null; new_collector_name: string | null;
  amount_cents: number; sold_at: string; notes: string | null;
  tax_cents: number; shipping_cents: number;
}
export interface NewArtworkInput {
  artwork: NewArtworkRecord; cover_path: string | null; gallery_paths: string[]; sale: NewSaleInput | null;
}
export interface CodePolicy {
  prefix: string; scope: 'GLOBAL' | 'YEARLY'; padding: number; separator: '-' | '.' | '_';
}

export interface GalleryImage {
  id: string; artwork_id: string; image_path: string;
  caption: string | null; created_at: string; sort_order: number;
}

export type MediaProblem = 'unsafe_path' | 'missing' | 'unreadable' | 'not_regular' | 'invalid_size' | 'invalid_header' | 'invalid_snapshot';
export interface MediaIssue {
  artworkId: string; imageId: string | null; role: 'primary' | 'gallery' | 'certificate';
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
export interface SaleInput {
  artwork_id: string; collector_id: string; amount_cents: number; sold_at: string; notes: string | null; tax_cents: number; shipping_cents: number;
}
export interface Certificate {
  id: string; artwork_id: string; serial: string; issued_at: string;
  artist_name: string; collector_name: string | null; notes: string | null;
  snapshot_json: string; artwork_title: string; created_at: string;
}
export interface CertificateInput {
  artwork_id: string; artist_name: string; collector_name: string | null; notes: string | null;
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
export interface McpClientStatus { configPath: string; appFound: boolean; configured: boolean; upToDate: boolean; }
export interface McpSetup { command: string; args: string[]; dataDir: string; }
