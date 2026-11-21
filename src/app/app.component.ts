import { CommonModule } from '@angular/common';
import { ChangeDetectorRef, Component, OnInit } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { isTauri } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { locale as osLocale } from '@tauri-apps/plugin-os';
import { AppLanguage, chooseLanguage, translate } from './i18n';
import { open, save } from '@tauri-apps/plugin-dialog';
import { api } from './api';
import { csvRegisterOptions, type CsvRegisterKind } from './csv-registers';
import { PaymentPlanComponent } from './payment-plan.component';
import { MediaCenterComponent } from './media-center.component';
import { MediaEditorComponent } from './media-editor.component';
import { emptyPayment, installmentRows, paymentToInput, toCents, type PaymentDraft } from './payments';
import { buildSaleIndexes, filterArtworks, filterCollectors } from './archive-filters';
import type { ArtworkOrder, CollectorFilter } from './archive-filters';
import type {
  Artwork, ArtworkInput, ArtworkStatus, Collector, CollectorInput, Sale,
  Certificate, CertificateInput, Overview, BackupSummary, RestoreSummary,
  StudioSettings, Expense, ExpenseInput, FinanceOverview, GalleryImage,
  SaleReceipt, SaleReceiptInput, SaleReceiptMethod, SaleBalance, ReceiptYearOverview,
  MediaAudit, MediaIssue, ReceiptAttachment, McpSetup, McpClientStatus, CodePolicy, NewArtworkInput, CollectorImage, AllocationPlan, AllocationPart, BackupScheduleStatus, Installment,
  EditOperation, PhotoSource, RawFile, MediaItem, PreviewData
} from './models';

/** A picture chosen for a new artwork, possibly edited before it is stored. */
interface PendingPhoto { mediaId: string; label: string; thumb: string; }
type Section = 'overview' | 'artworks' | 'media' | 'collectors' | 'sales' | 'certificates' | 'finance' | 'settings';
const freshArtwork = (): ArtworkInput => ({
  code: null, title: '', year: null, technique: null, support: null,
  width_cm: null, height_cm: null, depth_cm: null, description: null,
  location: null, status: 'available'
});
const freshCollector = (): CollectorInput => ({
  first_name: '', last_name: null, email: null, phone: null, address: null, notes: null,
  birth_date: null, birth_time: null, birth_place: null,
  instagram: null, facebook: null, tiktok: null, website: null, linkedin: null
});
const today = () => new Date().toLocaleDateString('sv-SE');
const statusLabels: Record<ArtworkStatus, string> = {
  available: 'Disponibile', reserved: 'Riservata', sold: 'Venduta', archived: 'In archivio'
};

@Component({
  selector: 'rp-root',
  standalone: true,
  imports: [CommonModule, FormsModule, PaymentPlanComponent, MediaCenterComponent, MediaEditorComponent],
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.css']
})
export class AppComponent implements OnInit {
  readonly appVersion = '0.1.0';
  readonly sections: { id: Section; label: string }[] = [
    { id: 'overview', label: 'Panoramica' },
    { id: 'artworks', label: 'Opere' },
    { id: 'media', label: 'Libreria immagini' },
    { id: 'collectors', label: 'Collezionisti' },
    { id: 'sales', label: 'Vendite' },
    { id: 'certificates', label: 'Certificati' },
    { id: 'finance', label: 'Economia' },
    { id: 'settings', label: 'Impostazioni' }
  ];
  readonly statusLabels = statusLabels;
  language: AppLanguage = 'en';
  systemLanguage: AppLanguage = chooseLanguage(typeof navigator !== 'undefined' ? navigator.language : 'en');
  languageChoice = 'system';
  readonly languageOptions = [
    {code:'system',name:'System / Sistema'},
    {code:'en',name:'English'},
    {code:'it',name:'Italiano'},
    {code:'es',name:'Español'},
    {code:'fr',name:'Français'},
    {code:'de',name:'Deutsch'}
  ];
  section: Section = 'overview';
  initialized = false;
  loading = true;
  busy = false;
  errorMessage = '';
  notice = '';
  search = '';
  artworkStatusFilter: ArtworkStatus | 'all' = 'all';
  artworkCollectorFilter = 'all';
  artworkOrder: ArtworkOrder = 'recent';
  expandedCollectorId: string | null = null;
  salesByCollector: Record<string, Sale[]> = {};
  salesByArtwork: Record<string, Sale> = {};
  artworks: Artwork[] = [];
  collectors: Collector[] = [];
  sales: Sale[] = [];
  receipts: SaleReceipt[] = [];
  balancesBySale: Record<string, SaleBalance> = {};
  selectedReceiptSale: Sale | null = null;
  receiptAttachments: Record<string, ReceiptAttachment | null> = {};
  receiptAttachmentsLoading = false;
  receiptPreview: { name: string; src: string } | null = null;
  private receiptAttachmentLoadVersion = 0;
  receiptDraft = {
    amount_eur: null as number | null, received_at: today(),
    method: 'bank_transfer' as SaleReceiptMethod,
    reference: null as string | null, notes: null as string | null
  };
  expenses: Expense[] = [];
  finances: FinanceOverview | null = null;
  receiptYear: ReceiptYearOverview | null = null;
  financeYear = new Date().getFullYear();
  financeYears = Array.from({length: 8}, (_,i)=>new Date().getFullYear()-i);
  settings: StudioSettings = {
    artist_coefficient:1,base_price_cm2_cents:16,personal_bp:3334,activity_bp:3333,
    savings_bp:3333,treasury_start_date:null,default_tax_bp:0,ui_language:null
  };
  settingsDraft: StudioSettings = {...this.settings};
  rateEuro = .16;
  installments: Installment[] = [];
  installmentsBySale: Record<string, Installment[]> = {};
  salePayment: PaymentDraft = emptyPayment(today());
  newSalePayment: PaymentDraft = emptyPayment(today());
  planDraft: PaymentDraft = emptyPayment(today());
  planEditing = false;
  schedule: BackupScheduleStatus | null = null;
  scheduleDraft = { enabled: false, frequency: 'DAILY' as BackupScheduleStatus['frequency'], time_of_day: '20:00',
    weekday: 0, day_of_month: 1, destination: null as string | null, keep_last: 0 };
  readonly weekdayIndexes = [0, 1, 2, 3, 4, 5, 6];
  allocationPlan: AllocationPlan = { enabled: true, parts: [] };
  allocationDraft = { enabled: true, parts: [] as { id?: string; name: string; percent: number; pays_expenses: boolean }[] };
  defaultTaxPercent = 0;
  estimateCents: number | null = null;
  expenseDialog = false;
  readonly expenseCategories = [
    {code:'painting-materials',label:'Materiali pittura'},
    {code:'shipping',label:'Spedizioni e imballaggi'},
    {code:'equipment',label:'Attrezzatura'},
    {code:'software',label:'Software e servizi'},
    {code:'exhibitions',label:'Mostre e fiere'},
    {code:'marketing',label:'Marketing'},
    {code:'training',label:'Formazione'},
    {code:'other',label:'Altro'}
  ];
  expenseDraft = {
    id: undefined as string | undefined,expense_date:today(),
    category:'painting-materials',description:'',amount_eur:null as number|null,
    fund:null as string|null,supplier:null as string|null,
    purchase_url:null as string|null,notes:null as string|null
  };

  certificates: Certificate[] = [];
  overview: Overview = { artworks: 0, collectors: 0, sales: 0, certificates: 0, revenue_cents: 0 };
  dataPath = '';
  backupDestination = '';
  lastBackup: BackupSummary | null = null;
  restoreCandidate: BackupSummary | null = null;
  lastRestore: RestoreSummary | null = null;
  mediaAudit: MediaAudit | null = null;
  showGuide = false;
  private readonly guideKey = 'repertica-first-run-guide';
  mcpSetup: McpSetup | null = null;
  mcpClients: McpClientStatus[] = [];

