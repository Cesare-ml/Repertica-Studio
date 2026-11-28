-- Read-only projection from the same immutable local copy used for v2.
BEGIN READ ONLY;
SET LOCAL statement_timeout = '5s';
WITH requested(id) AS (SELECT unnest(:'artwork_ids'::integer[])),
selected AS (
  SELECT o.id,btrim(o.codice_archivio) AS code,o.id_upload_immagine_principale AS cover_id
  FROM requested r JOIN public.opere o ON o.id=r.id
  WHERE o.id_utente_proprietario=CAST(:'owner_id' AS integer)
), slots AS (
  SELECT s.id AS artwork_id,s.cover_id AS upload_id,0 AS position,NULL::text AS kind,NULL::text AS caption
  FROM selected s WHERE s.cover_id IS NOT NULL
  UNION ALL
  SELECT s.id,io.id_upload,CAST(row_number() OVER(PARTITION BY s.id ORDER BY io.ordine_visualizzazione,io.id) AS integer),io.tipo,io.descrizione
  FROM selected s JOIN public.immagini_opera io ON io.id_opera=s.id
), resolved AS (
  SELECT sl.artwork_id,sl.position,jsonb_build_object('position',sl.position,'kind',sl.kind,'caption',sl.caption,
    'path',b.storage_key,'sha256',b.sha256,'size',CAST(b.dimensione_byte AS text)) AS photo
  FROM slots sl
  LEFT JOIN public.uploads u ON u.id=sl.upload_id AND u.id_utente_proprietario=CAST(:'owner_id' AS integer) AND u.stato='READY'
  LEFT JOIN public.media_revisions mr ON mr.id=u.current_revision_id AND mr.id_upload=u.id AND mr.status='ACTIVE'
  LEFT JOIN public.media_blobs b ON b.id=mr.result_blob_id AND b.id_utente_proprietario=CAST(:'owner_id' AS integer)
    AND b.storage_provider='LOCAL' AND b.status='AVAILABLE'
), projected AS (
  SELECT s.code,COALESCE((SELECT jsonb_agg(r.photo ORDER BY r.position) FROM resolved r WHERE r.artwork_id=s.id),'[]'::jsonb) AS photos
  FROM selected s WHERE (SELECT count(*) FROM requested) BETWEEN 1 AND 1000
    AND (SELECT count(DISTINCT id) FROM requested)=(SELECT count(*) FROM requested)
    AND (SELECT count(*) FROM selected)=(SELECT count(*) FROM requested)
)
SELECT COALESCE(json_agg(row_to_json(projected)),'[]'::json) FROM projected;
COMMIT;
