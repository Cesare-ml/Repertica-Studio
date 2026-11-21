import { CommonModule } from '@angular/common';
import { Component, Input, OnInit } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { open } from '@tauri-apps/plugin-dialog';
import { api } from './api';
import type { AppLanguage } from './i18n';
import type { Collector, MailAccount, MailAccountInput, MailDetail, MailDraft, MailDraftInput, MailFolder, MailHeader, MailList } from './models';
import { mailText } from './mail-i18n';

type MailView = 'inbox' | 'drafts' | 'compose' | 'accounts';
type Composer = { id: string | null; revision: number | null; to: string; cc: string; bcc: string; subject: string; body: string; attachments: string[] };
const blankComposer = (): Composer => ({ id:null,revision:null,to:'',cc:'',bcc:'',subject:'',body:'',attachments:[] });
const freshAccount = (): MailAccountInput => ({
  id:null,email:'',display_name:'',imap_host:'',imap_port:993,imap_username:'',
  smtp_host:'',smtp_port:465,smtp_username:'',sent_folder:'',signature:'',
  imap_password:'',smtp_password:''
});
const errText = (err: unknown) => typeof err === 'string' ? err : err instanceof Error ? err.message : 'Operation failed';

/**
 * A personal mail client, not the marketing Email Platform.
 * Remote operations always start from a user click. No HTML mail or remote
 * resources are rendered and credentials are never stored in Angular state
 * after account save. Outgoing messages always require explicit confirmation.
 */
