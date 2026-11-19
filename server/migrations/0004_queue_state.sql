-- 0004 — download queue state on chapters (M2: single-writer discipline stays, now with an
-- explicit DB-backed FIFO queue; safety: state is data, not memory).

ALTER TABLE chapter ADD COLUMN state TEXT NOT NULL DEFAULT 'pending';
ALTER TABLE chapter ADD COLUMN state_error TEXT;
ALTER TABLE chapter ADD COLUMN queued_at INTEGER;

-- legacy rows were treated as fully downloaded by the M1 worker contract
UPDATE chapter SET state = 'downloaded' WHERE state = 'legacy';

CREATE INDEX idx_chapter_state ON chapter (state, queued_at);