  thumbnails: Record<string, string> = {};
  galleryArtwork: Artwork | null = null;
  galleryImages: GalleryImage[] = [];
  galleryPreviews: Record<string, string> = {};
  galleryCaption = '';
  editingGalleryCaptionId: string | null = null;
  editingGalleryCaption = '';
  artworkDialog = false;
  collectorDialog = false;
  saleDialog = false;
  certificateDialog = false;
  artworkDraft: ArtworkInput = freshArtwork();
  nextCode = '';
  get editingArtwork(): Artwork | null { return this.artworks.find(a => a.id === this.artworkDraft.id) ?? null; }
  newCover: PendingPhoto | null = null;
  newGallery: PendingPhoto[] = [];
  newRawPath: string | null = null;
  /** Library picture being edited (cover chosen for a new artwork, stored cover or gallery photo). */
  photoEditor: { mediaId: string; label: string; source: PreviewData; done: () => Promise<void> | void } | null = null;
  /** Pictures imported while the artwork dialog is open: unused ones are moved to the trash afterwards. */
  private readonly sessionImports = new Set<string>();
  libraryPicker: { purpose: 'new-cover' | 'new-gallery' | 'existing-cover' | 'existing-gallery'; artworkId?: string } | null = null;
  rawByArtwork: Record<string, RawFile> = {};
  mediaOpenId: string | null = null;
  readonly rawExtensions = ['cr2', 'cr3', 'crw', 'nef', 'nrw', 'arw', 'srf', 'sr2', 'dng', 'orf', 'rw2', 'raf', 'pef', 'srw', '3fr',
    'fff', 'erf', 'kdc', 'dcr', 'mrw', 'x3f', 'iiq', 'rwl', 'raw', 'gpr', 'mef', 'mos', 'cap', 'rwz'];
  withSale = false;
  newSaleDraft = this.freshNewSale();
  codePolicy: CodePolicy = { prefix: 'ART', scope: 'GLOBAL', padding: 4, separator: '-' };
  collectorDraft: CollectorInput = freshCollector();
  collectorFilter: CollectorFilter = 'all';
  collectorThumbs: Record<string, string> = {};
  collectorImages: CollectorImage[] = [];
  collectorPreviews: Record<string, string> = {};
  newCollectorProfile: string | null = null;
  newCollectorPhotos: string[] = [];
  saleDraft = { artwork_id: '', collector_id: '', amount_eur: null as number | null,
    taxes_eur: 0, shipping_eur: 0, sold_at: today(), notes: null as string | null };
  certificateDraft: CertificateInput = { artwork_id: '', artist_name: '', collector_name: null, notes: null };
  selectedCertificate: Certificate | null = null;
  selectedSnapshot: Artwork | null = null;

