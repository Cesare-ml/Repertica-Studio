-- Run only against an explicitly selected local Gestionale Opere copy.
-- psql variables are quoted literals, never interpolated SQL fragments.
BEGIN READ ONLY;
SET LOCAL statement_timeout = '5s';
WITH requested(id) AS (
    SELECT unnest(:'artwork_ids'::integer[])
), eligible AS (
    SELECT o.codice_archivio, o.titolo, o.anno_realizzazione, o.tecnica, o.supporto,
           CAST(o.larghezza_cm AS text) AS larghezza_cm,
           CAST(o.altezza_cm AS text) AS altezza_cm,
           CAST(o.profondita_cm AS text) AS profondita_cm,
           o.descrizione, CAST(o.stato AS text) AS stato,
           o.firma_presente, o.cornice_presente,
           CAST(o.vernice_finale AS text) AS vernice_finale,
           CAST(o.coefficiente_artista_override AS text) AS coefficiente_artista_override,
           CAST(o.prezzo_personalizzato AS text) AS prezzo_personalizzato,
           NULL AS id_vendita
    FROM requested r
    JOIN public.opere o ON o.id = r.id
    WHERE o.id_utente_proprietario = CAST(:'owner_id' AS integer)
      AND o.stato IN ('DISPONIBILE', 'RISERVATA')
      AND o.firma_presente IS NOT NULL AND o.cornice_presente IS NOT NULL
      AND o.vernice_finale IN ('OPACA', 'LUCIDA', 'SATINATA')
      -- Check all sale links, including inconsistent cross-owner links. Never export them.
      AND NOT EXISTS (SELECT 1 FROM public.vendite v WHERE v.id_opera = o.id)
), guarded AS (
    SELECT * FROM eligible
    WHERE (SELECT count(*) FROM requested) BETWEEN 1 AND 1000
      AND (SELECT count(DISTINCT id) FROM requested) = (SELECT count(*) FROM requested)
      AND (SELECT count(*) FROM eligible) = (SELECT count(*) FROM requested)
)
SELECT COALESCE(json_agg(row_to_json(guarded) ORDER BY codice_archivio), '[]'::json)
FROM guarded;
COMMIT;
