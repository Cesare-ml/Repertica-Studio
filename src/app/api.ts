import type { DeletionImpact } from './deletion-impact';
import { Channel, invoke } from '@tauri-apps/api/core';
import type {
  Artwork, ArtworkInput, Brochure, Collector, CollectorInput, Sale, SaleInput,
  MailAccount, MailAccountInput, MailFolder, MailList, MailDetail, MailDraft, MailSyncState, MailOverview,
  MailDraftInput, MailSendInput, MailSendResult,
  Certificate, CertificateInput, Overview, BackupSummary, RestoreSummary,
  StudioSettings, Expense, ExpenseInput, FinanceOverview, GalleryImage,
  SaleReceipt, SaleReceiptInput, SaleBalance, ReceiptYearOverview, ReceiptAttachment, MediaAudit, McpSetup,
  NewArtworkInput, PhotoKind, CodePolicy, McpClientStatus, CollectorImage, NewCollectorInput,
  AllocationPlan, AllocationPlanInput, BackupScheduleStatus, BackupScheduleInput,
  Installment, InstallmentLineInput, PlanLine,
  EditOperation, RawFile, MediaTag, MediaDetail, MediaPage, MediaCollection, MediaMetadataInput, PreviewData, ImportResult,
  DuplicateGroup, SimilarPair, MediaExportOptions, UpdateReply, DownloadProgress
} from './models';
import type { CsvRegisterKind } from './csv-registers';

