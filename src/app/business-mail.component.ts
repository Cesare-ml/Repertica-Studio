import { CommonModule } from '@angular/common';
import { Component, EventEmitter, Input, OnChanges, OnDestroy, OnInit, Output, SimpleChanges } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { open } from '@tauri-apps/plugin-dialog';
import { api } from './api';
import type { AppLanguage } from './i18n';
import type { Collector, MailAccount, MailAccountInput, MailDetail, MailDraft, MailDraftInput, MailFolder, MailHeader, MailList, MailOverview, MailSyncState } from './models';
import { MailIconComponent } from './mail-icons';
import { mailText } from './mail-i18n';
import { badgeText, filterMessages, folderLabel, inboxStamp, messageDate, senderName, splitFolders, splitInbox } from './mail-sync';

type MailView = 'mail' | 'drafts' | 'settings';
type ComposeMode = 'normal' | 'minimized' | 'expanded';
type Composer = { id: string | null; revision: number | null; to: string; cc: string; bcc: string; subject: string; body: string; attachments: string[] };
const blankComposer = (): Composer => ({ id:null,revision:null,to:'',cc:'',bcc:'',subject:'',body:'',attachments:[] });
const freshAccount = (): MailAccountInput => ({
  id:null,email:'',display_name:'',imap_host:'',imap_port:993,imap_username:'',
  smtp_host:'',smtp_port:465,smtp_username:'',sent_folder:'',signature:'',
  imap_password:'',smtp_password:''
});
/**
 * What was last loaded for each mailbox (headers only, in memory, never written to disk): the section shows it
 * at once when it is opened again, and refreshes it in the background.
 */
interface MailMemory { folders: MailFolder[]; lists: Map<string, MailList>; }
const mailMemory = new Map<string, MailMemory>();
const remember = (account: string): MailMemory => {
  let entry = mailMemory.get(account);
  if (!entry) { entry = { folders: [], lists: new Map() }; mailMemory.set(account, entry); }
  return entry;
};
const errText = (err: unknown) => typeof err === 'string' ? err : err instanceof Error ? err.message : 'Operation failed';
/** The open folder is refreshed in the background this often (the inbox is also refreshed when new mail is reported). */
const REFRESH_MS = 60_000;

/**
 * A personal mail client, laid out like the Gestionale Opere's: folders on the left, messages split into
 * "still to read" and the rest, a reader, and a floating compose window.
 * Everything is synchronised automatically: the list refreshes by itself, opening a message marks it as read,
 * and the unread counter in the menu and the system notifications come from the background watcher.
 * No HTML mail or remote resources are rendered, credentials are never kept in Angular state after the
 * account is saved, and outgoing messages always require explicit confirmation.
 */
@Component({
  selector:'rp-business-mail', standalone:true,
  imports:[CommonModule,FormsModule,MailIconComponent],
  templateUrl:'./business-mail.component.html',
  styleUrls:['./business-mail.component.css']
})
export class BusinessMailComponent implements OnInit, OnChanges, OnDestroy {
  @Input() language: AppLanguage = 'en';
  @Input() collectors: Collector[] = [];
  /** Figures from the background watcher (unread counters, new mail). */
  @Input() sync: MailSyncState | null = null;
  /** Show a system notification when new mail arrives (setting saved by the application). */
  @Input() notifications = true;
  @Output() notificationsChange = new EventEmitter<boolean>();

  accounts: MailAccount[] = [];
  selectedId = '';
  folders: MailFolder[] = [];
  selectedFolder = 'INBOX';
  listing: MailList | null = null;
  message: MailDetail | null = null;
  /** The body of the open message is being fetched (its header is already shown). */
  messageLoading = false;
  drafts: MailDraft[] = [];
  composer: Composer = blankComposer();
  composeOpen = false;
  composeMode: ComposeMode = 'normal';
  showCc = false;
  showBcc = false;
  private originalComposer = '';
  accountForm: MailAccountInput | null = null;
  view: MailView = 'mail';
  query = '';
  collapsed = { unread: false, rest: false };
  otherFoldersOpen = false;
  /** An operation started by the user (sending, saving…) is running. */
  busy = false;
  /** The first load of the folders and messages is running. */
  loading = false;
  /** A refresh is running in the background (the list on screen is the previous one). */
  syncing = false;
  notice = '';
  error = '';
  lastSyncAt: Date | null = null;
  syncFailed = false;
  private loadVersion = 0;
  private timer: ReturnType<typeof setInterval> | null = null;
  private lastStamp = '';
  private refreshing = false;

