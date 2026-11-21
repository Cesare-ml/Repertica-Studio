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
  artwork_id: string; collector_id: string; amount_cents: number; sold_at: string; notes: string | null;
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
