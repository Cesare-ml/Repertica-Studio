import { CommonModule } from '@angular/common';
import { ChangeDetectorRef, Component, OnInit } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { isTauri } from '@tauri-apps/api/core';
import { locale as osLocale } from '@tauri-apps/plugin-os';
import { AppLanguage, chooseLanguage, translate } from './i18n';
import { open, save } from '@tauri-apps/plugin-dialog';
import { api } from './api';
import { csvRegisterOptions, type CsvRegisterKind } from './csv-registers';
import { buildSaleIndexes, filterArtworks, filterCollectors } from './archive-filters';
import type { ArtworkOrder } from './archive-filters';
import type {
  Artwork, ArtworkInput, ArtworkStatus, Collector, CollectorInput, Sale,
  Certificate, CertificateInput, Overview, BackupSummary, RestoreSummary,
  StudioSettings, Expense, ExpenseInput, FinanceOverview, GalleryImage,
  SaleReceipt, SaleReceiptInput, SaleReceiptMethod, SaleBalance
} from './models';

type Section = 'overview' | 'artworks' | 'collectors' | 'sales' | 'certificates' | 'finance' | 'settings';
const freshArtwork = (): ArtworkInput => ({
  code: '', title: '', year: null, technique: null, support: null,
  width_cm: null, height_cm: null, depth_cm: null, description: null,
  location: null, status: 'available'
});
const freshCollector = (): CollectorInput => ({
  name: '', email: null, phone: null, address: null, notes: null
});
const today = () => new Date().toLocaleDateString('sv-SE');
const statusLabels: Record<ArtworkStatus, string> = {
  available: 'Disponibile', reserved: 'Riservata', sold: 'Venduta', archived: 'In archivio'
};

@Component({
  selector: 'rp-root',
  standalone: true,
  imports: [CommonModule, FormsModule],
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.css']
})
export class AppComponent implements OnInit {
  readonly appVersion = '0.1.0';
  readonly sections: { id: Section; label: string; icon: string }[] = [
    { id: 'overview', label: 'Panoramica', icon: '◫' },
    { id: 'artworks', label: 'Opere', icon: '▣' },
    { id: 'collectors', label: 'Collezionisti', icon: '♧' },
    { id: 'sales', label: 'Vendite', icon: '◇' },
    { id: 'certificates', label: 'Certificati', icon: '▤' },
    { id: 'finance', label: 'Economia', icon: '▥' },
    { id: 'settings', label: 'Impostazioni', icon: '⚙' }
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
  receiptDraft = {
    amount_eur: null as number | null, received_at: today(),
    method: 'bank_transfer' as SaleReceiptMethod,
    reference: null as string | null, notes: null as string | null
  };
  expenses: Expense[] = [];
  finances: FinanceOverview | null = null;
  financeYear = new Date().getFullYear();
  financeYears = Array.from({length: 8}, (_,i)=>new Date().getFullYear()-i);
  settings: StudioSettings = {
    artist_coefficient:1,base_price_cm2_cents:16,personal_bp:3334,activity_bp:3333,
    savings_bp:3333,treasury_start_date:null,default_tax_bp:0,ui_language:null
  };
  settingsDraft: StudioSettings = {...this.settings};
  rateEuro = .16;
  personalPercent = 33.34;
  activityPercent = 33.33;
  savingsPercent = 33.33;
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
    fund:'activity' as 'activity'|'savings',supplier:null as string|null,
    purchase_url:null as string|null,notes:null as string|null
  };