  get selected():MailAccount | undefined { return this.accounts.find(x=>x.id===this.selectedId); }
  get hasUnsavedComposer():boolean {
    return this.composeOpen && JSON.stringify(this.composer)!==this.originalComposer;
  }
  t(key:string):string { return mailText(this.language,key); }
  get fromCollectors():{name:string;email:string}[] {
    return this.collectors.filter(c=>Boolean(c.email)).map(c=>({name:c.name,email:c.email!})).slice(0,300);
  }
  // ---- folders and lists as shown
  get folderGroups() { return splitFolders(this.folders); }
  get currentFolder(): MailFolder | undefined { return this.folders.find(f=>f.name===this.selectedFolder); }
  get isInbox(): boolean { return (this.currentFolder?.role ?? (this.selectedFolder==='INBOX'?'inbox':'')) === 'inbox'; }
  get folderTitle(): string { const f=this.currentFolder; return f ? this.label(f) : this.t('Inbox'); }
  label(folder: MailFolder): string { return folderLabel(folder, k=>this.t(k)); }
  icon(folder: MailFolder): string {
    return ({inbox:'inbox',sent:'send',trash:'trash',junk:'junk',archive:'archive',drafts:'drafts'} as Record<string,string>)[folder.role] ?? 'folder';
  }
  get inboxUnread(): number { return this.folders.find(f=>f.role==='inbox')?.unread ?? 0; }
  badge(n: number): string { return badgeText(n); }
  get visibleItems(): MailHeader[] { return filterMessages(this.listing?.items ?? [], this.query); }
  get sections() { return splitInbox(this.visibleItems); }
  sender(row: MailHeader): string { return senderName(row.from) || this.t('Unknown sender'); }
  when(row: MailHeader): Date | null { return messageDate(row); }
  initial(row: MailHeader): string { return (this.sender(row).trim().charAt(0) || '?').toUpperCase(); }
  get syncLabel(): string {
    if (this.syncing) return this.t('Updating…');
    if (this.syncFailed) return this.t('Waiting for the mail server');
    return this.t('Automatic updates');
  }

  async ngOnInit():Promise<void>{
    await this.loadAccounts();
    this.timer = setInterval(()=>void this.autoRefresh(), REFRESH_MS);
  }
  ngOnDestroy():void { if (this.timer) clearInterval(this.timer); }
  ngOnChanges(changes: SimpleChanges):void {
    if (!changes['sync'] || !this.selectedId) return;
    const stamp = inboxStamp(this.sync, this.selectedId);
    const failed = this.sync?.accounts.find(a=>a.account_id===this.selectedId)?.error ?? false;
    this.syncFailed = failed;
    if (stamp && stamp !== this.lastStamp) {
      const first = this.lastStamp === '';
      this.lastStamp = stamp;
      // The inbox changed (new mail, or mail read elsewhere): refresh what is on screen without any click.
      if (!first) void this.autoRefresh(true);
    }
  }