  constructor(private readonly change: ChangeDetectorRef) {}
  tr(key: string): string { return translate(key, this.language); }
  private applyLanguage(language: AppLanguage): void {
    this.language = language;
    document.documentElement.lang = language;
  }
  async ngOnInit(): Promise<void> {
    if (!isTauri()) {
      this.errorMessage = 'Apri Repertica Studio con npm run tauri:dev: il browser non può accedere al database locale.';
      this.loading = false;
      return;
    }
    try {
      const [system, stored] = await Promise.all([osLocale().catch(() => null), api.getSettings()]);
      this.systemLanguage = chooseLanguage(system ?? navigator.language);
      this.settings = stored;
      this.settingsDraft = {...stored};
      this.rateEuro = stored.base_price_cm2_cents / 100;
      this.defaultTaxPercent = stored.default_tax_bp / 100;
      this.languageChoice = stored.ui_language ?? 'system';
      this.applyLanguage(chooseLanguage(stored.ui_language ?? system ?? navigator.language));
      this.initialized = true;
      this.showGuide = !this.guideDismissed();
      try { this.codePolicy = await api.getArchiveCodePolicy(); } catch { /* keep defaults */ }
      await this.loadAllocationPlan();
      await this.loadSchedule();
      // The timer runs in the native side; it reports each automatic backup here.
      void listen<BackupScheduleStatus>('auto-backup-finished', event => {
        this.schedule = event.payload;
        this.notice = event.payload.last_status === 'OK' ? this.tr('Backup automatico completato') : '';
        if (event.payload.last_status === 'ERROR') this.errorMessage = event.payload.last_message ?? '';
        this.change.detectChanges();
      }).catch(() => undefined);
      await this.refresh();
    } catch (error) {
      this.errorMessage = this.error(error);
      this.loading = false;
    }
  }
  navigate(section: Section): void {
    this.section = section;
    this.mediaOpenId = null;
    this.search = '';
    this.errorMessage = '';
    this.notice = '';
    if (section === 'artworks') {
      this.artworkStatusFilter = 'all';
      this.artworkCollectorFilter = 'all';
      this.artworkOrder = 'recent';
    }
    if (section === 'collectors') this.expandedCollectorId = null;
    if (section === 'settings' && this.initialized) void this.loadMcpSetup();
  }
  private guideDismissed(): boolean {
    try { return localStorage.getItem(this.guideKey) === 'done'; } catch { return false; }
  }
  dismissGuide(): void {
    try { localStorage.setItem(this.guideKey, 'done'); } catch { /* the guide simply reappears next time */ }
    this.showGuide = false;
  }
  reopenGuide(): void {
    try { localStorage.removeItem(this.guideKey); } catch { /* ignore */ }
    this.navigate('overview');
    this.showGuide = true;
  }
  async loadMcpSetup(): Promise<void> {
    try {
      const [setup, clients] = await Promise.all([api.mcpSetupInfo(), api.mcpClientsStatus()]);
      this.mcpSetup = setup;
      // Installed assistants first.
      this.mcpClients = [...clients].sort((a, b) => Number(b.appFound) - Number(a.appFound));
    } catch (error) {
      this.errorMessage = this.error(error);
    }
    this.change.detectChanges();
  }
  clientStatusLabel(client: McpClientStatus): string {
    if (client.error) return this.tr('Configurazione non leggibile');
    if (!client.appFound) return this.tr('Non trovato su questo computer');
    if (!client.configured) return this.tr('Non collegato');
    return client.upToDate ? this.tr('Collegato') : this.tr('Collegato, da aggiornare');
  }
  private replaceClient(updated: McpClientStatus): void {
    this.mcpClients = this.mcpClients.map(c => c.id === updated.id ? updated : c);
  }
  async connectClient(client: McpClientStatus): Promise<void> {
    if (this.busy) return;
    if (!window.confirm(this.tr('Verrà aggiunto Repertica Studio alla configurazione di') + ' ' + client.name + '. ' + this.tr('Il file modificato è:') + '\n\n' + client.configPath +
        '\n\n' + this.tr('Una copia del file originale viene conservata accanto ad esso. Continuare?'))) return;
    await this.run(async () => { this.replaceClient(await api.mcpClientConfigure(client.id)); },
      this.tr('Collegamento completato: riavvia l’assistente'));
  }
  async disconnectClient(client: McpClientStatus): Promise<void> {
    if (this.busy) return;
    await this.run(async () => { this.replaceClient(await api.mcpClientRemove(client.id)); },
      this.tr('Collegamento rimosso: riavvia l’assistente'));
  }
  get mcpConfigJson(): string {
    const setup = this.mcpSetup;
    if (!setup) return '';
    return JSON.stringify({ mcpServers: { 'repertica-studio': { command: setup.command, args: setup.args } } }, null, 2);
  }
  get mcpTomlSnippet(): string {
    const setup = this.mcpSetup;
    if (!setup) return '';
    // TOML literal strings need no escaping of backslashes.
    const literal = (value: string) => value.includes("'") ? JSON.stringify(value) : `'${value}'`;
    return `[mcp_servers.repertica-studio]\ncommand = ${literal(setup.command)}\nargs = [${setup.args.map(literal).join(', ')}]`;
  }
  get mcpCommands(): string {
    const setup = this.mcpSetup;
    if (!setup) return '';
    const quote = (value: string) => `"${value}"`;
    const call = `repertica-studio -- ${quote(setup.command)} ${setup.args.map(a => a.startsWith('--') ? a : quote(a)).join(' ')}`;
    return `codex mcp add ${call}\nclaude mcp add ${call}`;
  }
  async copyText(text: string): Promise<void> {
    try {
      await navigator.clipboard.writeText(text);
      this.notice = this.tr('Configurazione copiata');
    } catch {
      this.errorMessage = this.tr('Impossibile copiare: seleziona il testo e copialo a mano');
    }
    this.change.detectChanges();
  }
  get currentTitle(): string {
    return this.tr(this.sections.find(item => item.id === this.section)?.label ?? 'Repertica Studio');
  }
  get visibleArtworks(): Artwork[] {
    return filterArtworks(this.artworks, {
      query: this.search,
      status: this.artworkStatusFilter,
      collectorId: this.artworkCollectorFilter,
      order: this.artworkOrder
    }, { byCollector: this.salesByCollector, byArtwork: this.salesByArtwork });
  }
  get visibleCollectors(): Collector[] {
    return filterCollectors(this.collectors, this.search,
      { byCollector: this.salesByCollector, byArtwork: this.salesByArtwork }, this.collectorFilter);
  }
  showCollectorArtworks(collectorId: string): void {
    this.navigate('artworks');
    this.artworkCollectorFilter = collectorId;
  }
  openCollectorFromArtwork(artworkId: string): void {
    const sale = this.salesByArtwork[artworkId];
    if (!sale) return;
    this.navigate('collectors');
    this.expandedCollectorId = sale.collector_id;
  }
  toggleCollectorAcquisitions(collectorId: string): void {
    this.expandedCollectorId = this.expandedCollectorId === collectorId ? null : collectorId;
  }
  get sellableArtworks(): Artwork[] { return this.artworks.filter(a => ['available','reserved'].includes(a.status)); }
  get money(): string { return this.euros(this.overview.revenue_cents); }
  euros(cents: number): string {
    return new Intl.NumberFormat(this.language, { style: 'currency', currency: 'EUR' }).format(cents / 100);
  }
  date(value: string): string {
    const d = new Date(value.length === 10 ? value + 'T12:00:00' : value);
    return Number.isNaN(d.getTime()) ? value : d.toLocaleDateString(this.language);
  }
  dimensions(artwork: Artwork): string {
    const parts = [artwork.height_cm, artwork.width_cm, artwork.depth_cm].filter(v => v !== null);
    return parts.length ? parts.map(v => String(v)).join(' × ') + ' cm' : '—';
  }
  async refresh(): Promise<void> {
    this.loading = true;
    try {
      const [artworks, collectors, sales, certificates, overview, dataPath, expenses, finances,
        receipts, balances, receiptYear] = await Promise.all([
        api.listArtworks(), api.listCollectors(), api.listSales(), api.listCertificates(),
        api.overview(), api.dataDirectory(), api.listExpenses(), api.financeOverview(this.financeYear),
        api.listSaleReceipts(), api.listSaleBalances(), api.receiptYearOverview(this.financeYear)
      ]);
      this.artworks=artworks; this.collectors=collectors; this.sales=sales; this.certificates=certificates;
      this.overview=overview; this.dataPath=dataPath; this.expenses=expenses; this.finances=finances;
      this.receipts=receipts;
      this.installments = await api.listSaleInstallments();
      this.rawByArtwork = Object.fromEntries((await api.listArtworkRawFiles()).map(f => [f.artwork_id, f]));
      this.installmentsBySale = this.installments.reduce((map, item) => { (map[item.sale_id] ??= []).push(item); return map; }, {} as Record<string, Installment[]>);
      this.receiptYear=receiptYear;
      this.balancesBySale=Object.fromEntries(balances.map(b=>[b.sale_id,b]));
      // Build indexes once per refresh instead of scanning the full sales register for every row.
      const indexes = buildSaleIndexes(this.sales);
      this.salesByCollector = indexes.byCollector;
      this.salesByArtwork = indexes.byArtwork;
      if (this.artworkCollectorFilter !== 'all' &&
          !this.collectors.some(c => c.id === this.artworkCollectorFilter)) {
        this.artworkCollectorFilter = 'all';
      }
      if (this.expandedCollectorId && !this.collectors.some(c => c.id === this.expandedCollectorId)) {
        this.expandedCollectorId = null;
      }
      this.errorMessage = '';
      const wanted = this.artworks.filter(a => a.image_path && !this.thumbnails[a.id]).slice(0, 35);
      await Promise.all(wanted.map(async a => {
        try {
          const image = await api.artworkImage(a.id);
          if (image) this.thumbnails[a.id] = image;
        } catch { /* a missing local image must not hide the archive */ }
      }));
      const profiles = this.collectors.filter(c => c.profile_image_id && !this.collectorThumbs[c.profile_image_id]).slice(0, 60);
      await Promise.all(profiles.map(async c => {
        try { this.collectorThumbs[c.profile_image_id!] = await api.collectorImageData(c.profile_image_id!); }
        catch { /* a missing photograph must not hide the list */ }
      }));
    } catch (error) {
      this.errorMessage = this.error(error);
    } finally {
      this.loading = false;
      this.change.detectChanges();
    }
  }
  private error(error: unknown): string {
    return typeof error === 'string' ? error : error instanceof Error ? error.message : 'Operazione non riuscita';
  }
  private async run(action: () => Promise<unknown>, success: string): Promise<boolean> {
    if (this.busy) return false;
    this.busy = true; this.errorMessage = ''; this.notice = '';
    try {
      await action();
      this.notice = success;
      await this.refresh();
      return true;
    } catch (error) {
      this.errorMessage = this.error(error);
      return false;
    } finally {
      this.busy = false;
      this.change.detectChanges();
    }
  }
  async verifyArtworkMedia(): Promise<void> {
    if (this.busy || !this.initialized) return;
    this.busy = true;
    this.mediaAudit = null;
    this.errorMessage = '';
    try {
      this.mediaAudit = await api.auditArtworkMedia();
    } catch (error) {
      this.errorMessage = this.error(error);
    } finally {
      this.busy = false;
      this.change.detectChanges();
    }
  }
  mediaProblemLabel(problem: MediaIssue['problem']): string {
    const labels: Record<MediaIssue['problem'], string> = {
      unsafe_path: 'Percorso non sicuro',
      missing: 'File mancante',
      unreadable: 'File non leggibile',
      not_regular: 'File non regolare o collegamento simbolico',
      invalid_size: 'Dimensione file non valida',
      invalid_header: 'Formato o intestazione non valida',
      invalid_snapshot: 'Snapshot del certificato non valido'
    };
    return this.tr(labels[problem]);
  }
  mediaArtworkLabel(id: string): string {
    return this.artworks.find(a => a.id === id)?.title || this.collectors.find(c => c.id === id)?.name || id;
  }
  async chooseBackupDestination(): Promise<void> {
    if (this.busy || !this.initialized) return;
    try {
      const selection = await open({ directory: true, multiple: false, title: 'Cartella in cui conservare i backup' });
      if (typeof selection === 'string') this.backupDestination = selection;
    } catch (error) { this.errorMessage = this.error(error); }
    this.change.detectChanges();
  }
  async makeBackup(): Promise<void> {
    if (!this.backupDestination || this.busy) return;
    await this.run(async () => {
      this.lastBackup = await api.createBackup(this.backupDestination);
    }, 'Backup completato e verificato. Conserva la cartella risultante in un luogo sicuro.');
  }
  async exportLedger(kind: CsvRegisterKind): Promise<void> {
    if (this.busy || !this.initialized) return;
    const option = csvRegisterOptions[kind];
    try {
      const destination = await save({
        title: this.tr(option.title),
        defaultPath: `ReperticaStudio-${option.filename}-${today()}.csv`,
        filters: [{ name: 'CSV', extensions: ['csv'] }]
      });
      if (typeof destination !== 'string') return;
      await this.run(
        () => api.exportLedgerCsv(kind, destination),
        this.tr(option.success)
      );
    } catch (error) {
      this.errorMessage = this.error(error);
    }
    this.change.detectChanges();
  }
  async chooseRestoreFolder(): Promise<void> {
    if (this.busy || !this.initialized) return;
    try {
      const selection = await open({ directory: true, multiple: false, title: 'Scegli una cartella di backup ReperticaStudio-backup-…' });
      if (typeof selection !== 'string') return;
      this.busy = true;
      this.errorMessage = '';
      this.restoreCandidate = await api.inspectBackup(selection);
    } catch (error) {
      this.restoreCandidate = null;
      this.errorMessage = this.error(error);
    } finally {
      this.busy = false;
      this.change.detectChanges();
    }
  }
  async restoreSelectedBackup(): Promise<void> {
    if (this.busy || !this.restoreCandidate) return;
    const candidate = this.restoreCandidate;
    if (!window.confirm('RIPRISTINO ARCHIVIO\\n\\nL’archivio locale attuale verrà sostituito con quello del backup. Una copia completa dell’archivio precedente verrà conservata accanto alla cartella dati.\\n\\nContinuare?')) return;
    this.busy = true; this.errorMessage = ''; this.notice = '';
    try {
      this.lastRestore = await api.restoreBackup(candidate.directory);
      this.thumbnails = {};
      this.closeGallery();
      this.restoreCandidate = null;
      await this.refresh();
      await this.loadAllocationPlan();
      await this.loadSchedule();
      this.notice = 'Archivio ripristinato e verificato. Copia precedente: ' + this.lastRestore.previousArchive;
    } catch (error) {
      this.errorMessage = this.error(error);
    } finally {
      this.busy = false;
      this.change.detectChanges();
    }
  }
  bytes(value: number): string {
    return new Intl.NumberFormat('it-IT', { maximumFractionDigits: 1 }).format(value / 1048576) + ' MiB';
  }
  async loadFinanceYear(): Promise<void> {
    try {
      [this.finances,this.receiptYear] = await Promise.all([
        api.financeOverview(this.financeYear), api.receiptYearOverview(this.financeYear)
      ]);
      this.errorMessage = '';
    } catch (e) { this.errorMessage = this.error(e); }
    this.change.detectChanges();
  }
  monthName(month: string): string {
    const date = new Date(month + '-01T12:00:00');
    return date.toLocaleDateString(this.language, {month:'short'});
  }
  chartWidth(value: number): number {
    const limit = Math.max(1,...(this.finances?.months.map(m=>Math.max(m.gross_cents,m.expense_cents)) ?? [1]));
    return Math.max(0,Math.min(100,Math.round(value/limit*100)));
  }
  categoryLabel(code:string):string {
    const value=this.expenseCategories.find(c=>c.code===code);
    return value ? this.tr(value.label) : code;
  }
  async saveEconomySettings(): Promise<void> {
    const base=Math.round(this.rateEuro*100);
    const tax=Math.round(this.defaultTaxPercent*100);
    const draft: StudioSettings = {...this.settingsDraft,
      base_price_cm2_cents:base,default_tax_bp:tax,
      ui_language:this.languageChoice==='system'?null:this.languageChoice as AppLanguage};
    if (await this.run(async()=>{this.settings=await api.saveSettings(draft);this.settingsDraft={...this.settings};},
      this.tr('Salva impostazioni'))) {
      this.applyLanguage(this.languageChoice==='system'?this.systemLanguage:chooseLanguage(this.languageChoice));
      await this.loadFinanceYear();
    }
  }
  weekdayLabel(day: number): string {
    // 2024-01-01 was a Monday.
    return new Date(2024, 0, 1 + day).toLocaleDateString(this.language, { weekday: 'long' });
  }
  dateTime(value: string): string {
    const parsed = new Date(value);
    return Number.isNaN(parsed.getTime()) ? value : parsed.toLocaleString(this.language, { dateStyle: 'medium', timeStyle: 'short' });
  }
  async loadSchedule(): Promise<void> {
    try {
      this.schedule = await api.getBackupSchedule();
      const s = this.schedule;
      this.scheduleDraft = { enabled: s.enabled, frequency: s.frequency, time_of_day: s.time_of_day, weekday: s.weekday,
        day_of_month: s.day_of_month, destination: s.destination, keep_last: s.keep_last };
    } catch (error) { this.errorMessage = this.error(error); }
  }
  async chooseScheduleDestination(): Promise<void> {
    if (this.busy || !this.initialized) return;
    try {
      const selection = await open({ directory: true, multiple: false, title: this.tr('Cartella di destinazione') });
      if (typeof selection === 'string') this.scheduleDraft.destination = selection;
    } catch (error) { this.errorMessage = this.error(error); }
    this.change.detectChanges();
  }
  async saveBackupSchedule(): Promise<void> {
    const d = this.scheduleDraft;
    await this.run(async () => {
      this.schedule = await api.saveBackupSchedule({ enabled: d.enabled, frequency: d.frequency, time_of_day: d.time_of_day,
        weekday: Number(d.weekday), day_of_month: Number(d.day_of_month), destination: d.destination, keep_last: Number(d.keep_last) || 0 });
    }, this.tr('Backup automatico salvato'));
  }
  async runBackupNow(): Promise<void> {
    if (this.busy) return;
    this.busy = true; this.errorMessage = ''; this.notice = '';
    try {
      this.schedule = await api.runBackupNow();
      if (this.schedule.last_status === 'OK') this.notice = this.tr('Backup completato');
      else this.errorMessage = this.schedule.last_message ?? this.tr('Operazione non riuscita');
    } catch (error) {
      this.errorMessage = this.error(error);
    } finally {
      this.busy = false;
      this.change.detectChanges();
    }
  }
  async loadAllocationPlan(): Promise<void> {
    try {
      this.allocationPlan = await api.getAllocationPlan();
      this.resetAllocationDraft();
    } catch (error) { this.errorMessage = this.error(error); }
  }
  private resetAllocationDraft(): void {
    this.allocationDraft = {
      enabled: this.allocationPlan.enabled,
      parts: this.allocationPlan.parts.map(p => ({ id: p.id, name: p.name, percent: p.share_bp / 100, pays_expenses: p.pays_expenses }))
    };
  }
  addAllocationPart(): void {
    if (this.allocationDraft.parts.length >= 12) return;
    this.allocationDraft.parts = [...this.allocationDraft.parts, { name: '', percent: 0, pays_expenses: false }];
  }
  removeAllocationPart(index: number): void {
    if (this.allocationDraft.parts.length <= 1) return;
    this.allocationDraft.parts = this.allocationDraft.parts.filter((_, i) => i !== index);
  }
  splitAllocationEvenly(): void {
    const parts = this.allocationDraft.parts;
    if (!parts.length) return;
    const base = Math.floor(10000 / parts.length);
    parts.forEach((p, i) => p.percent = (i === parts.length - 1 ? 10000 - base * (parts.length - 1) : base) / 100);
  }
  get allocationTotal(): number {
    return this.allocationDraft.parts.reduce((sum, p) => sum + Math.round((Number(p.percent) || 0) * 100), 0) / 100;
  }
  get allocationValid(): boolean {
    const parts = this.allocationDraft.parts;
    const names = parts.map(p => p.name.trim().toLowerCase());
    return parts.length > 0 && parts.every(p => p.name.trim() && Number.isFinite(Number(p.percent)) && p.percent >= 0 && p.percent <= 100)
      && new Set(names).size === names.length && Math.round(this.allocationTotal * 100) === 10000;
  }
  async saveAllocationPlan(): Promise<void> {
    if (this.allocationDraft.enabled && !this.allocationValid) { this.errorMessage = this.tr('Deve essere 100%'); return; }
    const input = {
      enabled: this.allocationDraft.enabled,
      parts: this.allocationDraft.parts.map(p => ({ id: p.id, name: p.name.trim(),
        share_bp: Math.round((Number(p.percent) || 0) * 100), pays_expenses: p.pays_expenses }))
    };
    if (await this.run(async () => { this.allocationPlan = await api.saveAllocationPlan(input); },
      this.tr('Ripartizione salvata'))) {
      this.resetAllocationDraft();
      await this.loadFinanceYear();
    }
  }
  private readonly defaultPartNames: Record<string, string> = { personal: 'Personale', activity: 'Attività', savings: 'Risparmio' };
  /** The three default parts are shown in the interface language until the artist renames them. */
  partName(id: string, name: string): string {
    return this.defaultPartNames[id] === name ? this.tr(name) : name;
  }
  /** Parts an expense can be charged to; a part that is no longer offered stays selectable on old expenses. */
  get expenseFunds(): AllocationPart[] {
    const funds = this.allocationPlan.parts.filter(p => p.pays_expenses);
    const current = this.expenseDraft.fund;
    return current && !funds.some(p => p.id === current)
      ? [...funds, { id: current, name: this.fundLabel(current), share_bp: 0, pays_expenses: true, position: 99, active: false }]
      : funds;
  }
  fundLabel(fund: string | null): string {
    if (!fund) return '—';
    const part = this.finances?.parts.find(p => p.part_id === fund) ?? this.allocationPlan.parts.find(p => p.id === fund);
    const name = part ? ('name' in part ? part.name : '') : '';
    return name ? this.partName(fund, name) : fund;
  }
  selectLanguage(): void {
    this.applyLanguage(this.languageChoice === 'system' ? this.systemLanguage : chooseLanguage(this.languageChoice));
  }
  newExpense(): void {
    this.expenseDraft={id:undefined,expense_date:today(),category:'painting-materials',description:'',amount_eur:null,
      fund:this.allocationPlan.parts.find(p=>p.pays_expenses)?.id ?? null,supplier:null,purchase_url:null,notes:null};
    this.expenseDialog=true;
  }
  editExpense(e: Expense): void {
    this.expenseDraft={id:e.id,expense_date:e.expense_date,category:e.category,description:e.description,
      amount_eur:e.amount_cents/100,fund:e.fund,supplier:e.supplier,purchase_url:e.purchase_url,notes:e.notes};
    this.expenseDialog=true;
  }
  async saveExpense(): Promise<void> {
    const value=this.expenseDraft.amount_eur;
    if (value===null || !Number.isFinite(value) || value<=0) { this.errorMessage=this.tr('Importo spesa (€)'); return; }
    const input: ExpenseInput={
      id:this.expenseDraft.id,expense_date:this.expenseDraft.expense_date,
      description:this.expenseDraft.description,category:this.expenseDraft.category,
      amount_cents:Math.round(value*100),fund:this.expenseDraft.fund,supplier:this.expenseDraft.supplier,
      purchase_url:this.expenseDraft.purchase_url,notes:this.expenseDraft.notes
    };
    if (await this.run(()=>api.saveExpense(input),this.tr('Salva spesa'))) this.expenseDialog=false;
  }
  async deleteExpense(e:Expense):Promise<void> {
    if(!window.confirm(this.tr('Elimina') + ': '+e.description+'?'))return;
    await this.run(()=>api.deleteExpense(e.id),this.tr('Elimina'));
  }
  async previewPrice():Promise<void> {
    try {
      this.estimateCents=await api.pricePreview(this.artworkDraft.width_cm??0,this.artworkDraft.height_cm??0);
    } catch (error) {
      this.errorMessage=this.error(error);
    }
    this.change.detectChanges();
  }
  private freshNewSale() {
    return { collector_id: '', new_collector_name: null as string | null, amount_eur: null as number | null,
      taxes_eur: 0, shipping_eur: 0, sold_at: today(), notes: null as string | null };
  }
  fileName(path: string): string { return path.split(/[\\/]/).pop() ?? path; }
  async newArtwork(): Promise<void> {
    this.estimateCents = null; this.artworkDraft = freshArtwork();
    this.newCover = null; this.newGallery = []; this.newRawPath = null; this.withSale = false; this.sessionImports.clear();
    this.newSaleDraft = this.freshNewSale(); this.newSalePayment = emptyPayment(today()); this.nextCode = '';
    this.artworkDialog = true;
    await this.refreshNextCode();
  }
  async refreshNextCode(): Promise<void> {
    if (this.artworkDraft.id || !this.initialized) return;
    try { this.nextCode = await api.nextArchiveCode(this.artworkDraft.year); } catch { this.nextCode = ''; }
    this.change.detectChanges();
  }
  // Every picture goes through the library, so it can be cropped and corrected right away.
  private async toPending(id: string, label: string, large: boolean): Promise<PendingPhoto> {
    let thumb = '';
    try { thumb = large ? (await api.mediaPreview(id, 720)).data_url : await api.mediaThumbnail(id); }
    catch (error) { this.errorMessage = this.error(error); }
    return { mediaId: id, label, thumb };
  }
  private async importToLibrary(paths: string[]): Promise<MediaItem[]> {
    if (!paths.length) return [];
    const result = await api.mediaImport(paths);
    if (result.failed.length) this.errorMessage = result.failed.map(f => `${f.path}: ${f.message}`).join(' · ');
    for (const item of result.imported) this.sessionImports.add(item.id);
    return result.imported;
  }
  async openPhotoEditor(mediaId: string, label: string, done: () => Promise<void> | void): Promise<void> {
    try {
      this.photoEditor = { mediaId, label, source: await api.mediaPreview(mediaId, 1400), done };
    } catch (error) { this.errorMessage = this.error(error); }
    this.change.detectChanges();
  }
  readonly previewLibraryEdit = (operations: EditOperation[]): Promise<PreviewData> =>
    api.mediaPreview(this.photoEditor!.mediaId, 1400, operations);
  async applyLibraryEdit(operations: EditOperation[]): Promise<void> {
    const editor = this.photoEditor;
    if (!editor) return;
    try {
      await api.mediaApplyEdit(editor.mediaId, operations);
      this.photoEditor = null;
      await editor.done();
    } catch (error) { this.errorMessage = this.error(error); }
    this.change.detectChanges();
  }
  async closePhotoEditor(): Promise<void> {
    const editor = this.photoEditor;
    this.photoEditor = null;
    if (editor) await editor.done();
    this.change.detectChanges();
  }
  async pickCover(): Promise<void> {
    if (this.busy) return;
    try {
      const path = await this.pickPhoto(this.tr('Scegli copertina…'));
      if (!path) return;
      this.busy = true; this.change.detectChanges();
      const [item] = await this.importToLibrary([path]);
      if (item) {
        const label = item.title || item.original_name;
        this.newCover = await this.toPending(item.id, label, true);
        this.busy = false;
        await this.openPhotoEditor(item.id, label, async () => { this.newCover = await this.toPending(item.id, label, true); });
      }
    } catch (error) { this.errorMessage = this.error(error); }
    finally { this.busy = false; }
    this.change.detectChanges();
  }
  async pickGalleryPhotos(): Promise<void> {
    if (this.busy) return;
    try {
      const selected = await open({ directory: false, multiple: true, title: this.tr('Aggiungi fotografie…'),
        filters: [{ name: 'JPEG / PNG / WebP', extensions: ['jpg', 'jpeg', 'png', 'webp'] }] });
      const paths = (Array.isArray(selected) ? selected : typeof selected === 'string' ? [selected] : []).slice(0, 20 - this.newGallery.length);
      if (!paths.length) return;
      this.busy = true; this.change.detectChanges();
      for (const item of await this.importToLibrary(paths)) {
        this.newGallery = [...this.newGallery, await this.toPending(item.id, item.title || item.original_name, false)];
      }
    } catch (error) { this.errorMessage = this.error(error); }
    finally { this.busy = false; }
    this.change.detectChanges();
  }
  removeNewGalleryPhoto(index: number): void { this.newGallery = this.newGallery.filter((_, i) => i !== index); }
  async editNewPhoto(cover: boolean, index = 0): Promise<void> {
    const photo = cover ? this.newCover : this.newGallery[index];
    if (!photo) return;
    await this.openPhotoEditor(photo.mediaId, photo.label, async () => {
      const refreshed = await this.toPending(photo.mediaId, photo.label, cover);
      if (cover) this.newCover = refreshed;
      else this.newGallery = this.newGallery.map(p => p.mediaId === photo.mediaId ? refreshed : p);
    });
  }
  openLibraryPicker(purpose: 'new-cover' | 'new-gallery' | 'existing-cover' | 'existing-gallery', artworkId?: string): void {
    this.libraryPicker = { purpose, artworkId };
  }
  async onLibraryPicked(item: MediaItem): Promise<void> {
    const picker = this.libraryPicker;
    if (!picker) return;
    this.libraryPicker = null;
    const label = item.title || item.original_name;
    try {
      if (picker.purpose === 'new-cover') {
        this.newCover = await this.toPending(item.id, label, true);
      } else if (picker.purpose === 'new-gallery') {
        if (this.newGallery.length < 20 && !this.newGallery.some(g => g.mediaId === item.id)) {
          this.newGallery = [...this.newGallery, await this.toPending(item.id, label, false)];
        }
      } else if (picker.artworkId) {
        const artworkId = picker.artworkId;
        if (picker.purpose === 'existing-cover') {
          if (await this.run(() => api.artworkSetCoverFromMedia(artworkId, item.id), this.tr('Copertina aggiornata'))) {
            delete this.thumbnails[artworkId]; await this.refresh();
          }
        } else if (await this.run(() => api.artworkAddGalleryFromMedia(artworkId, item.id, this.galleryCaption || null), this.tr('Fotografia aggiunta alla galleria'))) {
          this.galleryImages = await api.listArtworkGallery(artworkId);
          this.galleryCaption = '';
        }
      }
    } catch (error) { this.errorMessage = this.error(error); }
    this.change.detectChanges();
  }
  /** Edit a picture that is already stored (the artwork shows the new version straight away). */
  private async editStoredPicture(path: string | null, done: () => Promise<void>): Promise<void> {
    if (!path) return;
    try {
      const id = await api.mediaIdForPath(path);
      if (!id) { this.errorMessage = this.tr('Immagine non trovata nella libreria'); return; }
      await this.openPhotoEditor(id, this.fileName(path), done);
    } catch (error) { this.errorMessage = this.error(error); }
  }
  async editCoverPicture(): Promise<void> {
    const artwork = this.editingArtwork;
    if (!artwork) return;
    await this.editStoredPicture(artwork.image_path, async () => { delete this.thumbnails[artwork.id]; await this.refresh(); });
  }
  async editGalleryPicture(image: GalleryImage): Promise<void> {
    const artwork = this.galleryArtwork;
    if (!artwork) return;
    await this.editStoredPicture(image.image_path, async () => {
      this.galleryPreviews = {};
      this.galleryImages = await api.listArtworkGallery(artwork.id);
      await this.refresh();
    });
  }
  /** Pictures imported for a dialog that was not completed are moved to the trash (recoverable there). */
  private async finishPhotoSession(keep: Set<string>): Promise<void> {
    const leftovers = [...this.sessionImports].filter(id => !keep.has(id));
    this.sessionImports.clear();
    for (const id of leftovers) { try { await api.mediaTrash(id); } catch { /* in use or already gone */ } }
  }
  async cancelArtworkDialog(): Promise<void> {
    this.artworkDialog = false;
    await this.finishPhotoSession(new Set());
  }
  async pickRaw(): Promise<void> {
    try {
      const selected = await open({ directory: false, multiple: false, title: this.tr('Scegli file RAW…'),
        filters: [{ name: 'RAW', extensions: this.rawExtensions }] });
      if (typeof selected === 'string') this.newRawPath = selected;
    } catch (error) { this.errorMessage = this.error(error); }
    this.change.detectChanges();
  }
  async attachRaw(artworkId: string): Promise<void> {
    if (this.busy) return;
    try {
      const selected = await open({ directory: false, multiple: false, title: this.tr('Scegli file RAW…'),
        filters: [{ name: 'RAW', extensions: this.rawExtensions }] });
      if (typeof selected !== 'string') return;
      await this.run(() => api.attachArtworkRaw(artworkId, selected), this.tr('File RAW allegato'));
    } catch (error) { this.errorMessage = this.error(error); }
  }
  async removeRaw(artworkId: string): Promise<void> {
    if (!window.confirm(this.tr('Rimuovere il file RAW allegato? Il file dell’archivio verrà eliminato.'))) return;
    await this.run(() => api.removeArtworkRaw(artworkId), this.tr('File RAW rimosso'));
  }
  async exportRaw(artworkId: string): Promise<void> {
    const raw = this.rawByArtwork[artworkId];
    if (!raw || this.busy) return;
    try {
      const extension = raw.original_name.split('.').pop() ?? 'raw';
      const destination = await save({ title: this.tr('Salva copia del file RAW'), defaultPath: raw.original_name,
        filters: [{ name: extension.toUpperCase(), extensions: [extension] }] });
      if (typeof destination !== 'string') return;
      await this.run(() => api.exportArtworkRaw(artworkId, destination), this.tr('Copia del file RAW salvata'));
    } catch (error) { this.errorMessage = this.error(error); }
  }
  onWithSaleChange(): void {
    if (this.withSale && !this.newSaleDraft.collector_id) {
      this.newSaleDraft.collector_id = this.collectors.length ? '' : '__new__';
    }
  }
  recalculateNewSaleTax(): void {
    const amount = this.newSaleDraft.amount_eur;
    if (amount === null || !Number.isFinite(amount) || amount < 0) return;
    this.newSaleDraft.taxes_eur = Math.round(amount * this.defaultTaxPercent) / 100;
  }
  get codeExample(): string {
    const p = this.codePolicy;
    const number = '1'.padStart(Math.min(8, Math.max(2, Number(p.padding) || 4)), '0');
    const prefix = (p.prefix || 'ART').toUpperCase();
    return p.scope === 'YEARLY' ? [prefix, new Date().getFullYear(), number].join(p.separator) : [prefix, number].join(p.separator);
  }
  async saveCodePolicy(): Promise<void> {
    await this.run(async () => {
      this.codePolicy = await api.saveArchiveCodePolicy({ ...this.codePolicy, padding: Number(this.codePolicy.padding) });
    }, this.tr('Codice archivio salvato'));
  }
  editArtwork(a: Artwork): void {
    this.estimateCents=null;
    this.artworkDraft = {
      id: a.id, code: a.code, title: a.title, year: a.year, technique: a.technique,
      support: a.support, width_cm: a.width_cm, height_cm: a.height_cm,
      depth_cm: a.depth_cm, description: a.description, location: a.location, status: a.status
    };
    this.artworkDialog = true;
  }
  async saveArtwork(): Promise<void> {
    if (this.artworkDraft.id) {
      if (await this.run(() => api.saveArtwork(this.artworkDraft), this.tr('Opera salvata'))) this.artworkDialog = false;
      return;
    }
    let sale: NewArtworkInput['sale'] = null;
    if (this.withSale) {
      const draft = this.newSaleDraft;
      const amount = draft.amount_eur;
      const newCollector = draft.collector_id === '__new__';
      if (amount === null || !Number.isFinite(amount) || amount <= 0 || Math.round(amount * 100) > 100_000_000_000) {
        this.errorMessage = this.tr('Inserisci un importo maggiore di zero'); return;
      }
      if (!draft.collector_id || (newCollector && !(draft.new_collector_name ?? '').trim())) {
        this.errorMessage = this.tr('Scegli un collezionista o inserisci il nome del nuovo'); return;
      }
      const tax = Math.round(draft.taxes_eur * 100);
      const shipping = Math.round(draft.shipping_eur * 100);
      if (![tax, shipping].every(v => Number.isFinite(v) && v >= 0) || tax + shipping > Math.round(amount * 100)) {
        this.errorMessage = this.tr('Importo tasse (€)') + ' / ' + this.tr('Spese spedizione (€)'); return;
      }
      const payment = paymentToInput(this.newSalePayment, Math.round(amount * 100));
      if (!payment.ok) { this.errorMessage = this.tr(payment.error); return; }
      sale = {
        payment: payment.input,
        collector_id: newCollector ? null : draft.collector_id,
        new_collector_name: newCollector ? (draft.new_collector_name ?? '').trim() : null,
        amount_cents: Math.round(amount * 100), sold_at: draft.sold_at, notes: draft.notes,
        tax_cents: tax, shipping_cents: shipping
      };
    }
    const { id: _unused, ...record } = this.artworkDraft;
    const source = (photo: PendingPhoto): PhotoSource => ({ path: null, media_id: photo.mediaId, operations: [] });
    const input: NewArtworkInput = {
      artwork: { ...record, code: null, status: this.withSale ? 'sold' : record.status },
      cover: this.newCover ? source(this.newCover) : null, gallery: this.newGallery.map(source), raw_path: this.newRawPath, sale
    };
    const used = new Set([...(this.newCover ? [this.newCover.mediaId] : []), ...this.newGallery.map(p => p.mediaId)]);
    if (await this.run(() => api.createArtworkFull(input), this.tr('Opera salvata'))) { this.artworkDialog = false; await this.finishPhotoSession(used); }
  }
  async deleteArtwork(a: Artwork): Promise<void> {
    if (!window.confirm(`Eliminare definitivamente l'opera "${a.title}"? Questa operazione non è reversibile.`)) return;
    await this.run(() => api.deleteArtwork(a.id), 'Opera eliminata');
    delete this.thumbnails[a.id];
  }
  private async pickPhoto(title: string): Promise<string | null> {
    const selected = await open({
      directory: false, multiple: false, title,
      filters: [{ name: 'JPEG / PNG / WebP', extensions: ['jpg', 'jpeg', 'png', 'webp'] }]
    });
    return typeof selected === 'string' ? selected : null;
  }
  async chooseCoverPhoto(artwork: Artwork): Promise<void> {
    if (this.busy || !this.initialized) return;
    try {
      const path = await this.pickPhoto(this.tr('Foto'));
      if (!path) return;
      const [item] = await this.importToLibrary([path]);
      if (!item) return;
      if (!(await this.run(() => api.artworkSetCoverFromMedia(artwork.id, item.id), this.tr('Immagine principale salvata')))) return;
      this.sessionImports.delete(item.id);
      const reload = async () => { delete this.thumbnails[artwork.id]; await this.refresh(); };
      await reload();
      // The picture is in the library: it can be cropped and corrected right away.
      await this.openPhotoEditor(item.id, item.title || item.original_name, reload);
    } catch (error) {
      this.errorMessage = this.error(error);
    }
    this.change.detectChanges();
  }
  async openGallery(artwork: Artwork): Promise<void> {
    if (this.busy) return;
    this.galleryArtwork = artwork;
    this.galleryImages = [];
    this.galleryPreviews = {};
    this.galleryCaption = '';
    this.cancelGalleryCaptionEdit();
    try {
      this.galleryImages = await api.listArtworkGallery(artwork.id);
      if (this.galleryImages.length) await this.loadGalleryPreview(this.galleryImages[0]);
    } catch (error) {
      this.errorMessage = this.error(error);
    }
    this.change.detectChanges();
  }
  closeGallery(): void {
    this.galleryArtwork = null;
    this.galleryImages = [];
    this.galleryPreviews = {};
    this.galleryCaption = '';
    this.cancelGalleryCaptionEdit();
  }
  startGalleryCaptionEdit(image: GalleryImage): void {
    if (this.busy) return;
    this.editingGalleryCaptionId = image.id;
    this.editingGalleryCaption = image.caption ?? '';
  }
  cancelGalleryCaptionEdit(): void {
    this.editingGalleryCaptionId = null;
    this.editingGalleryCaption = '';
  }
  async saveGalleryCaption(image: GalleryImage): Promise<void> {
    if (this.editingGalleryCaptionId !== image.id || this.galleryArtwork?.id !== image.artwork_id) return;
    const caption = this.editingGalleryCaption;
    await this.run(async () => {
      const saved = await api.updateArtworkGalleryCaption(image.id, caption);
      if (this.galleryArtwork?.id === saved.artwork_id) {
        this.galleryImages = this.galleryImages.map(item => item.id === saved.id ? saved : item);
        this.cancelGalleryCaptionEdit();
      }
    }, this.tr('Didascalia aggiornata'));
  }
  async moveGalleryPhoto(image: GalleryImage, direction: 'up' | 'down'): Promise<void> {
    const artwork = this.galleryArtwork;
    if (!artwork || this.busy || image.artwork_id !== artwork.id) return;
    const moved = await this.run(() => api.moveArtworkGalleryImage(artwork.id, image.id, direction),
      this.tr('Ordine fotografie aggiornato'));
    if (moved && this.galleryArtwork?.id === artwork.id) {
      this.galleryImages = await api.listArtworkGallery(artwork.id);
      this.change.detectChanges();
    }
  }
  async loadGalleryPreview(image: GalleryImage): Promise<void> {
    if (this.galleryPreviews[image.id]) return;
    try {
      const data = await api.artworkGalleryImageData(image.id);
      if (this.galleryArtwork?.id === image.artwork_id) this.galleryPreviews[image.id] = data;
    } catch (error) {
      this.errorMessage = this.error(error);
    }
    this.change.detectChanges();
  }
  async addGalleryPhoto(): Promise<void> {
    const artwork = this.galleryArtwork;
    if (!artwork || this.busy || this.galleryImages.length >= 20) return;
    try {
      const path = await this.pickPhoto(this.tr('Aggiungi fotografia'));
      if (!path || this.galleryArtwork?.id !== artwork.id) return;
      const [item] = await this.importToLibrary([path]);
      if (!item) return;
      if (await this.run(() => api.artworkAddGalleryFromMedia(artwork.id, item.id, this.galleryCaption || null), this.tr('Fotografia aggiunta alla galleria'))) {
        this.sessionImports.delete(item.id);
        this.galleryImages = await api.listArtworkGallery(artwork.id);
        this.galleryCaption = '';
      }
    } catch (error) {
      this.errorMessage = this.error(error);
    }
    this.change.detectChanges();
  }
  async makeGalleryPrimary(image: GalleryImage): Promise<void> {
    const artwork = this.galleryArtwork;
    if (!artwork || image.artwork_id !== artwork.id || this.busy) return;
    if (!window.confirm(this.tr('Impostare questa foto come principale? La precedente, se presente, resterà in galleria. La didascalia della foto scelta non verrà trasferita.'))) return;
    const promoted = await this.run(async () => {
      await api.promoteArtworkGalleryImage(artwork.id, image.id);
      // Refresh must reload the cover instead of reusing the previous thumbnail.
      delete this.thumbnails[artwork.id];
    }, this.tr('Fotografia principale aggiornata'));
    if (!promoted || this.galleryArtwork?.id !== artwork.id) return;
    this.galleryArtwork = this.artworks.find(item => item.id === artwork.id) ?? artwork;
    this.galleryPreviews = {};
    this.cancelGalleryCaptionEdit();
    try {
      this.galleryImages = await api.listArtworkGallery(artwork.id);
      if (this.galleryImages.length) await this.loadGalleryPreview(this.galleryImages[0]);
    } catch (error) {
      this.errorMessage = this.error(error);
    }
    this.change.detectChanges();
  }
  async removeGalleryPhoto(image: GalleryImage): Promise<void> {
    if (!window.confirm(this.tr('Eliminare questa fotografia aggiuntiva?'))) return;
    const artwork = this.galleryArtwork;
    if (await this.run(() => api.deleteArtworkGalleryImage(image.id), this.tr('Fotografia eliminata')) && artwork) {
      delete this.galleryPreviews[image.id];
      if (this.editingGalleryCaptionId === image.id) this.cancelGalleryCaptionEdit();
      this.galleryImages = await api.listArtworkGallery(artwork.id);
      this.change.detectChanges();
    }
  }
  initials(c: Collector): string {
    return ((c.first_name.slice(0, 1) + (c.last_name ?? '').slice(0, 1)) || c.name.slice(0, 1)).toUpperCase();
  }
  get profileImage(): CollectorImage | null { return this.collectorImages.find(i => i.kind === 'PROFILE') ?? null; }
  get profilePreview(): string { const p = this.profileImage; return p ? (this.collectorPreviews[p.id] ?? '') : ''; }
  get otherCollectorImages(): CollectorImage[] { return this.collectorImages.filter(i => i.kind === 'PHOTO'); }
  newCollector(): void {
    this.collectorDraft = freshCollector();
    this.collectorImages = []; this.collectorPreviews = {};
    this.newCollectorProfile = null; this.newCollectorPhotos = [];
    this.collectorDialog = true;
  }
  editCollector(c: Collector): void {
    this.collectorDraft = {
      id: c.id, first_name: c.first_name, last_name: c.last_name, email: c.email, phone: c.phone,
      address: c.address, notes: c.notes, birth_date: c.birth_date, birth_time: c.birth_time,
      birth_place: c.birth_place, instagram: c.instagram, facebook: c.facebook, tiktok: c.tiktok,
      website: c.website, linkedin: c.linkedin
    };
    this.collectorImages = []; this.collectorPreviews = {};
    this.collectorDialog = true;
    void this.loadCollectorImages(c.id);
  }
  private async loadCollectorImages(collectorId: string): Promise<void> {
    try {
      const images = await api.listCollectorImages(collectorId);
      if (this.collectorDraft.id !== collectorId) return;
      this.collectorImages = images;
      await Promise.all(images.map(async image => {
        try { this.collectorPreviews[image.id] = await api.collectorImageData(image.id); }
        catch { /* shown without a preview */ }
      }));
    } catch (error) { this.errorMessage = this.error(error); }
    this.change.detectChanges();
  }
  async saveCollector(): Promise<void> {
    if (this.collectorDraft.id) {
      if (await this.run(() => api.saveCollector(this.collectorDraft), this.tr('Collezionista salvato'))) this.collectorDialog = false;
      return;
    }
    const { id: _unused, ...record } = this.collectorDraft;
    if (await this.run(() => api.createCollectorFull({
      collector: record, profile_path: this.newCollectorProfile, photo_paths: this.newCollectorPhotos
    }), this.tr('Collezionista salvato'))) this.collectorDialog = false;
  }
  async pickCollectorProfile(): Promise<void> {
    try {
      const path = await this.pickPhoto(this.tr('Scegli foto profilo…'));
      if (path) this.newCollectorProfile = path;
    } catch (error) { this.errorMessage = this.error(error); }
    this.change.detectChanges();
  }
  private async pickPhotos(title: string): Promise<string[]> {
    const selected = await open({ directory: false, multiple: true, title,
      filters: [{ name: 'JPEG / PNG / WebP', extensions: ['jpg', 'jpeg', 'png', 'webp'] }] });
    return Array.isArray(selected) ? selected : typeof selected === 'string' ? [selected] : [];
  }
  async pickCollectorPhotos(): Promise<void> {
    try {
      const paths = await this.pickPhotos(this.tr('Aggiungi fotografie…'));
      this.newCollectorPhotos = [...new Set([...this.newCollectorPhotos, ...paths])].slice(0, 20);
    } catch (error) { this.errorMessage = this.error(error); }
    this.change.detectChanges();
  }
  removeNewCollectorPhoto(index: number): void { this.newCollectorPhotos = this.newCollectorPhotos.filter((_, i) => i !== index); }
  async setCollectorProfile(): Promise<void> {
    const id = this.collectorDraft.id;
    if (!id || this.busy) return;
    try {
      const path = await this.pickPhoto(this.tr('Scegli foto profilo…'));
      if (!path) return;
      if (await this.run(() => api.addCollectorImageFromPath(id, path, 'PROFILE'), this.tr('Foto profilo salvata'))) {
        await this.loadCollectorImages(id);
      }
    } catch (error) { this.errorMessage = this.error(error); }
    this.change.detectChanges();
  }
  async addCollectorPhotos(): Promise<void> {
    const id = this.collectorDraft.id;
    if (!id || this.busy) return;
    try {
      const paths = (await this.pickPhotos(this.tr('Aggiungi fotografie…'))).slice(0, 20 - this.otherCollectorImages.length);
      if (!paths.length) return;
      if (await this.run(async () => { for (const path of paths) await api.addCollectorImageFromPath(id, path, 'PHOTO'); },
        this.tr('Fotografie aggiunte'))) {
        await this.loadCollectorImages(id);
      }
    } catch (error) { this.errorMessage = this.error(error); }
    this.change.detectChanges();
  }
  async deleteCollectorImage(image: CollectorImage): Promise<void> {
    if (!window.confirm(this.tr('Eliminare questa fotografia aggiuntiva?'))) return;
    const id = this.collectorDraft.id;
    if (await this.run(() => api.deleteCollectorImage(image.id), this.tr('Fotografia eliminata')) && id) {
      delete this.collectorPreviews[image.id];
      await this.loadCollectorImages(id);
    }
  }
  async deleteCollector(c: Collector): Promise<void> {
    if (!window.confirm(`Eliminare il contatto "${c.name}"?`)) return;
    await this.run(() => api.deleteCollector(c.id), 'Collezionista eliminato');
  }
  recalculateDefaultSaleTax(): void {
    const amount=this.saleDraft.amount_eur;
    if (amount===null || !Number.isFinite(amount) || amount<0) return;
    this.saleDraft.taxes_eur=Math.round(amount*this.defaultTaxPercent)/100;
  }
  newSale(): void {
    this.saleDraft = {
      artwork_id: this.sellableArtworks[0]?.id ?? '',
      collector_id: this.collectors[0]?.id ?? '',
      amount_eur: null, taxes_eur:0, shipping_eur:0, sold_at: today(), notes: null
    };
    this.salePayment = emptyPayment(today());
    this.saleDialog = true;
  }
  async saveSale(): Promise<void> {
    const amount = this.saleDraft.amount_eur;
    if (amount === null || !Number.isFinite(amount) || amount <= 0 || Math.round(amount * 100) > 100_000_000_000) {
      this.errorMessage = 'Inserisci un importo maggiore di zero'; return;
    }
    const tax_cents=Math.round(this.saleDraft.taxes_eur*100);
    const shipping_cents=Math.round(this.saleDraft.shipping_eur*100);
    if (![tax_cents,shipping_cents].every(v=>Number.isFinite(v)&&v>=0) || tax_cents+shipping_cents>Math.round(amount*100)) {
      this.errorMessage=this.tr('Importo tasse (€)')+' / '+this.tr('Spese spedizione (€)');return;
    }
    const payment = paymentToInput(this.salePayment, Math.round(amount * 100));
    if (!payment.ok) { this.errorMessage = this.tr(payment.error); return; }
    const input = {
      artwork_id: this.saleDraft.artwork_id, collector_id: this.saleDraft.collector_id,
      amount_cents: Math.round(amount * 100), tax_cents, shipping_cents,
      sold_at: this.saleDraft.sold_at, notes: this.saleDraft.notes, payment: payment.input
    };
    if (await this.run(() => api.recordSale(input), 'Vendita registrata e stato dell’opera aggiornato')) this.saleDialog = false;
  }
  async deleteSale(sale: Sale): Promise<void> {
    if (!window.confirm(`Eliminare la vendita di "${sale.artwork_title}"? L'opera tornerà disponibile.`)) return;
    await this.run(() => api.deleteSale(sale.id), 'Vendita eliminata');
  }
  get saleAmountCents(): number | null {
    const cents = toCents(this.saleDraft.amount_eur);
    return Number.isFinite(cents) && cents > 0 ? cents : null;
  }
  get newSaleAmountCents(): number | null {
    const cents = toCents(this.newSaleDraft.amount_eur);
    return Number.isFinite(cents) && cents > 0 ? cents : null;
  }
  nextInstallment(saleId: string): Installment | null {
    return (this.installmentsBySale[saleId] ?? []).find(item => item.status !== 'PAID') ?? null;
  }
  installmentStatusLabel(status: Installment['status']): string {
    return this.tr({ PAID: 'Pagata', PARTIAL: 'Parzialmente pagata', OVERDUE: 'Scaduta', DUE: 'Da incassare' }[status]);
  }
  startPlanEdit(sale: Sale): void {
    const existing = this.installmentsBySale[sale.id] ?? [];
    this.planDraft = { ...emptyPayment(today()), mode: 'INSTALLMENTS', count: Math.max(2, existing.length || 3),
      installments: existing.map(i => ({ due_date: i.due_date, amount_eur: i.amount_cents / 100, notes: i.notes })) };
    this.planEditing = true;
  }
  async savePlan(sale: Sale): Promise<void> {
    const plan = installmentRows(this.planDraft.installments, sale.amount_cents);
    if (!plan.ok) { this.errorMessage = this.tr(plan.error); return; }
    if (await this.run(() => api.saveSaleInstallments(sale.id, plan.items), this.tr('Piano rateale salvato'))) this.planEditing = false;
  }
  async removePlan(sale: Sale): Promise<void> {
    if (!window.confirm(this.tr('Rimuovere il piano rateale? Gli incassi registrati non vengono modificati.'))) return;
    await this.run(() => api.saveSaleInstallments(sale.id, []), this.tr('Piano rateale rimosso'));
  }
  balanceForSale(sale: Sale): SaleBalance {
    return this.balancesBySale[sale.id] ?? {
      sale_id:sale.id, sale_cents:sale.amount_cents, received_cents:0,
      outstanding_cents:sale.amount_cents
    };
  }
  saleReceipts(saleId: string): SaleReceipt[] {
    return this.receipts.filter(receipt=>receipt.sale_id===saleId);
  }
  openSaleReceipts(sale: Sale): void {
    this.planEditing = false;
    this.selectedReceiptSale=sale;
    this.receiptAttachments = {};
    this.receiptPreview = null;
    const remaining=this.balanceForSale(sale).outstanding_cents;
    this.receiptDraft={amount_eur:remaining>0?remaining/100:null,
      received_at:today(),method:'bank_transfer',reference:null,notes:null};
    void this.loadReceiptAttachments(sale.id);
  }
  async saveSaleReceipt(): Promise<void> {
    const sale=this.selectedReceiptSale;
    const value=this.receiptDraft.amount_eur;
    const cents=value===null?NaN:Math.round(value*100);
    if (!sale || !Number.isFinite(cents) || !Number.isSafeInteger(cents) ||
        cents<=0 || cents>this.balanceForSale(sale).outstanding_cents) {
      this.errorMessage=this.tr('Importo incasso non valido');return;
    }
    const input: SaleReceiptInput={sale_id:sale.id,amount_cents:cents,
      received_at:this.receiptDraft.received_at,method:this.receiptDraft.method,
      reference:this.receiptDraft.reference,notes:this.receiptDraft.notes};
    if (await this.run(()=>api.recordSaleReceipt(input),this.tr('Incasso registrato'))) {
      const remaining=this.balanceForSale(sale).outstanding_cents;
      this.receiptDraft.amount_eur=remaining>0?remaining/100:null;
    }
  }
  async deleteSaleReceipt(receipt: SaleReceipt): Promise<void> {
    if (!window.confirm(this.tr('Confermi eliminazione incasso?'))) return;
    await this.run(()=>api.deleteSaleReceipt(receipt.id),this.tr('Incasso eliminato'));
  }

