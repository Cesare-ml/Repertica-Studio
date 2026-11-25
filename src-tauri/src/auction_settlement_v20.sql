-- Reserved schema v20 proposal: immutable published association and receipt history.
-- Installed only through the explicit native migration, never a renderer command.
CREATE TABLE auction_published_bindings (
    scope TEXT NOT NULL CHECK(length(scope)=36),
    auction TEXT NOT NULL,
    tenant TEXT NOT NULL,
    association TEXT NOT NULL,
    cloud_work TEXT NOT NULL,
    local_work TEXT NOT NULL CHECK(length(local_work)=36),
    owner TEXT NOT NULL,
    request_id TEXT NOT NULL,
    selection_digest TEXT NOT NULL CHECK(length(selection_digest)=64 AND selection_digest NOT GLOB '*[^0-9a-f]*'),
    published_at TEXT NOT NULL,
    PRIMARY KEY(scope,auction),
    UNIQUE(scope,association),
    UNIQUE(scope,owner,request_id),
    UNIQUE(scope,auction,tenant,association,cloud_work,local_work,owner)
);
CREATE INDEX auction_published_bindings_work ON auction_published_bindings(scope,local_work,owner);
-- No FK to artworks: deleting a local artwork must not erase historical evidence
-- or change the core deletion contract. Native resolution checks existence.
CREATE TRIGGER auction_published_bindings_no_replace BEFORE INSERT ON auction_published_bindings
WHEN EXISTS(SELECT 1 FROM auction_published_bindings b WHERE b.scope=NEW.scope AND
    (b.auction=NEW.auction OR b.association=NEW.association OR (b.owner=NEW.owner AND b.request_id=NEW.request_id)))
BEGIN SELECT RAISE(ABORT,'immutable published association'); END;
CREATE TRIGGER auction_published_bindings_no_update BEFORE UPDATE ON auction_published_bindings
BEGIN SELECT RAISE(ABORT,'immutable published association'); END;
CREATE TRIGGER auction_published_bindings_no_delete BEFORE DELETE ON auction_published_bindings
BEGIN SELECT RAISE(ABORT,'immutable published association'); END;

CREATE TABLE auction_settlement_receipts (
    scope TEXT NOT NULL, auction TEXT NOT NULL, tenant TEXT NOT NULL,
    association TEXT NOT NULL, cloud_work TEXT NOT NULL, local_work TEXT NOT NULL,
    owner TEXT NOT NULL, sequence INTEGER NOT NULL CHECK(sequence BETWEEN 1 AND 9007199254740991),
    snapshot TEXT NOT NULL CHECK(json_valid(snapshot) AND length(CAST(snapshot AS BLOB))<=65536),
    acknowledged_at TEXT,
    PRIMARY KEY(scope,auction,sequence),
    FOREIGN KEY(scope,auction,tenant,association,cloud_work,local_work,owner)
        REFERENCES auction_published_bindings(scope,auction,tenant,association,cloud_work,local_work,owner)
);
CREATE TRIGGER auction_settlement_receipts_no_replace BEFORE INSERT ON auction_settlement_receipts
WHEN EXISTS(SELECT 1 FROM auction_settlement_receipts r WHERE r.scope=NEW.scope AND r.auction=NEW.auction AND r.sequence=NEW.sequence)
BEGIN SELECT RAISE(ABORT,'immutable settlement receipt'); END;
CREATE TRIGGER auction_settlement_receipts_ack_only BEFORE UPDATE ON auction_settlement_receipts
WHEN NEW.scope<>OLD.scope OR NEW.auction<>OLD.auction OR NEW.tenant<>OLD.tenant OR
     NEW.association<>OLD.association OR NEW.cloud_work<>OLD.cloud_work OR NEW.local_work<>OLD.local_work OR
     NEW.owner<>OLD.owner OR NEW.sequence<>OLD.sequence OR NEW.snapshot<>OLD.snapshot OR
     OLD.acknowledged_at IS NOT NULL OR NEW.acknowledged_at IS NULL
BEGIN SELECT RAISE(ABORT,'immutable settlement receipt'); END;
CREATE TRIGGER auction_settlement_receipts_no_delete BEFORE DELETE ON auction_settlement_receipts
BEGIN SELECT RAISE(ABORT,'immutable settlement receipt'); END;
