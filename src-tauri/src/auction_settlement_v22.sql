-- Native-only durable record of an explicitly reviewed ACK attempt.
-- No credentials, grants, user contact data, or remote routing information.
-- A scanned marker is NOT a confirmed receipt; only the ledger stamp is.
CREATE TABLE auction_settlement_ack_attempts (
    scope TEXT NOT NULL, auction TEXT NOT NULL,
    sequence INTEGER NOT NULL CHECK(typeof(sequence)='integer' AND sequence BETWEEN 1 AND 9007199254740991),
    owner TEXT NOT NULL, result_id TEXT NOT NULL CHECK(length(result_id) BETWEEN 1 AND 180),
    checked_through INTEGER CHECK(checked_through IS NULL OR
        (typeof(checked_through)='integer' AND checked_through>sequence AND checked_through<=9007199254740991)),
    PRIMARY KEY(scope,auction,sequence),
    FOREIGN KEY(scope,auction,sequence) REFERENCES auction_settlement_receipts(scope,auction,sequence)
);
CREATE TRIGGER auction_ack_attempt_no_replace BEFORE INSERT ON auction_settlement_ack_attempts
WHEN EXISTS(SELECT 1 FROM auction_settlement_ack_attempts a WHERE a.scope=NEW.scope AND a.auction=NEW.auction AND a.sequence=NEW.sequence)
BEGIN SELECT RAISE(ABORT,'immutable ACK attempt'); END;
CREATE TRIGGER auction_ack_attempt_checked_only BEFORE UPDATE ON auction_settlement_ack_attempts
WHEN NEW.scope<>OLD.scope OR NEW.auction<>OLD.auction OR NEW.sequence<>OLD.sequence OR
     NEW.owner<>OLD.owner OR NEW.result_id<>OLD.result_id OR
     OLD.checked_through IS NOT NULL OR NEW.checked_through IS NULL
BEGIN SELECT RAISE(ABORT,'immutable ACK attempt'); END;
CREATE TRIGGER auction_ack_attempt_no_delete BEFORE DELETE ON auction_settlement_ack_attempts
BEGIN SELECT RAISE(ABORT,'immutable ACK attempt'); END;