  private clearStatus():void{this.notice='';this.error='';}
  private fail(error:unknown):void{this.error=errText(error);this.notice='';}
  private async operation(fn:()=>Promise<void>):Promise<void> {
    if(this.busy)return;
    this.busy=true;this.clearStatus();
    try{await fn();}catch(error){this.fail(error);}finally{this.busy=false;}
  }
  async loadAccounts():Promise<void> {
    await this.operation(async()=>{
      this.accounts=await api.mailListAccounts();
      if(!this.accounts.some(x=>x.id===this.selectedId)){
        this.selectedId=this.accounts[0]?.id??'';
        this.folders=[];this.listing=null;this.message=null;
      }
      if(this.selectedId)await this.loadDraftsCore();
    });
    if (this.selectedId && !this.folders.length) await this.connect();
  }
  async selectAccount(id:string):Promise<void>{
    if(id===this.selectedId||!this.canLeave())return;
    this.selectedId=id;this.folders=[];this.listing=null;this.message=null;
    this.selectedFolder='INBOX';this.accountForm=null;this.view='mail';
    this.drafts=[];this.lastStamp='';++this.loadVersion;
    if(id){
      await this.operation(()=>this.loadDraftsCore());
      await this.connect();
    }
  }
  private async loadDraftsCore():Promise<void>{
    this.drafts=await api.mailListDrafts(this.selectedId);
  }
  async loadDrafts():Promise<void>{
    if(!this.selectedId)return;
    await this.operation(()=>this.loadDraftsCore());
  }
  // ---- synchronisation (automatic)
  /** Load folders and the inbox: runs by itself when the section opens or the account changes. */
  async connect():Promise<void>{
    if(!this.selectedId||this.loading||this.syncing)return;
    const version=++this.loadVersion;
    const account=this.selectedId;
    const known=remember(account);
    // What was loaded before appears immediately; the server only confirms and updates it.
    if(known.folders.length){
      this.folders=known.folders;
      const inbox=known.folders.find(f=>f.role==='inbox')??known.folders[0];
      if(!known.folders.some(f=>f.name===this.selectedFolder))this.selectedFolder=inbox?.name??'INBOX';
      this.listing=known.lists.get(this.selectedFolder)??null;
    }
    this.loading=!this.listing;this.syncing=true;
    try{
      // One command, one connection: folders and the messages of the open folder together.
      const overview=await api.mailOverview(account,known.folders.length?this.selectedFolder:null);
      if(version!==this.loadVersion)return;
      this.applyOverview(account,overview);
      this.message=null;this.lastSyncAt=new Date();this.syncFailed=false;
      this.lastStamp=inboxStamp(this.sync,account);
      void api.mailSyncNow().catch(()=>undefined);
    }catch(error){
      if(version===this.loadVersion){this.syncFailed=true;if(!this.listing)this.fail(error);}
    }finally{
      if(version===this.loadVersion){this.loading=false;this.syncing=false;}
    }
  }
  private applyOverview(account:string,overview:MailOverview):void{
    const known=remember(account);
    known.folders=overview.folders;
    this.folders=overview.folders;
    if(overview.listing){
      known.lists.set(overview.listing.folder,overview.listing);
      this.selectedFolder=overview.listing.folder;
      this.listing=overview.listing;
    }
  }
  /** Background refresh of the folder counters and the open list; never interrupts reading or writing. */
  async autoRefresh(inboxOnly=false):Promise<void>{
    if(!this.selectedId||this.loading||this.refreshing||this.view!=='mail'||this.accountForm)return;
    if(inboxOnly&&!this.isInbox)inboxOnly=false;
    this.refreshing=true;
    const version=this.loadVersion;
    try{
      const account=this.selectedId;
      const overview=await api.mailOverview(account,this.selectedFolder);
      if(version!==this.loadVersion||this.view!=='mail')return;
      this.applyOverview(account,overview);
      // The open message stays open; if it is no longer on the server the next reload shows it.
      this.lastSyncAt=new Date();this.syncFailed=false;
    }catch{
      this.syncFailed=true;
    }finally{
      this.refreshing=false;
    }
  }
  async openFolder(folder=this.selectedFolder):Promise<void>{
    if(!this.selectedId)return;
    this.view='mail';this.message=null;this.query='';this.selectedFolder=folder;
    const version=++this.loadVersion;
    const account=this.selectedId;
    this.listing=remember(account).lists.get(folder)??null;
    this.loading=!this.listing;this.syncing=true;this.clearStatus();
    try{
      const listing=await api.mailListMessages(account,folder);
      if(version===this.loadVersion){this.listing=listing;remember(account).lists.set(folder,listing);this.syncFailed=false;}
    }catch(error){
      if(version===this.loadVersion){this.syncFailed=true;if(!this.listing)this.fail(error);}
    }finally{
      if(version===this.loadVersion){this.loading=false;this.syncing=false;}
    }
  }
  toggleSection(kind:'unread'|'rest'):void{this.collapsed={...this.collapsed,[kind]:!this.collapsed[kind]};}

  // ---- reading
  async read(item:MailHeader):Promise<void>{
    if(!this.selectedId)return;
    const wasUnread=!item.seen;
    // The reader opens at once with the header we already have; the body follows.
    this.message={header:{...item},body:'',html_only:false,attachment_count:0};
    this.messageLoading=true;
    try{
      const detail=await api.mailReadMessage(this.selectedId,item.folder,item.uid,item.uid_validity);
      if(this.message?.header.uid===item.uid)this.message=detail;
    }catch(error){
      if(this.message?.header.uid===item.uid)this.message=null;
      this.fail(error);return;
    }finally{this.messageLoading=false;}
    if(wasUnread)void this.markSeen(item,true);
  }
  /** Opening a message marks it as read on the server and in the list, and refreshes the menu counter. */
  async markSeen(item:MailHeader,seen:boolean):Promise<void>{
    try{
      await api.mailSetSeen(this.selectedId,item.folder,item.uid,item.uid_validity,seen);
      item.seen=seen;
      if(this.message&&this.message.header.uid===item.uid)this.message.header.seen=seen;
      const folder=this.folders.find(f=>f.name===item.folder);
      if(folder?.unread!==null&&folder?.unread!==undefined)folder.unread=Math.max(0,folder.unread+(seen?-1:1));
      void api.mailSyncNow().catch(()=>undefined);
    }catch(error){this.fail(error);}
  }
  closeMessage():void{this.message=null;}
  async toggleSeen():Promise<void>{
    if(!this.message)return;
    const header=this.listing?.items.find(i=>i.uid===this.message!.header.uid)??this.message.header;
    await this.markSeen(header,!header.seen);
  }

