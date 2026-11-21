import { invoke } from '@tauri-apps/api/core';
import type {
  Artwork, ArtworkInput, Collector, CollectorInput, Sale, SaleInput,
  Certificate, CertificateInput, Overview, BackupSummary, RestoreSummary,
  StudioSettings, Expense, ExpenseInput, FinanceOverview, GalleryImage,
  SaleReceipt, SaleReceiptInput, SaleBalance, ReceiptYearOverview, ReceiptAttachment, MediaAudit, McpSetup
} from './models';
import type { CsvRegisterKind } from './csv-registers';

// No HTTP endpoint, API token or cloud state exists in the desktop MVP.
export const api = {
  listArtworks: () => invoke<Artwork[]>('list_artworks'),
  saveArtwork: (input: ArtworkInput) => invoke<Artwork>('save_artwork', { input }),
  deleteArtwork: (id: string) => invoke<void>('delete_artwork', { id }),
  artworkImage: (artworkId: string) => invoke<string | null>('artwork_image_data', { artworkId }),
  saveArtworkImageFromPath: (artworkId: string, sourcePath: string) =>
    invoke<void>('save_artwork_image_from_path', { artworkId, sourcePath }),
  listArtworkGallery: (artworkId: string) => invoke<GalleryImage[]>('list_artwork_gallery', { artworkId }),
  addArtworkGalleryImageFromPath: (artworkId: string, sourcePath: string, caption: string | null) =>
    invoke<GalleryImage>('add_artwork_gallery_image_from_path', { artworkId, sourcePath, caption }),
  updateArtworkGalleryCaption: (imageId: string, caption: string | null) =>
    invoke<GalleryImage>('update_artwork_gallery_caption', { imageId, caption }),
  promoteArtworkGalleryImage: (artworkId: string, imageId: string) =>
    invoke<void>('promote_artwork_gallery_image', { artworkId, imageId }),
  moveArtworkGalleryImage: (artworkId: string, imageId: string, direction: 'up' | 'down') =>
    invoke<void>('move_artwork_gallery_image', { artworkId, imageId, direction }),
  artworkGalleryImageData: (imageId: string) => invoke<string>('artwork_gallery_image_data', { imageId }),
  deleteArtworkGalleryImage: (imageId: string) => invoke<void>('delete_artwork_gallery_image', { imageId }),
  auditArtworkMedia: () => invoke<MediaAudit>('audit_artwork_media'),
  listCollectors: () => invoke<Collector[]>('list_collectors'),
  saveCollector: (input: CollectorInput) => invoke<Collector>('save_collector', { input }),
  deleteCollector: (id: string) => invoke<void>('delete_collector', { id }),
  listSales: () => invoke<Sale[]>('list_sales'),
  recordSale: (input: SaleInput) => invoke<void>('record_sale', { input }),
  deleteSale: (id: string) => invoke<void>('delete_sale', { id }),
  listSaleReceipts: () => invoke<SaleReceipt[]>('list_sale_receipts'),
  listSaleBalances: () => invoke<SaleBalance[]>('list_sale_balances'),
  receiptYearOverview: (year: number) => invoke<ReceiptYearOverview>('receipt_year_overview', {year}),
  recordSaleReceipt: (input: SaleReceiptInput) => invoke<SaleReceipt>('record_sale_receipt', {input}),
  deleteSaleReceipt: (id: string) => invoke<void>('delete_sale_receipt', {id}),
  getReceiptAttachment: (receiptId: string) =>
    invoke<ReceiptAttachment | null>('get_sale_receipt_attachment', {receiptId}),
  attachReceiptFile: (receiptId: string, sourcePath: string) =>
    invoke<ReceiptAttachment>('attach_sale_receipt_file', {receiptId, sourcePath}),
  receiptAttachmentData: (receiptId: string) =>
    invoke<string>('sale_receipt_attachment_data', {receiptId}),
  removeReceiptAttachment: (receiptId: string) =>
    invoke<void>('remove_sale_receipt_attachment', {receiptId}),
  exportReceiptAttachment: (receiptId: string, destination: string) =>
    invoke<string>('export_sale_receipt_attachment', {receiptId, destination}),
  listCertificates: () => invoke<Certificate[]>('list_certificates'),
  issueCertificate: (input: CertificateInput) => invoke<Certificate>('issue_certificate', { input }),
  overview: () => invoke<Overview>('overview'),
  dataDirectory: () => invoke<string>('data_directory'),
  createBackup: (destination: string) => invoke<BackupSummary>('create_backup', { destination }),
  inspectBackup: (folder: string) => invoke<BackupSummary>('inspect_backup', { folder }),
  restoreBackup: (folder: string) => invoke<RestoreSummary>('restore_backup', { folder }),
  getSettings: () => invoke<StudioSettings>('get_studio_settings'),
  saveSettings: (input: StudioSettings) => invoke<StudioSettings>('save_studio_settings', {input}),
  listExpenses: () => invoke<Expense[]>('list_expenses'),
  saveExpense: (input: ExpenseInput) => invoke<Expense>('save_expense', {input}),
  deleteExpense: (id: string) => invoke<void>('delete_expense', {id}),
  exportLedgerCsv: (kind: CsvRegisterKind, destination: string) =>
    invoke<string>('export_ledger_csv', {kind, destination}),
  financeOverview: (year: number) => invoke<FinanceOverview>('finance_overview', {year}),
  mcpSetupInfo: () => invoke<McpSetup>('mcp_setup_info'),
  pricePreview: (widthCm: number, heightCm: number) => invoke<number|null>('price_preview', {widthCm,heightCm})
};