// No HTTP endpoint, API token or cloud state exists in the desktop MVP.
export const api = {
  previewArtworkMetadata: (source: string) => invoke<import('./artwork-metadata-import').ArtworkMetadataPreview>('preview_artwork_metadata', {source}),
  importArtworkMetadata: (source: string, expectedDigest: string) => invoke<number>('import_artwork_metadata', {source, expectedDigest}),
  preflightArchiveBatches: (directory: string) => invoke<import('./archive-batch-progress').BatchPlan>('preflight_archive_batches', {directory}),
  reviewArchiveBatch: (directory: string, expectedManifest: string, index: number) => invoke<import('./artwork-metadata-import').ArtworkMetadataPreview>('review_archive_batch', {directory, expectedManifest, index}),
  commitArchiveBatch: (directory: string, expectedManifest: string, index: number, expectedPreview: string) => invoke<number>('commit_archive_batch', {directory, expectedManifest, index, expectedPreview}),
  mailListAccounts: () => invoke<MailAccount[]>('mail_list_accounts'),
  mailSaveAccount: (input: MailAccountInput) => invoke<MailAccount>('mail_save_account', {input}),
  mailRemoveAccount: (accountId: string) => invoke<void>('mail_remove_account', {accountId}),
  mailTestAccount: (accountId: string) => invoke<{ok: boolean; folders: number}>('mail_test_account', {accountId}),
  mailListFolders: (accountId: string) => invoke<MailFolder[]>('mail_list_folders', {accountId}),
  mailOverview: (accountId: string, folder: string | null) => invoke<MailOverview>('mail_overview',{accountId,folder}),
  mailListMessages: (accountId: string, folder: string) => invoke<MailList>('mail_list_messages',{accountId,folder}),
  mailReadMessage: (accountId: string, folder: string, uid: number, uidValidity: number) =>
    invoke<MailDetail>('mail_read_message',{accountId,folder,uid,uidValidity}),
  mailSetSeen: (accountId: string, folder: string, uid: number, uidValidity: number, seen: boolean) =>
    invoke<void>('mail_set_seen',{accountId,folder,uid,uidValidity,seen}),
  mailSyncState: () => invoke<MailSyncState>('mail_sync_state'),
  mailSyncNow: () => invoke<MailSyncState>('mail_sync_now'),
  mailListDrafts: (accountId: string) => invoke<MailDraft[]>('mail_list_drafts', {accountId}),
  mailSaveDraft: (input: MailDraftInput) => invoke<MailDraft>('mail_save_draft', {input}),
  mailRemoveDraft: (accountId: string, draftId: string, revision: number) =>
    invoke<void>('mail_remove_draft', {accountId,draftId,revision}),
  mailSend: (input: MailSendInput) => invoke<MailSendResult>('mail_send', {input}),
  listBrochures: () => invoke<Brochure[]>('list_brochures'),
  saveBrochure: (input: Brochure) => invoke<Brochure>('save_brochure', { input }),
  deleteBrochure: (id: string) => invoke<void>('delete_brochure', { id }),
  exportBrochurePdf: (brochureId: string, destination: string) =>
    invoke<string>('export_brochure_pdf', { brochureId, destination }),
  listArtworks: () => invoke<Artwork[]>('list_artworks'),
  saveArtwork: ({ expected_updated_at, ...input }: ArtworkInput) => invoke<Artwork>('save_artwork', { input, expectedUpdatedAt: expected_updated_at ?? null }),
  createArtworkFull: (input: NewArtworkInput) => invoke<Artwork>('create_artwork_full', { input }),
  /** `cascade` also deletes the certificates and the sale of the artwork: only after the user confirmed the impact. */
  deleteArtwork: (id: string, cascade: boolean, expectedRevision: string) => invoke<void>('delete_artwork', { id, cascade, expectedRevision }),
  artworkDeletionImpact: (id: string) => invoke<DeletionImpact>('artwork_deletion_impact', { id }),
  collectorDeletionImpact: (id: string) => invoke<DeletionImpact>('collector_deletion_impact', { id }),
  deleteCertificate: (id: string) => invoke<void>('delete_certificate', { id }),
  getArchiveCodePolicy: () => invoke<CodePolicy>('get_archive_code_policy'),
  saveArchiveCodePolicy: (input: CodePolicy) => invoke<CodePolicy>('save_archive_code_policy', { input }),
  nextArchiveCode: (year: string | null) => invoke<string>('next_archive_code', { year }),
  artworkImage: (artworkId: string) => invoke<string | null>('artwork_image_data', { artworkId }),
  saveArtworkImageFromPath: (artworkId: string, sourcePath: string) =>
    invoke<void>('save_artwork_image_from_path', { artworkId, sourcePath }),
  listArtworkGallery: (artworkId: string) => invoke<GalleryImage[]>('list_artwork_gallery', { artworkId }),
  addArtworkGalleryImageFromPath: (artworkId: string, sourcePath: string, caption: string | null) =>
    invoke<GalleryImage>('add_artwork_gallery_image_from_path', { artworkId, sourcePath, caption }),
  revealInFolder: (path: string) => invoke<void>('reveal_in_folder', { path }),
  checkForUpdate: () => invoke<UpdateReply>('check_for_update'),
  installUpdate: (version: string, skipBackup: boolean, onProgress: (progress: DownloadProgress) => void) => {
    const progress = new Channel<DownloadProgress>();
    progress.onmessage = onProgress;
    return invoke<UpdateReply>('install_update', { version, skipBackup, progress });
  },
  updateArtworkGalleryKind: (imageId: string, kind: PhotoKind) =>
    invoke<GalleryImage>('update_artwork_gallery_kind', { imageId, kind }),
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
  saveCollector: ({ expected_updated_at, ...input }: CollectorInput) => invoke<Collector>('save_collector', { input, expectedUpdatedAt: expected_updated_at ?? null }),
  createCollectorFull: (input: NewCollectorInput) => invoke<Collector>('create_collector_full', { input }),
  deleteCollector: (id: string, cascade: boolean, expectedRevision: string) => invoke<void>('delete_collector', { id, cascade, expectedRevision }),
  listCollectorImages: (collectorId: string) => invoke<CollectorImage[]>('list_collector_images', { collectorId }),
  addCollectorImageFromPath: (collectorId: string, sourcePath: string, kind: 'PROFILE' | 'PHOTO') =>
    invoke<CollectorImage>('add_collector_image_from_path', { collectorId, sourcePath, kind }),
  collectorImageData: (imageId: string) => invoke<string>('collector_image_data', { imageId }),
  deleteCollectorImage: (imageId: string) => invoke<void>('delete_collector_image', { imageId }),
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
  listArtworkRawFiles: () => invoke<RawFile[]>('list_artwork_raw_files'),
  attachArtworkRaw: (artworkId: string, sourcePath: string) => invoke<RawFile>('attach_artwork_raw', { artworkId, sourcePath }),
  removeArtworkRaw: (artworkId: string) => invoke<void>('remove_artwork_raw', { artworkId }),
  exportArtworkRaw: (artworkId: string, destination: string) => invoke<string>('export_artwork_raw', { artworkId, destination }),
  mediaList: (args: { query?: string; tagId?: string | null; collectionId?: string | null; state?: 'ACTIVE' | 'TRASHED'; offset?: number; limit?: number }) =>
    invoke<MediaPage>('media_list', { query: args.query ?? null, tagId: args.tagId ?? null, collectionId: args.collectionId ?? null,
      state: args.state ?? 'ACTIVE', offset: args.offset ?? 0, limit: args.limit ?? 60 }),
  mediaDetail: (mediaId: string) => invoke<MediaDetail>('media_detail', { mediaId }),
  mediaUpdate: (mediaId: string, input: MediaMetadataInput) => invoke<MediaDetail>('media_update', { mediaId, input }),
  mediaThumbnail: (mediaId: string) => invoke<string>('media_thumbnail', { mediaId }),
  mediaPreview: (mediaId: string, maxSide = 1400, operations: EditOperation[] | null = null, revisionId: string | null = null) =>
    invoke<PreviewData>('media_preview', { mediaId, revisionId, maxSide, operations }),
  mediaFilePreview: (path: string, maxSide = 1400, operations: EditOperation[] | null = null) =>
    invoke<PreviewData>('media_file_preview', { path, maxSide, operations }),
  mediaApplyEdit: (mediaId: string, operations: EditOperation[]) => invoke<MediaDetail>('media_apply_edit', { mediaId, operations }),
  mediaRestoreRevision: (mediaId: string, revisionId: string) => invoke<MediaDetail>('media_restore_revision', { mediaId, revisionId }),
  mediaImport: (paths: string[]) => invoke<ImportResult>('media_import', { paths }),
  mediaTags: () => invoke<MediaTag[]>('media_tags'),
  mediaCreateTag: (name: string) => invoke<MediaTag>('media_create_tag', { name }),
  mediaSetTags: (mediaId: string, tagIds: string[]) => invoke<MediaDetail>('media_set_tags', { mediaId, tagIds }),
  mediaCollections: () => invoke<MediaCollection[]>('media_collections'),
  mediaCreateCollection: (name: string) => invoke<MediaCollection>('media_create_collection', { name }),
  mediaDeleteCollection: (collectionId: string) => invoke<void>('media_delete_collection', { collectionId }),
  mediaCollectionAdd: (collectionId: string, mediaId: string) => invoke<MediaDetail>('media_collection_add', { collectionId, mediaId }),
  mediaCollectionRemove: (collectionId: string, mediaId: string) => invoke<MediaDetail>('media_collection_remove', { collectionId, mediaId }),
  mediaTrash: (mediaId: string) => invoke<MediaDetail>('media_trash', { mediaId }),
  mediaUntrash: (mediaId: string) => invoke<MediaDetail>('media_untrash', { mediaId }),
  mediaPurge: (mediaId: string) => invoke<void>('media_purge', { mediaId }),
  mediaExport: (mediaId: string, kind: 'current' | 'original' | 'custom', options: MediaExportOptions | null, destination: string) =>
    invoke<string>('media_export', { mediaId, kind, options, destination }),
  mediaIdForPath: (path: string) => invoke<string | null>('media_id_for_path', { path }),
  mediaExactDuplicates: () => invoke<DuplicateGroup[]>('media_exact_duplicates'),
  mediaSimilar: () => invoke<SimilarPair[]>('media_similar'),
  artworkSetCoverFromMedia: (artworkId: string, mediaId: string) => invoke<void>('artwork_set_cover_from_media', { artworkId, mediaId }),
  artworkAddGalleryFromMedia: (artworkId: string, mediaId: string, caption: string | null, kind: PhotoKind = 'OTHER') =>
    invoke<void>('artwork_add_gallery_from_media', { artworkId, mediaId, caption, kind }),
  listSaleInstallments: () => invoke<Installment[]>('list_sale_installments'),
  saveSaleInstallments: (saleId: string, items: InstallmentLineInput[]) =>
    invoke<void>('save_sale_installments', { saleId, items }),
  generateInstallmentPlan: (amountCents: number, count: number, firstDue: string) =>
    invoke<PlanLine[]>('generate_installment_plan', { amountCents, count, firstDue }),
  getBackupSchedule: () => invoke<BackupScheduleStatus>('get_backup_schedule'),
  saveBackupSchedule: (input: BackupScheduleInput) => invoke<BackupScheduleStatus>('save_backup_schedule', {input}),
  runBackupNow: () => invoke<BackupScheduleStatus>('run_backup_now'),
  getAllocationPlan: () => invoke<AllocationPlan>('get_allocation_plan'),
  saveAllocationPlan: (input: AllocationPlanInput) => invoke<AllocationPlan>('save_allocation_plan', {input}),
  financeOverview: (year: number) => invoke<FinanceOverview>('finance_overview', {year}),
  mcpSetupInfo: () => invoke<McpSetup>('mcp_setup_info'),
  mcpClientsStatus: () => invoke<McpClientStatus[]>('mcp_clients_status'),
  mcpClientConfigure: (clientId: string) => invoke<McpClientStatus>('mcp_client_configure', { clientId }),
  mcpClientRemove: (clientId: string) => invoke<McpClientStatus>('mcp_client_remove', { clientId }),
};
