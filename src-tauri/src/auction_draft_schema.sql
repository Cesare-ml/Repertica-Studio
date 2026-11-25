CREATE TABLE IF NOT EXISTS auction_archive_scope (
        singleton INTEGER PRIMARY KEY CHECK(singleton=1),
        scope TEXT NOT NULL CHECK(length(scope)=36)
    );
    CREATE TABLE IF NOT EXISTS auction_drafts (
        artwork_id TEXT PRIMARY KEY NOT NULL,
        revision TEXT NOT NULL CHECK(length(revision)=36),
        record_json TEXT NOT NULL CHECK(json_valid(record_json) AND length(CAST(record_json AS BLOB))<=64000)
    );