  // ---- compose (floating window)
  startCompose():void{
    if(!this.selectedId||!this.canLeave())return;
    this.composer={...blankComposer(),body:this.selected?.signature??''};
    this.openComposer();
  }
  reply(forward=false):void {
    if(!this.selectedId||!this.message||!this.canLeave())return;
    const header=this.message.header;
    const mail=/<([^<>\s]+@[^<>\s]+)>/.exec(header.from)?.[1]??header.from.trim();
    const prefix=forward?'Fwd: ':'Re: ';
    const messageSubject=header.subject.replace(/^(Re:|Fwd:)\s*/i,'');
    const quote=this.message.body.split('\n').slice(0,100).map(line=>'> '+line).join('\n').slice(0,5000);
    const signature=this.selected?.signature??'';
    this.composer={
      ...blankComposer(),to:forward?'':mail,subject:prefix+messageSubject,
      body:(signature ? signature+'\n\n' : '')+(quote?'\n'+quote:'')
    };
    this.openComposer();
  }
  openDraft(draft:MailDraft):void{
    if(!this.canLeave())return;
    this.composer={id:draft.id,revision:draft.revision,to:draft.to,
      cc:draft.cc,bcc:draft.bcc,subject:draft.subject,body:draft.body,attachments:[...(draft.attachments??[])]};
    this.openComposer();
  }
  private openComposer():void{
    this.originalComposer=JSON.stringify(this.composer);
    this.showCc=Boolean(this.composer.cc);this.showBcc=Boolean(this.composer.bcc);
    this.composeOpen=true;this.composeMode='normal';this.clearStatus();
  }
  /** Close the window; unsaved changes are confirmed first. */
  closeCompose():void{
    if(!this.canLeave())return;
    this.composeOpen=false;this.composer=blankComposer();this.originalComposer=JSON.stringify(this.composer);
  }
  minimizeCompose():void{this.composeMode=this.composeMode==='minimized'?'normal':'minimized';}
  expandCompose():void{this.composeMode=this.composeMode==='expanded'?'normal':'expanded';}
  private canLeave():boolean{
    return !this.hasUnsavedComposer || window.confirm(this.t('Discard unsaved changes to this message?'));
  }
  // ---- views
  showMail():void{
    this.view='mail';this.accountForm=null;this.clearStatus();
    void this.autoRefresh();
  }
  showDrafts():void{
    if(!this.selectedId)return;
    this.view='drafts';this.message=null;this.clearStatus();
    void this.loadDrafts();
  }
  toggleSettings():void{
    if(this.view==='settings'){this.showMail();return;}
    this.view='settings';this.accountForm=null;this.clearStatus();
  }
  // ---- accounts and settings
  editAccount(account?:MailAccount):void {
    const copy=account??freshAccount();
    this.accountForm={...copy,imap_password:'',smtp_password:''};
    this.view='settings';this.clearStatus();
  }
  cancelAccount():void{this.accountForm=null;}
  async saveAccount():Promise<void>{
    if(!this.accountForm)return;
    let saved=false;
    await this.operation(async()=>{
      const account=await api.mailSaveAccount(this.accountForm!);
      this.accountForm!.imap_password='';
      this.accountForm!.smtp_password='';
      this.accountForm=null;
      this.accounts=await api.mailListAccounts();
      this.selectedId=account.id;
      this.folders=[];this.listing=null;this.message=null;this.lastStamp='';
      await this.loadDraftsCore();
      this.notice=this.t('Account saved. Password is stored in the system keychain.');
      saved=true;
    });
    if(saved){this.view='mail';await this.connect();}
  }
  async removeAccount(account:MailAccount):Promise<void>{
    if(!window.confirm(this.t('Remove this account and its local drafts? Remote mail will not be deleted.')))return;
    await this.operation(async()=>{
      await api.mailRemoveAccount(account.id);
      this.accounts=await api.mailListAccounts();
      this.selectedId=this.accounts[0]?.id??'';
      this.accountForm=null;this.drafts=[];this.folders=[];this.listing=null;this.message=null;this.lastStamp='';
      if(this.selectedId)await this.loadDraftsCore();
      this.notice=this.t('Account removed.');
    });
    void api.mailSyncNow().catch(()=>undefined);
    if(this.selectedId)await this.connect();
  }
  async testAccount(account:MailAccount):Promise<void>{
    await this.operation(async()=>{
      const checked=await api.mailTestAccount(account.id);
      this.notice=this.t('IMAP connection successful:')+' '+checked.folders+' '+this.t('folders');
    });
  }
  setNotifications(on:boolean):void{this.notifications=on;this.notificationsChange.emit(on);}

