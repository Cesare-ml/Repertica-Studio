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
import { BrochuresComponent } from './brochures.component';
import { AuctionsComponent } from './auctions.component';
import { BusinessMailComponent } from './business-mail.component';
import { ArtworkViewerComponent } from './artwork-viewer.component';
import { CertificateSheetComponent } from './certificate-sheet.component';
import { emptyPayment, installmentRows, paymentToInput, toCents, type PaymentDraft } from './payments';
import { buildSaleIndexes, filterArtworks, filterCollectors } from './archive-filters';
import type { ArtworkOrder, CollectorFilter } from './archive-filters';
import { PHOTO_KINDS } from './models';
import { MenuState, SUPPORTS, TECHNIQUES, menuStateFor, valueFor } from './artwork-options';
import { EXAMPLE_SIZES, examplePriceCents, priceFieldsFor, priceModeOf, priceOf } from './pricing';
import type { PriceMode } from './pricing';
import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/plugin-notification';
import { badgeText, notificationContent } from './mail-sync';
import { mailText } from './mail-i18n';
import { artworkDeletionMessage, bulkDeletionMessage, collectorDeletionMessage, hasDependents, mergeImpacts } from './deletion-impact';
import type { DeletionImpact } from './deletion-impact';
import { FIRST_UPDATE_CHECK_DELAY_MS, UPDATE_CHECK_INTERVAL_MS, isQuietStatus, progressPercent, updateMessage } from './updates';
import { getVersion } from '@tauri-apps/api/app';
import type {
  Artwork, ArtworkInput, ArtworkStatus, Collector, CollectorInput, Sale,
  Certificate, CertificateInput, Overview, BackupSummary, RestoreSummary,
  StudioSettings, Expense, ExpenseInput, FinanceOverview, GalleryImage,
  SaleReceipt, SaleReceiptInput, SaleReceiptMethod, SaleBalance, ReceiptYearOverview,
  MediaAudit, MediaIssue, ReceiptAttachment, McpSetup, McpClientStatus, CodePolicy, NewArtworkInput, PhotoKind, UpdateReply, MailSyncState, CollectorImage, AllocationPlan, AllocationPart, BackupScheduleStatus, Installment,
  EditOperation, PhotoSource, RawFile, MediaItem, PreviewData
} from './models';