  private async loadReceiptAttachments(saleId: string): Promise<void> {
    const version = ++this.receiptAttachmentLoadVersion;
    this.receiptAttachmentsLoading = true;
    try {
      const entries = await Promise.all(this.saleReceipts(saleId).map(async (receipt) =>
        [receipt.id, await api.getReceiptAttachment(receipt.id)] as const));
      if (version === this.receiptAttachmentLoadVersion && this.selectedReceiptSale?.id === saleId) {
        this.receiptAttachments = Object.fromEntries(entries);
      }
    } catch (error) {
      if (version === this.receiptAttachmentLoadVersion && this.selectedReceiptSale?.id === saleId) {
        this.errorMessage = this.error(error);
      }
    } finally {
      if (version === this.receiptAttachmentLoadVersion) {
        this.receiptAttachmentsLoading = false;
        this.change.detectChanges();
      }
    }
  }
  async attachReceiptFile(receipt: SaleReceipt): Promise<void> {
    if (this.busy || this.receiptAttachmentsLoading || !this.initialized) return;
    try {
      const selected = await open({
        directory: false, multiple: false, title: this.tr('Allega documento'),
        filters: [{ name: 'PDF / JPEG / PNG / WebP', extensions: ['pdf', 'jpg', 'jpeg', 'png', 'webp'] }]
      });
      if (typeof selected !== 'string' || this.selectedReceiptSale?.id !== receipt.sale_id) return;
      const success = await this.run(
        () => api.attachReceiptFile(receipt.id, selected), this.tr('Documento allegato'));
      if (success) await this.loadReceiptAttachments(receipt.sale_id);
    } catch (error) {
      this.errorMessage = this.error(error);
    }
    this.change.detectChanges();
  }
  async viewReceiptImage(receipt: SaleReceipt, file: ReceiptAttachment): Promise<void> {
    if (this.busy || this.receiptAttachmentsLoading || !file.mime_type.startsWith('image/')) return;
    this.busy = true;
    this.errorMessage = '';
    try {
      const data = await api.receiptAttachmentData(receipt.id);
      if (!data.startsWith(`data:${file.mime_type};base64,`)) {
        throw new Error(this.tr('Formato documento non valido'));
      }
      if (this.selectedReceiptSale?.id === receipt.sale_id) {
        this.receiptPreview = { name: file.original_name, src: data };
      }
    } catch (error) {
      this.errorMessage = this.error(error);
    } finally {
      this.busy = false;
      this.change.detectChanges();
    }
  }
  async exportReceiptFile(receipt: SaleReceipt, file: ReceiptAttachment): Promise<void> {
    if (this.busy || this.receiptAttachmentsLoading || !this.initialized) return;
    const ext = file.mime_type === 'application/pdf' ? 'pdf' :
      file.mime_type === 'image/jpeg' ? 'jpg' :
      file.mime_type === 'image/png' ? 'png' : 'webp';
    try {
      const destination = await save({
        title: this.tr('Esporta documento'),
        defaultPath: `ReperticaStudio-incasso-${receipt.id.slice(0, 8)}.${ext}`,
        filters: [{ name: 'PDF / JPEG / PNG / WebP', extensions: [ext] }]
      });
      if (typeof destination !== 'string') return;
      await this.run(
        () => api.exportReceiptAttachment(receipt.id, destination), this.tr('Documento esportato'));
    } catch (error) {
      this.errorMessage = this.error(error);
    }
    this.change.detectChanges();
  }
  async removeReceiptFile(receipt: SaleReceipt): Promise<void> {
    if (this.busy || this.receiptAttachmentsLoading ||
      !window.confirm(this.tr('Confermi rimozione dell’allegato?'))) return;
    const success = await this.run(
      () => api.removeReceiptAttachment(receipt.id), this.tr('Documento rimosso'));
    if (success) {
      this.receiptPreview = null;
      if (this.selectedReceiptSale?.id === receipt.sale_id) {
        await this.loadReceiptAttachments(receipt.sale_id);
      }
    }
  }
  newCertificate(): void {
    this.certificateDraft = {
      artwork_id: this.artworks[0]?.id ?? '', artist_name: '',
      collector_name: null, notes: null
    };
    this.certificateDialog = true;
  }
  async saveCertificate(): Promise<void> {
    const certificate = await this.captureCertificate();
    if (certificate) {
      this.certificateDialog = false;
      this.showCertificate(certificate);
    }
  }
  private async captureCertificate(): Promise<Certificate | null> {
    if (this.busy) return null;
    this.busy = true; this.errorMessage = '';
    try {
      const c = await api.issueCertificate(this.certificateDraft);
      await this.refresh();
      this.notice = `Certificato ${c.serial} emesso`;
      return c;
    } catch (error) {
      this.errorMessage = this.error(error);
      return null;
    } finally { this.busy = false; this.change.detectChanges(); }
  }
  showCertificate(c: Certificate): void {
    this.selectedCertificate = c;
    try { this.selectedSnapshot = JSON.parse(c.snapshot_json) as Artwork; }
    catch { this.selectedSnapshot = null; }
    this.change.detectChanges();
  }
  printCertificate(c: Certificate): void {
    this.showCertificate(c);
    window.setTimeout(() => window.print(), 150);
  }
}
