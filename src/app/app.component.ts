import { CommonModule } from '@angular/common';
import { ChangeDetectorRef, Component, OnInit } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { isTauri } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { api } from './api';
import { buildSaleIndexes, filterArtworks, filterCollectors } from './archive-filters';
import type { ArtworkOrder } from './archive-filters';
import type {
  Artwork, ArtworkInput, ArtworkStatus, Collector, CollectorInput, Sale,
  Certificate, CertificateInput, Overview, BackupSummary, RestoreSummary
} from './models';

type Section = 'overview' | 'artworks' | 'collectors' | 'sales' | 'certificates' | 'settings';
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
    { id: 'settings', label: 'Impostazioni', icon: '⚙' }
  ];
  readonly statusLabels = statusLabels;
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
  certificates: Certificate[] = [];
  overview: Overview = { artworks: 0, collectors: 0, sales: 0, certificates: 0, revenue_cents: 0 };
  dataPath = '';
  backupDestination = '';
  lastBackup: BackupSummary | null = null;
  restoreCandidate: BackupSummary | null = null;
  lastRestore: RestoreSummary | null = null;

  thumbnails: Record<string, string> = {};
  artworkDialog = false;
  collectorDialog = false;
  saleDialog = false;
  certificateDialog = false;
  artworkDraft: ArtworkInput = freshArtwork();
  collectorDraft: CollectorInput = freshCollector();
  saleDraft = { artwork_id: '', collector_id: '', amount_eur: null as number | null, sold_at: today(), notes: null as string | null };
  certificateDraft: CertificateInput = { artwork_id: '', artist_name: '', collector_name: null, notes: null };
  selectedCertificate: Certificate | null = null;
  selectedSnapshot: Artwork | null = null;

  constructor(private readonly change: ChangeDetectorRef) {}
  async ngOnInit(): Promise<void> {
    if (!isTauri()) {
      this.errorMessage = 'Apri Repertica Studio con npm run tauri:dev: il browser non può accedere al database locale.';
      this.loading = false;
      return;
    }
    this.initialized = true;
    await this.refresh();
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
    return this.sections.find(item => item.id === this.section)?.label ?? 'Repertica Studio';
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
    return new Intl.NumberFormat('it-IT', { style: 'currency', currency: 'EUR' }).format(cents / 100);
  }
  date(value: string): string {
    const d = new Date(value.length === 10 ? value + 'T12:00:00' : value);
    return Number.isNaN(d.getTime()) ? value : d.toLocaleDateString('it-IT');
  }
  dimensions(artwork: Artwork): string {
    const parts = [artwork.height_cm, artwork.width_cm, artwork.depth_cm].filter(v => v !== null);
    return parts.length ? parts.map(v => String(v)).join(' × ') + ' cm' : '—';
  }
  async refresh(): Promise<void> {
    this.loading = true;
    try {
      [this.artworks, this.collectors, this.sales, this.certificates, this.overview, this.dataPath] = await Promise.all([
        api.listArtworks(), api.listCollectors(), api.listSales(), api.listCertificates(),
        api.overview(), api.dataDirectory()
      ]);
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
  newArtwork(): void { this.artworkDraft = freshArtwork(); this.artworkDialog = true; }
  editArtwork(a: Artwork): void {
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
  newSale(): void {
    this.saleDraft = {
      artwork_id: this.sellableArtworks[0]?.id ?? '',
      collector_id: this.collectors[0]?.id ?? '',
      amount_eur: null, sold_at: today(), notes: null
    };
    this.saleDialog = true;
  }
  async saveSale(): Promise<void> {
    const amount = this.saleDraft.amount_eur;
    if (amount === null || !Number.isFinite(amount) || amount <= 0 || Math.round(amount * 100) > 100_000_000_000) {
      this.errorMessage = 'Inserisci un importo maggiore di zero'; return;
    }
    const input = {
      artwork_id: this.saleDraft.artwork_id, collector_id: this.saleDraft.collector_id,
      amount_cents: Math.round(amount * 100), sold_at: this.saleDraft.sold_at, notes: this.saleDraft.notes
    };
    if (await this.run(() => api.recordSale(input), 'Vendita registrata e stato dell’opera aggiornato')) this.saleDialog = false;
  }
  async deleteSale(sale: Sale): Promise<void> {
    if (!window.confirm(`Eliminare la vendita di "${sale.artwork_title}"? L'opera tornerà disponibile.`)) return;
    await this.run(() => api.deleteSale(sale.id), 'Vendita eliminata');
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