/** A picture chosen for a new artwork, possibly edited before it is stored. */
interface PendingPhoto { mediaId: string; label: string; thumb: string; kind: PhotoKind; }
type SettingsTab = 'profile' | 'pricing' | 'archive' | 'backup' | 'export' | 'ai' | 'about';
interface SettingsTabInfo { id: SettingsTab; label: string; title: string; description: string; }
/** Categories of the Settings page, in the order they are listed. Texts are Italian interface strings. */
export const SETTINGS_TABS: readonly SettingsTabInfo[] = [
  { id: 'profile', label: 'Profilo', title: 'Profilo e preferenze', description: 'Il tuo nome sui certificati, la lingua dell’app e la guida di benvenuto.' },
  { id: 'pricing', label: 'Prezzi e vendite', title: 'Prezzi, vendite e tasse', description: 'Come si calcola il prezzo delle opere e come si ripartiscono gli incassi.' },
  { id: 'archive', label: 'Archivio', title: 'Archivio', description: 'Dove sono i dati, come si numerano le opere e il controllo delle fotografie.' },
  { id: 'backup', label: 'Backup', title: 'Backup e ripristino', description: 'Copie di sicurezza automatiche e manuali, e ripristino di un archivio.' },
  { id: 'export', label: 'Esportazioni', title: 'Esportazioni', description: 'Copie dei registri in formato CSV per Excel o LibreOffice.' },
  { id: 'ai', label: 'Assistenti IA', title: 'Assistenti IA locali', description: 'Collega un assistente IA installato sul tuo computer in sola lettura.' },
  { id: 'about', label: 'Aggiornamenti', title: 'Aggiornamenti e informazioni', description: 'Versione installata, aggiornamenti automatici e informazioni sul software.' },
];
type Section = 'overview' | 'artworks' | 'media' | 'collectors' | 'sales' | 'certificates' | 'auctions' | 'brochures' | 'mail' | 'finance' | 'settings';
const freshArtwork = (): ArtworkInput => ({
  code: null, title: '', year: null, technique: null, support: null,
  width_cm: null, height_cm: null, depth_cm: null, description: null,
  location: null, status: 'available', signed: false, final_varnish: false, framed: false,
  artist_coefficient_override: null, fixed_price_cents: null
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
  imports: [CommonModule, FormsModule, PaymentPlanComponent, MediaCenterComponent, MediaEditorComponent, BrochuresComponent, AuctionsComponent, BusinessMailComponent, ArtworkViewerComponent, CertificateSheetComponent],
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.css']
})
export class AppComponent implements OnInit {
  appVersion = '0.1.0';
  readonly settingsTabs = SETTINGS_TABS;
  settingsTab: SettingsTab = 'profile';
  /** Figures from the background mail check; the badge in the menu shows the unread total. */
  mailSync: MailSyncState | null = null;
  mailUnread = 0;
  update: UpdateReply | null = null;
  updateChecking = false;
  updateInstalling = false;
  updateProgress: number | null = null;
  updateBannerHidden = false;
  autoUpdate = true;
  private readonly autoUpdateKey = 'repertica-auto-update';
  private updateTimer: ReturnType<typeof setInterval> | null = null;
  readonly sections: { id: Section; label: string }[] = [
    { id: 'overview', label: 'Panoramica' },
    { id: 'artworks', label: 'Opere' },
    { id: 'media', label: 'Libreria immagini' },
    { id: 'collectors', label: 'Collezionisti' },
    { id: 'sales', label: 'Vendite' },
    { id: 'certificates', label: 'Certificati' },
    { id: 'auctions', label: 'Aste' },
    { id: 'brochures', label: 'Brochure' },
    { id: 'mail', label: 'Posta' },
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
  /** Records ticked for deleting several at once (only the ones currently listed count). */
  selectedArtworkIds = new Set<string>();
  selectedCollectorIds = new Set<string>();
  selectedCertificateIds = new Set<string>();
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
    savings_bp:3333,treasury_start_date:null,default_tax_bp:0,ui_language:null,artist_name:null,artist_alias:null,mail_notifications:true
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
  /** How the price of the artwork in the form is set: artist coefficient, own coefficient or fixed price. */
  priceMode: PriceMode = 'auto';
  readonly exampleSizes = EXAMPLE_SIZES;
  /** Profile asked on the first start (full name, stage name, coefficient). */
  profileDraft = { name: '', alias: '', coefficient: 1 };
  /** Index (in the filtered list) of the artwork shown in the full-screen viewer; null when closed. */
  viewerIndex: number | null = null;
  viewerArtworks: Artwork[] = [];
  readonly techniqueOptions = TECHNIQUES;
  readonly supportOptions = SUPPORTS;
  techniqueMenu: MenuState = { choice: '', other: '' };
  supportMenu: MenuState = { choice: '', other: '' };
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
  galleryKind: PhotoKind = 'OTHER';
  readonly photoKinds = PHOTO_KINDS;
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
  certificateDraft: CertificateInput = { artwork_id: '', collector_name: null, notes: null, language: 'it', place: null };
  certificatePreview = false;
  certificateImage: string | null = null;
  private readonly placeKey = 'repertica-certificate-place';
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
      // Mail: the native side checks the inboxes every minute; each check arrives here as an event.
      void listen<MailSyncState>('mail-status', event => this.onMailStatus(event.payload)).catch(() => undefined);
      void api.mailSyncState().then(state => this.onMailStatus({ ...state, fresh: [] })).catch(() => undefined);
      await this.refresh();
      void this.startUpdates();
    } catch (error) {
      this.errorMessage = this.error(error);
      this.loading = false;
    }
  }
  settingsTabInfo(id: SettingsTab): SettingsTabInfo { return SETTINGS_TABS.find(t => t.id === id) ?? SETTINGS_TABS[0]; }
  /** Open the Settings page on a given category. */
  goToSettings(tab: SettingsTab): void {
    this.settingsTab = tab;
    this.navigate('settings');
  }
  // ---------------------------------------------------------------- mail
  badge(count: number): string { return badgeText(count); }
  private onMailStatus(state: MailSyncState): void {
    this.mailSync = state;
    this.mailUnread = state.unread_total;
    void this.notifyMail(state);
    this.change.detectChanges();
  }
  /** System notification for mail that arrived since the previous check (if the user left it on). */
  private async notifyMail(state: MailSyncState): Promise<void> {
    if (!this.settings.mail_notifications || !state.fresh.length) return;
    const content = notificationContent(state.fresh, key => mailText(this.language, key));
    if (!content) return;
    try {
      let allowed = await isPermissionGranted();
      if (!allowed) allowed = (await requestPermission()) === 'granted';
      if (allowed) sendNotification(content);
    } catch { /* notifications are a convenience: the badge still shows the new mail */ }
  }
  async setMailNotifications(on: boolean): Promise<void> {
    try {
      this.settings = await api.saveSettings({ ...this.settings, mail_notifications: on });
      this.settingsDraft = { ...this.settingsDraft, mail_notifications: this.settings.mail_notifications };
      // Ask the system for permission right away, so the first notification is not lost.
      if (on && !(await isPermissionGranted())) await requestPermission();
    } catch (error) { this.errorMessage = this.error(error); }
    this.change.detectChanges();
  }
  // ---------------------------------------------------------------- updates
  private async startUpdates(): Promise<void> {
    try { this.appVersion = await getVersion(); } catch { /* keep the built-in value */ }
    try { this.autoUpdate = localStorage.getItem(this.autoUpdateKey) !== 'off'; } catch { /* default: on */ }
    this.change.detectChanges();
    if (!this.autoUpdate) return;
    setTimeout(() => void this.checkUpdates(false), FIRST_UPDATE_CHECK_DELAY_MS);
    this.updateTimer = setInterval(() => void this.checkUpdates(false), UPDATE_CHECK_INTERVAL_MS);
  }
  setAutoUpdate(on: boolean): void {
    this.autoUpdate = on;
    try { localStorage.setItem(this.autoUpdateKey, on ? 'on' : 'off'); } catch { /* the choice lasts until the app closes */ }
    if (this.updateTimer) { clearInterval(this.updateTimer); this.updateTimer = null; }
    if (on) {
      void this.checkUpdates(false);
      this.updateTimer = setInterval(() => void this.checkUpdates(false), UPDATE_CHECK_INTERVAL_MS);
    }
  }
  /** `manual` checks report every outcome; automatic ones stay silent unless something is available. */
  async checkUpdates(manual: boolean): Promise<void> {
    if (this.updateChecking || this.updateInstalling) return;
    this.updateChecking = true;
    if (manual) { this.update = null; this.updateBannerHidden = false; }
    this.change.detectChanges();
    try {
      const reply = await api.checkForUpdate();
      if (reply.status === 'available') { this.update = reply; this.updateBannerHidden = false; }
      else if (manual || !isQuietStatus(reply.status)) this.update = reply;
    } catch (error) {
      if (manual) this.errorMessage = this.error(error);
    } finally {
      this.updateChecking = false;
    }
    this.change.detectChanges();
  }
  async installUpdate(skipBackup = false): Promise<void> {
    const version = this.update?.version;
    if (!version || this.updateInstalling || this.busy) return;
    this.updateInstalling = true; this.updateProgress = null; this.errorMessage = '';
    this.change.detectChanges();
    try {
      const reply = await api.installUpdate(version, skipBackup, p => {
        this.updateProgress = progressPercent(p.downloaded, p.total);
        this.change.detectChanges();
      });
      // A successful install restarts the app, so anything returned here is a result to show.
      this.update = reply.version ? reply : { ...reply, version };
    } catch (error) {
      this.errorMessage = this.error(error);
    } finally {
      this.updateInstalling = false;
    }
    this.change.detectChanges();
  }
  updateText(): string { return this.update ? this.tr(updateMessage(this.update.status)) : ''; }
  /** Artwork whose auction draft the Aste section should open (set from the artwork row). */
  auctionArtworkId: string | null = null;
  startAuction(artwork: Artwork): void {
    this.auctionArtworkId = artwork.id;
    this.navigate('auctions');
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
  get pickedArtworks(): Artwork[] { return this.visibleArtworks.filter(a => this.selectedArtworkIds.has(a.id)); }
  get pickedCollectors(): Collector[] { return this.visibleCollectors.filter(c => this.selectedCollectorIds.has(c.id)); }
  get pickedCertificates(): Certificate[] { return this.certificates.filter(c => this.selectedCertificateIds.has(c.id)); }
  toggleId(picked: Set<string>, id: string): void { if (!picked.delete(id)) picked.add(id); }
  allPicked(items: { id: string }[], picked: Set<string>): boolean { return items.length > 0 && items.every(item => picked.has(item.id)); }
  toggleAll(items: { id: string }[], picked: Set<string>): void {
    if (this.allPicked(items, picked)) items.forEach(item => picked.delete(item.id));
    else items.forEach(item => picked.add(item.id));
  }
  get visibleCollectors(): Collector[] {
    return filterCollectors(this.collectors, this.search,
      { byCollector: this.salesByCollector, byArtwork: this.salesByArtwork }, this.collectorFilter, this.artworks);
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
      invalid_snapshot: 'Snapshot del certificato non valido',
      invalid_metadata: 'Metadati dell’allegato RAW non validi',
      hash_mismatch: 'Integrità RAW compromessa (SHA-256 differente)'
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
  openViewer(artwork: Artwork): void {
    // A snapshot: the list getter builds a new array on every change detection.
    const list = [...this.visibleArtworks];
    const index = list.findIndex(a => a.id === artwork.id);
    if (index >= 0) { this.viewerArtworks = list; this.viewerIndex = index; }
  }
  get buyersByArtwork(): Record<string, string> {
    return Object.fromEntries(Object.entries(this.salesByArtwork).map(([id, sale]) => [id, sale.collector_name]));
  }
  // ---------------------------------------------------------------- price
  get priceRules() { return { rateCm2Cents: this.settings.base_price_cm2_cents, artistCoefficient: this.settings.artist_coefficient }; }
  /** Price of the artwork in the form, recalculated as the measures, coefficient or fixed price change. */
  get draftPrice() {
    const d = this.artworkDraft;
    const fixed = this.priceMode === 'fixed' ? d.fixed_price_cents : null;
    const own = this.priceMode === 'coefficient' ? d.artist_coefficient_override : null;
    return priceOf(d.width_cm, d.height_cm, this.priceRules, own, fixed);
  }
  get fixedPriceEur(): number | null { return this.artworkDraft.fixed_price_cents === null ? null : this.artworkDraft.fixed_price_cents / 100; }
  setFixedPriceEur(value: number | null): void {
    this.artworkDraft.fixed_price_cents = value === null || value === undefined || !Number.isFinite(value) ? null : Math.round(value * 100);
  }
  setPriceMode(mode: PriceMode): void {
    const d = this.artworkDraft;
    if (mode === 'coefficient' && d.artist_coefficient_override === null) d.artist_coefficient_override = this.settings.artist_coefficient;
    // Starting a fixed price from the calculated one is usually what the artist wants to round off.
    if (mode === 'fixed' && d.fixed_price_cents === null) d.fixed_price_cents = this.draftPrice.cents !== null ? Math.round(this.draftPrice.cents / 100) * 100 : null;
    this.priceMode = mode;
  }
  /** The price fields to store for the chosen mode, or null (with a message shown) when the input is incomplete. */
  private priceFields() {
    const d = this.artworkDraft;
    const fields = priceFieldsFor(this.priceMode, d.artist_coefficient_override, d.fixed_price_cents);
    if ('error' in fields) { this.errorMessage = this.tr(fields.error); return null; }
    return fields;
  }
  /** Price of a stored artwork, with how it was obtained (for the archive and the viewer). */
  priceInfo(a: Artwork) { return priceOf(a.width_cm, a.height_cm, this.priceRules, a.artist_coefficient_override, a.fixed_price_cents); }
  priceLabel(a: Artwork): string {
    const info = this.priceInfo(a);
    return info.cents === null ? '—' : this.euros(info.cents);
  }
  priceNote(a: Artwork): string {
    if (a.fixed_price_cents !== null) return this.tr('Prezzo fisso');
    return a.artist_coefficient_override !== null ? this.tr('Coefficiente') + ' ' + a.artist_coefficient_override : '';
  }
  get pricesByArtwork(): Record<string, string> {
    return Object.fromEntries(this.viewerArtworks.map(a => [a.id, this.priceLabel(a) + (this.priceNote(a) ? ' · ' + this.priceNote(a) : '')]));
  }
  exampleCents(size: { width: number; height: number }, coefficient: number): number | null {
    return examplePriceCents(size, this.priceRules, coefficient);
  }
  /** Same examples for the economy settings, which edit the rate and the coefficient before saving. */
  exampleCentsDraft(size: { width: number; height: number }): number | null {
    return examplePriceCents(size, { rateCm2Cents: Math.round(this.rateEuro * 100), artistCoefficient: this.settingsDraft.artist_coefficient }, this.settingsDraft.artist_coefficient);
  }
  private syncMenus(): void {
    this.techniqueMenu = menuStateFor(TECHNIQUES, this.artworkDraft.technique);
    this.supportMenu = menuStateFor(SUPPORTS, this.artworkDraft.support);
  }
  setTechniqueChoice(choice: string): void {
    this.techniqueMenu = { choice, other: choice === 'Altro' ? this.techniqueMenu.other : '' };
    this.artworkDraft.technique = valueFor(this.techniqueMenu.choice, this.techniqueMenu.other);
  }
  setTechniqueOther(text: string): void {
    this.techniqueMenu = { ...this.techniqueMenu, other: text };
    this.artworkDraft.technique = valueFor(this.techniqueMenu.choice, text);
  }
  setSupportChoice(choice: string): void {
    this.supportMenu = { choice, other: choice === 'Altro' ? this.supportMenu.other : '' };
    this.artworkDraft.support = valueFor(this.supportMenu.choice, this.supportMenu.other);
  }
  setSupportOther(text: string): void {
    this.supportMenu = { ...this.supportMenu, other: text };
    this.artworkDraft.support = valueFor(this.supportMenu.choice, text);
  }
  private freshNewSale() {
    return { collector_id: '', new_collector_name: null as string | null, amount_eur: null as number | null,
      taxes_eur: 0, shipping_eur: 0, sold_at: today(), notes: null as string | null };
  }
  fileName(path: string): string { return path.split(/[\\/]/).pop() ?? path; }
  async newArtwork(): Promise<void> {
    this.artworkDraft = freshArtwork(); this.priceMode = 'auto'; this.syncMenus();
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
    return { mediaId: id, label, thumb, kind: 'OTHER' };
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
      const refreshed = { ...(await this.toPending(photo.mediaId, photo.label, cover)), kind: photo.kind };
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
        } else if (await this.run(() => api.artworkAddGalleryFromMedia(artworkId, item.id, this.galleryCaption || null, this.galleryKind), this.tr('Fotografia aggiunta alla galleria'))) {
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
    // The price of the artwork is the natural amount of its sale.
    const price = this.draftPrice.cents;
    if (this.withSale && this.newSaleDraft.amount_eur === null && price !== null && price > 0) {
      this.newSaleDraft.amount_eur = price / 100;
      this.recalculateNewSaleTax();
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
    this.artworkDraft = {
      id: a.id, code: a.code, title: a.title, year: a.year, technique: a.technique,
      support: a.support, width_cm: a.width_cm, height_cm: a.height_cm,
      depth_cm: a.depth_cm, description: a.description, location: a.location, status: a.status,
      signed: a.signed, final_varnish: a.final_varnish, framed: a.framed,
      artist_coefficient_override: a.artist_coefficient_override, fixed_price_cents: a.fixed_price_cents
    };
    this.syncMenus();
    this.priceMode = priceModeOf(a.artist_coefficient_override, a.fixed_price_cents);
    this.artworkDialog = true;
  }
  async saveArtwork(): Promise<void> {
    const priced = this.priceFields();
    if (!priced) return;
    this.artworkDraft = { ...this.artworkDraft, ...priced };
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
    const source = (photo: PendingPhoto): PhotoSource => ({ path: null, media_id: photo.mediaId, operations: [], kind: photo.kind });
    const input: NewArtworkInput = {
      artwork: { ...record, code: null, status: this.withSale ? 'sold' : record.status },
      cover: this.newCover ? source(this.newCover) : null, gallery: this.newGallery.map(source), raw_path: this.newRawPath, sale
    };
    const used = new Set([...(this.newCover ? [this.newCover.mediaId] : []), ...this.newGallery.map(p => p.mediaId)]);
    if (await this.run(() => api.createArtworkFull(input), this.tr('Opera salvata'))) { this.artworkDialog = false; await this.finishPhotoSession(used); }
  }
  async deleteArtwork(a: Artwork): Promise<void> {
    // Certificates, the sale and its receipts depend on the artwork: show them and ask before deleting them too.
    let impact;
    try { impact = await api.artworkDeletionImpact(a.id); }
    catch (error) { this.errorMessage = this.error(error); return; }
    if (!window.confirm(artworkDeletionMessage(a.title, impact, this.formatters()))) return;
    if (await this.run(() => api.deleteArtwork(a.id, hasDependents(impact)), 'Opera eliminata')) delete this.thumbnails[a.id];
  }
  /** One confirmation for the whole selection, then the records are deleted one after the other. */
  private async deleteMany(steps: (() => Promise<void>)[], success: string): Promise<boolean> {
    return this.run(async () => {
      let done = 0;
      try { for (const step of steps) { await step(); done++; } }
      catch (error) {
        await this.refresh();
        throw new Error(`${done}/${steps.length} ${this.tr('eliminati')}: ${this.error(error)}`);
      }
    }, success);
  }
  async deleteSelectedArtworks(): Promise<void> {
    const items = this.pickedArtworks;
    if (!items.length) return;
    let impacts: DeletionImpact[];
    try { impacts = await Promise.all(items.map(a => api.artworkDeletionImpact(a.id))); }
    catch (error) { this.errorMessage = this.error(error); return; }
    if (!window.confirm(bulkDeletionMessage('artworks', items.map(a => a.title), mergeImpacts(impacts), this.formatters()))) return;
    if (await this.deleteMany(items.map((a, i) => () => api.deleteArtwork(a.id, hasDependents(impacts[i]))), 'Opere eliminate')) {
      items.forEach(a => { delete this.thumbnails[a.id]; this.selectedArtworkIds.delete(a.id); });
    }
  }
  async deleteSelectedCollectors(): Promise<void> {
    const items = this.pickedCollectors;
    if (!items.length) return;
    let impacts: DeletionImpact[];
    try { impacts = await Promise.all(items.map(c => api.collectorDeletionImpact(c.id))); }
    catch (error) { this.errorMessage = this.error(error); return; }
    if (!window.confirm(bulkDeletionMessage('collectors', items.map(c => c.name), mergeImpacts(impacts), this.formatters()))) return;
    if (await this.deleteMany(items.map((c, i) => () => api.deleteCollector(c.id, impacts[i].sales.length > 0)), 'Contatti eliminati')) {
      items.forEach(c => this.selectedCollectorIds.delete(c.id));
    }
  }
  async deleteSelectedCertificates(): Promise<void> {
    const items = this.pickedCertificates;
    if (!items.length) return;
    const none: DeletionImpact = { certificates: [], sales: [], receipts: 0, attachments: 0 };
    if (!window.confirm(bulkDeletionMessage('certificates', items.map(c => `${c.serial} (${c.artwork_title})`), none, this.formatters()))) return;
    if (await this.deleteMany(items.map(c => () => api.deleteCertificate(c.id)), 'Certificati eliminati')) {
      items.forEach(c => { this.selectedCertificateIds.delete(c.id); if (this.selectedCertificate?.id === c.id) this.selectedCertificate = null; });
    }
  }
  private formatters() {
    return { t: (key: string) => this.tr(key), money: (cents: number) => this.euros(cents), date: (value: string) => this.date(value) };
  }
  async deleteCertificate(c: Certificate): Promise<void> {
    if (!window.confirm(`${this.tr('Eliminare il certificato')} ${c.serial} (${c.artwork_title})?\n\n${this.tr('L’opera e la sua vendita non vengono modificate. Questa operazione non è reversibile.')}`)) return;
    if (await this.run(() => api.deleteCertificate(c.id), 'Certificato eliminato') && this.selectedCertificate?.id === c.id) this.selectedCertificate = null;
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
    this.galleryKind = 'OTHER';
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
    this.galleryKind = 'OTHER';
    this.cancelGalleryCaptionEdit();
  }
  setNewPhotoKind(index: number, kind: PhotoKind): void {
    this.newGallery = this.newGallery.map((p, i) => i === index ? { ...p, kind } : p);
  }
  async setGalleryKind(image: GalleryImage, kind: PhotoKind): Promise<void> {
    if (this.busy || image.kind === kind) return;
    await this.run(async () => {
      const saved = await api.updateArtworkGalleryKind(image.id, kind);
      this.galleryImages = this.galleryImages.map(i => i.id === image.id ? saved : i);
    }, this.tr('Tipo di fotografia aggiornato'));
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
      if (await this.run(() => api.artworkAddGalleryFromMedia(artwork.id, item.id, this.galleryCaption || null, this.galleryKind), this.tr('Fotografia aggiunta alla galleria'))) {
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
    let impact;
    try { impact = await api.collectorDeletionImpact(c.id); }
    catch (error) { this.errorMessage = this.error(error); return; }
    if (!window.confirm(collectorDeletionMessage(c.name, impact, this.formatters()))) return;
    await this.run(() => api.deleteCollector(c.id, impact.sales.length > 0), 'Collezionista eliminato');
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
    let place: string | null = null;
    try { place = localStorage.getItem(this.placeKey); } catch { /* optional convenience */ }
    this.certificateDraft = {
      artwork_id: this.artworks[0]?.id ?? '', collector_name: null, notes: null, language: 'it', place
    };
    this.certificateDialog = true;
  }
  get artistName(): string { return (this.settings.artist_name ?? '').trim(); }
  /** Name shown on certificates and brochures: the stage name when there is one. */
  get displayArtistName(): string { return (this.settings.artist_alias ?? '').trim() || this.artistName; }
  /** Save the artist's names (and, from the first-start card, the coefficient). */
  async saveArtistProfile(name: string, alias: string, coefficient?: number): Promise<void> {
    const fullName = name.trim();
    if (this.busy || !fullName) return;
    if (coefficient !== undefined && !(Number.isFinite(coefficient) && coefficient > 0 && coefficient <= 100_000)) {
      this.errorMessage = this.tr('Il coefficiente deve essere un numero maggiore di zero'); return;
    }
    const draft: StudioSettings = {
      ...this.settings, artist_name: fullName, artist_alias: alias.trim() || null,
      artist_coefficient: coefficient ?? this.settings.artist_coefficient
    };
    if (await this.run(async () => {
      this.settings = await api.saveSettings(draft);
      this.settingsDraft = { ...this.settingsDraft, artist_name: this.settings.artist_name, artist_alias: this.settings.artist_alias, artist_coefficient: this.settings.artist_coefficient };
    }, this.tr('Profilo dell’artista salvato'))) {
      this.profileDraft = { name: '', alias: '', coefficient: this.settings.artist_coefficient };
    }
  }
  async saveCertificate(): Promise<void> {
    const certificate = await this.captureCertificate();
    if (certificate) {
      try { if (this.certificateDraft.place) localStorage.setItem(this.placeKey, this.certificateDraft.place); } catch { /* optional */ }
      this.certificateDialog = false;
      await this.previewCertificate(certificate);
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
  /** Frozen picture of the certificate: the exact file recorded in the snapshot, even if the cover changed since. */
  private async certificatePicture(path: string | null | undefined): Promise<string | null> {
    if (!path) return null;
    try {
      const id = await api.mediaIdForPath(path);
      if (!id) return null;
      const revision = (await api.mediaDetail(id)).revisions.find(r => r.image_path === path);
      return (await api.mediaPreview(id, 1600, null, revision?.id ?? null)).data_url;
    } catch { return null; }
  }
  async showCertificate(c: Certificate): Promise<void> {
    this.selectedCertificate = c;
    try { this.selectedSnapshot = JSON.parse(c.snapshot_json) as Artwork; }
    catch { this.selectedSnapshot = null; }
    this.certificateImage = null;
    this.change.detectChanges();
    this.certificateImage = await this.certificatePicture(this.selectedSnapshot?.image_path);
    this.change.detectChanges();
  }
  async previewCertificate(c: Certificate): Promise<void> {
    await this.showCertificate(c);
    this.certificatePreview = true;
    this.change.detectChanges();
  }
  closeCertificatePreview(): void { this.certificatePreview = false; }
  async printCertificate(c: Certificate): Promise<void> {
    if (this.selectedCertificate?.id !== c.id || !this.selectedSnapshot) await this.showCertificate(c);
    window.setTimeout(() => window.print(), 150);
  }
}