  certificates: Certificate[] = [];
  overview: Overview = { artworks: 0, collectors: 0, sales: 0, certificates: 0, revenue_cents: 0 };
  dataPath = '';
  backupDestination = '';
  lastBackup: BackupSummary | null = null;
  restoreCandidate: BackupSummary | null = null;
  lastRestore: RestoreSummary | null = null;

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
  collectorDraft: CollectorInput = freshCollector();
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
      this.personalPercent = stored.personal_bp / 100;
      this.activityPercent = stored.activity_bp / 100;
      this.savingsPercent = stored.savings_bp / 100;
      this.defaultTaxPercent = stored.default_tax_bp / 100;
      this.languageChoice = stored.ui_language ?? 'system';
      this.applyLanguage(chooseLanguage(stored.ui_language ?? system ?? navigator.language));
      this.initialized = true;
      await this.refresh();
    } catch (error) {
      this.errorMessage = this.error(error);
      this.loading = false;
    }
  }
  navigate(section: Section): void {
    this.section = section;
    this.search = '';
    this.errorMessage = '';
    this.notice = '';
    if (section === 'artworks') {
      this.artworkStatusFilter = 'all';
      this.artworkCollectorFilter = 'all';
      this.artworkOrder = 'recent';
    }
    if (section === 'collectors') this.expandedCollectorId = null;
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
      { byCollector: this.salesByCollector, byArtwork: this.salesByArtwork });
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
        receipts, balances] = await Promise.all([
        api.listArtworks(), api.listCollectors(), api.listSales(), api.listCertificates(),
        api.overview(), api.dataDirectory(), api.listExpenses(), api.financeOverview(this.financeYear),
        api.listSaleReceipts(), api.listSaleBalances()
      ]);
      this.artworks=artworks; this.collectors=collectors; this.sales=sales; this.certificates=certificates;
      this.overview=overview; this.dataPath=dataPath; this.expenses=expenses; this.finances=finances;
      this.receipts=receipts;
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
      this.finances = await api.financeOverview(this.financeYear);
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
    const p=[this.personalPercent,this.activityPercent,this.savingsPercent].map(v=>Math.round(v*100));
    if (p.some((v)=>!Number.isFinite(v)||v<0||v>10000) || p.reduce((a,b)=>a+b,0)!==10000) {
      this.errorMessage = this.tr('Deve essere 100%'); return;
    }
    const base=Math.round(this.rateEuro*100);
    const tax=Math.round(this.defaultTaxPercent*100);
    const draft: StudioSettings = {...this.settingsDraft,
      base_price_cm2_cents:base,default_tax_bp:tax,
      personal_bp:p[0],activity_bp:p[1],savings_bp:p[2],
      ui_language:this.languageChoice==='system'?null:this.languageChoice as AppLanguage};
    if (await this.run(async()=>{this.settings=await api.saveSettings(draft);this.settingsDraft={...this.settings};},
      this.tr('Salva impostazioni'))) {
      this.applyLanguage(this.languageChoice==='system'?this.systemLanguage:chooseLanguage(this.languageChoice));
      await this.loadFinanceYear();
    }
  }
  get allocationValid(): boolean {
    return [this.personalPercent,this.activityPercent,this.savingsPercent].every(v=>Number.isFinite(v)&&v>=0&&v<=100)
      && [this.personalPercent,this.activityPercent,this.savingsPercent].reduce((a,b)=>a+Math.round(b*100),0)===10000;
  }
  selectLanguage(): void {
    this.applyLanguage(this.languageChoice === 'system' ? this.systemLanguage : chooseLanguage(this.languageChoice));
  }
  newExpense(): void {
    this.expenseDraft={id:undefined,expense_date:today(),category:'painting-materials',description:'',amount_eur:null,
      fund:'activity',supplier:null,purchase_url:null,notes:null};
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
  newArtwork(): void { this.estimateCents=null; this.artworkDraft = freshArtwork(); this.artworkDialog = true; }
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
    if (await this.run(() => api.saveArtwork(this.artworkDraft), 'Opera salvata nell’archivio locale')) this.artworkDialog = false;
  }
  async deleteArtwork(a: Artwork): Promise<void> {
    if (!window.confirm(`Eliminare definitivamente l'opera "${a.title}"? Questa operazione non è reversibile.`)) return;
    await this.run(() => api.deleteArtwork(a.id), 'Opera eliminata');
    delete this.thumbnails[a.id];
  }
  async onImage(event: Event, artwork: Artwork): Promise<void> {
    const input = event.target as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    if (file.size > 12 * 1024 * 1024) {
      this.errorMessage = 'Immagine troppo grande: massimo 12 MiB'; input.value = ''; return;
    }
    const base64 = await new Promise<string>((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = () => resolve(String(reader.result).split(',')[1] ?? '');
      reader.onerror = () => reject(new Error('Impossibile leggere il file locale'));
      reader.readAsDataURL(file);
    }).catch(error => { this.errorMessage = this.error(error); return ''; });
    if (!base64) { input.value = ''; return; }
    if (await this.run(() => api.saveArtworkImage(artwork.id, base64), 'Immagine principale salvata')) {
      delete this.thumbnails[artwork.id];
      this.thumbnails[artwork.id] = (await api.artworkImage(artwork.id)) ?? '';
    }
    input.value = '';
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
  async addGalleryPhoto(event: Event): Promise<void> {
    const input = event.target as HTMLInputElement;
    const file = input.files?.[0];
    const artwork = this.galleryArtwork;
    if (!file || !artwork) return;
    try {
      if (file.size > 12 * 1024 * 1024) throw new Error('Immagine troppo grande: massimo 12 MiB');
      const dataBase64 = await new Promise<string>((resolve, reject) => {
        const reader = new FileReader();
        reader.onload = () => resolve(String(reader.result).split(',')[1] ?? '');
        reader.onerror = () => reject(new Error('Impossibile leggere il file locale'));
        reader.readAsDataURL(file);
      });
      if (await this.run(
        () => api.addArtworkGalleryImage(artwork.id, dataBase64, this.galleryCaption),
        this.tr('Fotografia aggiunta alla galleria')
      )) {
        this.galleryImages = await api.listArtworkGallery(artwork.id);
        this.galleryCaption = '';
      }
    } catch (error) {
      this.errorMessage = this.error(error);
    } finally {
      input.value = '';
      this.change.detectChanges();
    }
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
  newCollector(): void { this.collectorDraft = freshCollector(); this.collectorDialog = true; }
  editCollector(c: Collector): void {
    this.collectorDraft = { id: c.id, name: c.name, email: c.email, phone: c.phone, address: c.address, notes: c.notes };
    this.collectorDialog = true;
  }
  async saveCollector(): Promise<void> {
    if (await this.run(() => api.saveCollector(this.collectorDraft), 'Collezionista salvato')) this.collectorDialog = false;
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
    const input = {
      artwork_id: this.saleDraft.artwork_id, collector_id: this.saleDraft.collector_id,
      amount_cents: Math.round(amount * 100), tax_cents, shipping_cents,
      sold_at: this.saleDraft.sold_at, notes: this.saleDraft.notes
    };
    if (await this.run(() => api.recordSale(input), 'Vendita registrata e stato dell’opera aggiornato')) this.saleDialog = false;
  }
  async deleteSale(sale: Sale): Promise<void> {
    if (!window.confirm(`Eliminare la vendita di "${sale.artwork_title}"? L'opera tornerà disponibile.`)) return;
    await this.run(() => api.deleteSale(sale.id), 'Vendita eliminata');
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
    this.selectedReceiptSale=sale;
    const remaining=this.balanceForSale(sale).outstanding_cents;
    this.receiptDraft={amount_eur:remaining>0?remaining/100:null,
      received_at:today(),method:'bank_transfer',reference:null,notes:null};
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
