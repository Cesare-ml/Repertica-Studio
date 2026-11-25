import { test } from 'node:test';
import assert from 'node:assert/strict';
import { AuctionConnection } from '../src/app/auction-connection.ts';
const org = {tenantId:'tenant-test',artistName:'Synthetic',role:'owner',status:'active',publicationEligible:true,publishedThisMonth:1};
const body = JSON.stringify({tenantId:'tenant-test',title:'Public only'});
const ref = {artworkId:'a',scope:'archive',revision:'r'};
function fixture(overrides = {}) {
  const calls = [];
  const responses = {status:{status:'connected'},disconnect:{status:'disconnected'},organizations:[org],
    selectOrganization:{organization:org,previews:[]},reviewNew:{confirmationId:'c',publicBody:body,action:'prepare'},
    reviewPending:{confirmationId:'d',publicBody:body,action:'reconcile'},confirmPrepare:{revision:'next'},
    confirmReconcile:{tenantId:'tenant-test',auctionId:'auction',slug:'artist--art'},cancelReview:null,...overrides};
  return {calls,session:new AuctionConnection(async request=>{
    calls.push(request); const response=responses[request.action];
    if (response instanceof Error) throw response;
    return response;
  })};
}
async function select(session) { await session.refreshStatus(); await session.loadOrganizations(); await session.selectOrganization('tenant-test'); }

test('disabled native status allows offline use and does not call remote actions', async () => {
  const {session,calls}=fixture({status:{status:'disabled'},disconnect:{status:'disabled'}});
  await session.refreshStatus(); assert.equal(session.state,'disabled');
  await assert.rejects(session.loadOrganizations(),/auction_not_connected/);
  await assert.rejects(session.reviewDraft(ref,body,false),/auction_not_connected/);
  assert.deepEqual(calls,[{action:'status'}]);
  await session.disconnect(); assert.equal(session.state,'disabled');
});
test('prepare and reconcile each need a fresh review, with exact capability only', async () => {
  const {session,calls}=fixture(); await select(session);
  await assert.rejects(session.confirm(),/auction_review_required/);
  await session.reviewDraft({...ref,token:'never send'},body,false);
  assert.deepEqual(calls.at(-1),{action:'reviewNew',...ref,publicBody:body});
  assert.deepEqual(await session.confirm(),{action:'prepare',revision:'next'});
  assert.deepEqual(calls.at(-1),{action:'confirmPrepare',confirmationId:'c'});
  await assert.rejects(session.confirm(),/auction_review_required/);
  await session.reviewDraft({...ref,revision:'next'},body,true);
  assert.deepEqual(calls.at(-1),{action:'reviewPending',...ref,revision:'next'});
  assert.deepEqual(await session.confirm(),{action:'reconcile',receipt:{tenantId:'tenant-test',auctionId:'auction',slug:'artist--art'}});
  assert.deepEqual(calls.at(-1),{action:'confirmReconcile',confirmationId:'d'});
});
test('viewer and cross-tenant requests never reach review IPC', async () => {
  const viewer={...org,role:'viewer',publicationEligible:false};
  const {session,calls}=fixture({selectOrganization:{organization:viewer,previews:[]}}); await select(session);
  await assert.rejects(session.reviewDraft(ref,body,false),/auction_access_denied/);
  assert.equal(calls.length,3);
  const second=fixture(); await select(second.session);
  await assert.rejects(second.session.reviewDraft(ref,body.replace('tenant-test','other'),true),/auction_access_denied/);
  assert.equal(second.calls.length,3);
});
test('changed review, failed confirmation, cancellation and logout fail closed', async () => {
  const bad=fixture({reviewNew:{confirmationId:'c',publicBody:'{}',action:'prepare'}}); await select(bad.session);
  await assert.rejects(bad.session.reviewDraft(ref,body,false),/auction_invalid_response/);
  assert.equal(bad.session.state,'unknown'); assert.equal(bad.session.review,null);
  const failed=fixture({confirmPrepare:new Error('timeout')}); await select(failed.session);
  await failed.session.reviewDraft(ref,body,false); await assert.rejects(failed.session.confirm(),/timeout/);
  await assert.rejects(failed.session.confirm(),/auction_not_connected/);
  assert.equal(failed.calls.filter(c=>c.action==='confirmPrepare').length,1);
  const {session,calls}=fixture(); await select(session);
  await session.reviewDraft(ref,body,false); await session.cancelReview();
  assert.deepEqual(calls.at(-1),{action:'cancelReview'});
  await assert.rejects(session.confirm(),/auction_review_required/);
  await session.reviewDraft(ref,body,false); await session.disconnect();
  assert.equal(session.review,null); assert.equal(session.selected,null); assert.equal(session.state,'disconnected');
  await assert.rejects(session.confirm(),/auction_not_connected/);
});
test('concurrent UI calls cannot race selection or confirmation', async () => {
  let release;
  const waiting=new Promise(resolve=>{release=resolve;});
  const session=new AuctionConnection(async request=>request.action==='status'?waiting:[]);
  const read=session.refreshStatus();
  await assert.rejects(session.disconnect(),/auction_editor_busy/);
  await assert.rejects(session.loadOrganizations(),/auction_editor_busy/);
  release({status:'disabled'}); await read; assert.equal(session.state,'disabled');
});
