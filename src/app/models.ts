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
  'code' | 'title' | 'year' | 'technique' | 'support' | 'width_cm' |
  'height_cm' | 'depth_cm' | 'description' | 'location' | 'status'
> & { id?: string };

export interface GalleryImage {
  id: string; artwork_id: string; image_path: string;
  caption: string | null; created_at: string; sort_order: number;
}

export type MediaProblem = 'unsafe_path' | 'missing' | 'unreadable' | 'not_regular' | 'invalid_size' | 'invalid_header';
export interface MediaIssue {
  artworkId: string; imageId: string | null; role: 'primary' | 'gallery';
  path: string; problem: MediaProblem;
}
export interface MediaAudit {
  checked: number; healthy: number; problems: number; truncated: boolean; issues: MediaIssue[];
}

export interface Collector {
  id: string; name: string; email: string | null; phone: string | null;
  address: string | null; notes: string | null; created_at: string; updated_at: string;
}
export type CollectorInput = Pick<Collector, 'name' | 'email' | 'phone' | 'address' | 'notes'> & { id?: string };

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
  amount_cents:number; fund:'activity'|'savings'; supplier:string|null;
  purchase_url:string|null; notes:string|null; created_at:string;updated_at:string;
}
export type ExpenseInput = Pick<Expense,'expense_date'|'category'|'description'|'amount_cents'|'fund'|'supplier'|'purchase_url'|'notes'> & {id?:string};
export interface MonthlyFinance {
  month:string; gross_cents:number; taxes_cents:number; shipping_cents:number; net_cents:number;
  expense_cents:number; margin_cents:number; personal_cents:number; activity_cents:number;
  savings_cents:number; activity_expenses_cents:number; savings_expenses_cents:number;
}
export interface FinanceOverview {
  year:number; months:MonthlyFinance[]; gross_cents:number; net_cents:number;
  expense_cents:number; margin_cents:number; personal_cents:number;
  activity_balance_cents:number; savings_balance_cents:number;
}

export type SaleReceiptMethod = 'bank_transfer' | 'cash' | 'card' | 'other';
export interface SaleReceipt {
  id: string; sale_id: string; received_at: string; amount_cents: number;
  method: SaleReceiptMethod; reference: string | null; notes: string | null; created_at: string;
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