  // ---- compose actions
  attachmentName(path:string):string { return path.split(/[\\/]/).pop()??path; }
  async addAttachments():Promise<void>{
    if(this.busy)return;
    try{
      const selection=await open({multiple:true,title:this.t('Choose attachments (max 5 files)')});
      const paths=typeof selection==='string'?[selection]:Array.isArray(selection)?selection:[];
      const merged=[...new Set([...this.composer.attachments,...paths])];
      if(merged.length>5){this.error=this.t('You can attach up to five files.');return;}
      this.composer.attachments=merged;
      this.error='';
    }catch(error){this.fail(error);}
  }
  removeAttachment(path:string):void{
    this.composer.attachments=this.composer.attachments.filter(p=>p!==path);
  }
  async saveDraft():Promise<void>{
    if(!this.selectedId)return;
    await this.operation(async()=>{
      const draft:MailDraftInput={
        id:this.composer.id,revision:this.composer.revision,
        account_id:this.selectedId,to:this.composer.to,cc:this.composer.cc,bcc:this.composer.bcc,
        subject:this.composer.subject,body:this.composer.body,
        attachments:[...this.composer.attachments]
      };
      const saved=await api.mailSaveDraft(draft);
      this.composer.id=saved.id;this.composer.revision=saved.revision;
      this.originalComposer=JSON.stringify(this.composer);
      await this.loadDraftsCore();
      this.notice=this.t('Local draft saved.');
    });
  }
  async removeDraft(draft:MailDraft):Promise<void>{
    if(!window.confirm(this.t('Delete the selected local draft?')))return;
    await this.operation(async()=>{
      await api.mailRemoveDraft(this.selectedId,draft.id,draft.revision);
      this.drafts=this.drafts.filter(d=>d.id!==draft.id);
      if(this.composer.id===draft.id){this.composer=blankComposer();this.originalComposer=JSON.stringify(this.composer);this.composeOpen=false;}
      this.notice=this.t('Draft deleted.');
    });
  }
  /** Throw the open message away (and its saved draft, if any). */
  async discardCompose():Promise<void>{
    const draft=this.composer.id?this.drafts.find(d=>d.id===this.composer.id):undefined;
    if(draft){await this.removeDraft(draft);return;}
    if(this.hasUnsavedComposer&&!window.confirm(this.t('Discard unsaved changes to this message?')))return;
    this.composeOpen=false;this.composer=blankComposer();this.originalComposer=JSON.stringify(this.composer);
  }
  async send():Promise<void>{
    if(!this.selectedId)return;
    if(!window.confirm(this.t('Send this email now? This action cannot be undone.')))return;
    await this.operation(async()=>{
      const current={...this.composer};
      // One deliberate click issues one send. Never auto-retry on a network error.
      const result=await api.mailSend({
        account_id:this.selectedId,to:current.to,cc:current.cc,bcc:current.bcc,
        subject:current.subject,body:current.body,attachments:current.attachments,confirm:true
      });
      if(result.delivery==='accepted'){
        this.composer=blankComposer();this.originalComposer=JSON.stringify(this.composer);
        this.composeOpen=false;
        if(current.id && current.revision!==null){
          try{await api.mailRemoveDraft(this.selectedId,current.id,current.revision);}
          catch{/* Delivery was accepted; do not mask it with a draft cleanup error. */}
          await this.loadDraftsCore();
        }
      }
      this.notice=this.t('SMTP accepted the message.')+' '+(
        result.sent_copy==='saved'?this.t('A copy was stored in Sent.'):
        result.sent_copy==='not_saved'?this.t('Could not store a copy in Sent. Do not resend.'):
        this.t('No Sent folder configured. Check the provider mailbox.'));
    });
  }
}
