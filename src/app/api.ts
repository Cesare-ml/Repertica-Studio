import { invoke } from '@tauri-apps/api/core';
import type {
  Artwork, ArtworkInput, Collector, CollectorInput, Sale, SaleInput,
  Certificate, CertificateInput, Overview
} from './models';

// No HTTP endpoint, API token or cloud state exists in the desktop MVP.
export const api = {
  listArtworks: () => invoke<Artwork[]>('list_artworks'),
  saveArtwork: (input: ArtworkInput) => invoke<Artwork>('save_artwork', { input }),
  deleteArtwork: (id: string) => invoke<void>('delete_artwork', { id }),
  artworkImage: (artworkId: string) => invoke<string | null>('artwork_image_data', { artworkId }),
  saveArtworkImage: (artworkId: string, dataBase64: string) =>
    invoke<void>('save_artwork_image', { artworkId, dataBase64 }),
  listCollectors: () => invoke<Collector[]>('list_collectors'),
  saveCollector: (input: CollectorInput) => invoke<Collector>('save_collector', { input }),
  deleteCollector: (id: string) => invoke<void>('delete_collector', { id }),
  listSales: () => invoke<Sale[]>('list_sales'),
  recordSale: (input: SaleInput) => invoke<void>('record_sale', { input }),
  deleteSale: (id: string) => invoke<void>('delete_sale', { id }),
  listCertificates: () => invoke<Certificate[]>('list_certificates'),
  issueCertificate: (input: CertificateInput) => invoke<Certificate>('issue_certificate', { input }),
  overview: () => invoke<Overview>('overview'),
  dataDirectory: () => invoke<string>('data_directory')
};