@Component({
  selector:'rp-business-mail', standalone:true,
  imports:[CommonModule,FormsModule],
  templateUrl:'./business-mail.component.html',
  styleUrls:['./business-mail.component.css']
})
export class BusinessMailComponent implements OnInit {
  @Input() language: AppLanguage = 'en';
  @Input() collectors: Collector[] = [];
  accounts: MailAccount[] = [];
  selectedId = '';
  folders: MailFolder[] = [];
  selectedFolder = 'INBOX';
  listing: MailList | null = null;
  message: MailDetail | null = null;
  drafts: MailDraft[] = [];
  composer: Composer = blankComposer();
  private originalComposer = '';
  accountForm: MailAccountInput | null = null;
  view: MailView = 'inbox';
  busy = false;
  notice = '';
  error = '';
  private loadVersion = 0;
  get selected():MailAccount | undefined { return this.accounts.find(x=>x.id===this.selectedId); }
  get hasUnsavedComposer():boolean {
    return this.view==='compose' && JSON.stringify(this.composer)!==this.originalComposer;
  }
  t(key:string):string { return mailText(this.language,key); }
  get fromCollectors():{name:string;email:string}[] {
    return this.collectors.filter(c=>Boolean(c.email)).map(c=>({name:c.name,email:c.email!})).slice(0,300);
  }
  async ngOnInit():Promise<void>{await this.loadAccounts();}
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
  }
  async selectAccount(id:string):Promise<void>{
    if(id===this.selectedId||!this.canLeave())return;
    this.selectedId=id;this.folders=[];this.listing=null;this.message=null;
    this.selectedFolder='INBOX';this.accountForm=null;this.view='inbox';
    this.drafts=[];++this.loadVersion;
    if(id)await this.operation(()=>this.loadDraftsCore());
  }
  private async loadDraftsCore():Promise<void>{
    this.drafts=await api.mailListDrafts(this.selectedId);
  }
  async loadDrafts():Promise<void>{
    if(!this.selectedId)return;
    await this.operation(()=>this.loadDraftsCore());
  }
  editAccount(account?:MailAccount):void {
    if(!this.canLeave())return;
    const copy=account??freshAccount();
    this.accountForm={...copy,imap_password:'',smtp_password:''};
    this.view='accounts';this.clearStatus();
  }
  cancelAccount():void{this.accountForm=null;}
  async saveAccount():Promise<void>{
    if(!this.accountForm)return;
    await this.operation(async()=>{
      const account=await api.mailSaveAccount(this.accountForm!);
      this.accountForm!.imap_password='';
      this.accountForm!.smtp_password='';
      this.accountForm=null;
      this.accounts=await api.mailListAccounts();
      this.selectedId=account.id;
      this.folders=[];this.listing=null;this.message=null;
      await this.loadDraftsCore();
      this.notice=this.t('Account saved. Password is stored in the system keychain.');
    });
  }
  async removeAccount(account:MailAccount):Promise<void>{
    if(!window.confirm(this.t('Remove this account and its local drafts? Remote mail will not be deleted.')))return;
    await this.operation(async()=>{
      await api.mailRemoveAccount(account.id);
      this.accounts=await api.mailListAccounts();
      this.selectedId=this.accounts[0]?.id??'';
      this.accountForm=null;this.drafts=[];this.folders=[];this.listing=null;this.message=null;
      if(this.selectedId)await this.loadDraftsCore();
      this.notice=this.t('Account removed.');
    });
  }
  async testAccount():Promise<void>{
    if(!this.selectedId)return;
    await this.operation(async()=>{
      const checked=await api.mailTestAccount(this.selectedId);
      this.notice=this.t('IMAP connection successful:')+' '+checked.folders+' '+this.t('folders');
    });
  }
  async connect():Promise<void>{
    if(!this.selectedId)return;
    await this.operation(async()=>{
      this.folders=await api.mailListFolders(this.selectedId);
      const folder=this.folders.find(f=>f.role==='inbox')??this.folders[0];
      this.selectedFolder=folder?.name??'INBOX';
      this.listing=null;this.message=null;
      this.view='inbox';
      await this.loadMessagesCore();
    });
  }
  private async loadMessagesCore():Promise<void>{
    this.listing=await api.mailListMessages(this.selectedId,this.selectedFolder);
    this.message=null;
  }
  async loadMessages(folder=this.selectedFolder):Promise<void>{
    if(!this.selectedId||!this.canLeave())return;
    await this.operation(async()=>{
      this.selectedFolder=folder;this.view='inbox';
      await this.loadMessagesCore();
    });
  }
  async read(item:MailHeader):Promise<void>{
    if(!this.selectedId)return;
    await this.operation(async()=>{
      this.message=await api.mailReadMessage(this.selectedId,item.folder,item.uid,item.uid_validity);
    });
  }
  startCompose():void{
    if(!this.selectedId||!this.canLeave())return;
    this.composer={...blankComposer(),body:this.selected?.signature??''};
    this.originalComposer=JSON.stringify(this.composer);
    this.view='compose';this.clearStatus();
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
    this.originalComposer=JSON.stringify(this.composer);
    this.view='compose';this.clearStatus();
  }
  openDraft(draft:MailDraft):void{
    if(!this.canLeave())return;
    this.composer={id:draft.id,revision:draft.revision,to:draft.to,
      cc:draft.cc,bcc:draft.bcc,subject:draft.subject,body:draft.body,attachments:[...(draft.attachments??[])]};
    this.originalComposer=JSON.stringify(this.composer);
    this.view='compose';this.clearStatus();
  }
  private canLeave():boolean{
    return !this.hasUnsavedComposer || window.confirm(this.t('Discard unsaved changes to this message?'));
  }
  showDrafts():void{
    if(!this.selectedId||!this.canLeave())return;
    this.view='drafts';this.clearStatus();
    void this.loadDrafts();
  }
  showInbox():void{
    if(!this.canLeave())return;
    this.view='inbox';this.clearStatus();
  }
  showAccounts():void{
    if(!this.canLeave())return;
    this.view='accounts';this.accountForm=null;this.clearStatus();
  }
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
      if(this.composer.id===draft.id){this.composer=blankComposer();this.view='drafts';}
      this.notice=this.t('Draft deleted.');
    });
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
        this.view='inbox';
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
